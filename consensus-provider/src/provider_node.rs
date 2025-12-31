//! # Provider 节点核心模块
//!
//! 这个模块实现了 Provider 节点的核心逻辑，负责管理任务执行、
//! 与共识服务器通信以及监控节点状态。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use crate::types::*;
use crate::task_executor::{TaskExecutor, TaskExecution};
use crate::consensus_client::ConsensusClient;
use crate::api_server::{ApiServer, ProviderStatusInfo};
use crate::logging::*;
use crate::config::ProviderConfig;

/// Provider 节点
pub struct ProviderNode {
    /// 节点配置
    config: ProviderConfig,
    /// 任务执行器
    task_executor: TaskExecutor,
    /// 共识客户端
    consensus_client: ConsensusClient,
    /// API 服务器
    api_server: ApiServer,
    /// 活跃任务追踪器
    active_tasks: Arc<RwLock<HashMap<String, TaskExecution>>>,
    /// 节点统计信息
    stats: Arc<RwLock<NodeStats>>,
    /// 节点是否正在运行
    running: Arc<RwLock<bool>>,
}

impl ProviderNode {
    /// 创建新的 Provider 节点
    pub async fn new(config: ProviderConfig) -> Result<Self, anyhow::Error> {
        log::info!("创建 Provider 节点: {}", config.node_id);

        // 创建任务执行器
        let task_executor = TaskExecutor::new();

        // 创建共识客户端
        let consensus_client = ConsensusClient::new("http://localhost:7465").await?;
        let mut consensus_client_clone = consensus_client.clone();
        consensus_client_clone.register_provider(&config.node_id).await?;

        // 创建 API 服务器
        let api_server = ApiServer::new(config.api_port);

        // 初始化统计信息
        let stats = NodeStats::new(&config.node_id);

        Ok(Self {
            config,
            task_executor,
            consensus_client: consensus_client_clone,
            api_server,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(stats)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 启动 Provider 节点
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        log::info!("启动 Provider 节点: {}", self.config.node_id);
        log_system_status("provider_node", "starting", Some(&self.config.node_id));

        // 标记为运行状态
        *self.running.write().await = true;

        // 启动 API 服务器
        let api_server_clone = self.api_server.clone();
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api_server_clone.start().await {
                log_error_with_context("启动 API 服务器失败", &*e);
            }
        });

        // 启动任务轮询循环 (暂时禁用以修复编译错误)
        // let task_polling_handle = self.start_task_polling(); // 暂时禁用

        // 启动健康检查
        let health_check_handle = self.start_health_checks();

        // 启动统计更新
        let stats_update_handle = self.start_stats_updates();

        // 等待所有任务完成或接收到停止信号
        tokio::select! {
            _ = api_handle => {
                log::warn!("API 服务器停止");
            }
            // 任务轮询暂时禁用
            _ = health_check_handle => {
                log::warn!("健康检查停止");
            }
            _ = stats_update_handle => {
                log::warn!("统计更新停止");
            }
        }

        // 标记为停止状态
        *self.running.write().await = false;
        log_system_status("provider_node", "stopped", Some(&self.config.node_id));

        Ok(())
    }

    /// 停止 Provider 节点
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        log::info!("停止 Provider 节点: {}", self.config.node_id);
        *self.running.write().await = false;
        Ok(())
    }

    /// 获取节点状态
    pub async fn get_status(&self) -> ProviderStatus {
        let active_tasks = self.active_tasks.read().await.len();
        let _stats = self.stats.read().await;
        let running = *self.running.read().await;

        if !running {
            ProviderStatus::Stopped
        } else if active_tasks > 0 {
            ProviderStatus::Busy
        } else {
            ProviderStatus::Running
        }
    }

    /// 获取节点统计信息
    pub async fn get_stats(&self) -> NodeStats {
        self.stats.read().await.clone()
    }

    /// 启动任务轮询循环
    fn start_task_polling(&self) -> tokio::task::JoinHandle<()> {
        // 在函数开始时克隆所有需要的变量，避免循环中的所有权问题
        let consensus_client = self.consensus_client.clone();
        let task_executor = self.task_executor.clone();
        let active_tasks = self.active_tasks.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        let running = self.running.clone();
        let api_server = self.api_server.clone();

        // 保存节点ID用于整个函数
        let node_id = config.node_id.clone();
        let max_active_tasks = config.max_active_tasks;
        let cheating_mode = config.cheating_mode.clone();

        tokio::spawn(async move {
            log::info!("启动任务轮询循环 for Provider: {}", &node_id);

            while *running.read().await {
                // 检查是否可以接受新任务
                let active_count = active_tasks.read().await.len();
                if active_count >= max_active_tasks {
                    log::debug!("Provider {} 已达到最大活跃任务数: {}", &node_id, active_count);
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }

                // 轮询新任务
                if let Some(task) = consensus_client.poll_task().await {
                    log::info!("Provider {} 收到新任务: {}", &node_id, task.id);

                    // 创建任务执行 - 每次都重新克隆变量
                    let execution = TaskExecution::new(
                        task.clone(),
                        node_id.clone(), // 重新克隆
                        cheating_mode.clone(), // 重新克隆
                    );

                    // 添加到活跃任务
                    active_tasks.write().await.insert(task.id.clone(), execution.clone());

                    // 异步执行任务 - 为每个任务克隆所需变量
                    let active_tasks_clone = active_tasks.clone();
                    let stats_clone = stats.clone();
                    let consensus_client_clone = consensus_client.clone();
                    let api_server_clone = api_server.clone();
                    let task_executor_clone = task_executor.clone();
                    let config_clone = config.clone();
                    let task_clone = task.clone();
                    let cheating_mode_clone = cheating_mode.clone(); // 重新克隆
                    let node_id_clone = node_id.clone(); // 重新克隆

                    tokio::spawn(async move {
                        // 执行任务
                        let start_time = std::time::Instant::now();
                        let result = task_executor_clone.execute_task(execution.clone()).await;
                        let execution_time = start_time.elapsed();

                        // 记录性能
                        log_performance(
                            &format!("task_execution_{}", task_clone.id),
                            execution_time.as_millis() as u64,
                            result.success,
                        );

                        // 记录任务执行日志
                        log_task_execution(
                            &result.task_id,
                            &result.provider_id,
                            &execution.task.task_type.as_str(),
                            result.success,
                            result.execution_time_ms,
                        );

                        // 如果是作弊行为，记录安全事件
                        if cheating_mode_clone.is_some() {
                            log_security_event(
                                "cheating_detected",
                                &node_id_clone,
                                &format!("任务 {} 执行了作弊行为", task_clone.id),
                                crate::logging::SecuritySeverity::Medium,
                            );
                        }

                        // 提交结果到共识服务器
                        if let Err(e) = consensus_client_clone.submit_result(&task_clone.id, &result).await {
                            log_error_with_context(
                                &format!("提交任务 {} 结果失败", task_clone.id),
                                &*e,
                            );
                        }

                        // 更新统计信息
                        {
                            let mut stats = stats_clone.write().await;
                            stats.total_tasks_completed += 1;
                            if result.success {
                                stats.successful_tasks += 1;
                            } else {
                                stats.failed_tasks += 1;
                            }
                            stats.total_execution_time_ms += result.execution_time_ms;
                            stats.last_task_completed_at = Some(chrono::Utc::now());
                        }

                        // 更新 API 服务器状态
                        let status_info = ProviderStatusInfo {
                            provider_id: config_clone.node_id.clone(),
                            name: config_clone.name.clone(),
                            status: ProviderStatus::Running,
                            active_tasks: active_tasks_clone.read().await.len().saturating_sub(1),
                            total_completed_tasks: {
                                let stats = stats_clone.read().await;
                                stats.total_tasks_completed
                            },
                            cheating_stats: None,
                            gpu_memory_gb: config_clone.gpu_memory_gb,
                            cpu_cores: config_clone.cpu_cores,
                            last_updated: chrono::Utc::now(),
                        };

                        api_server_clone.update_provider_status(
                            config_clone.node_id.clone(),
                            status_info,
                        ).await;

                        api_server_clone.add_task_result(result).await;

                        // 从活跃任务中移除
                        active_tasks_clone.write().await.remove(&task_clone.id);
                    });
                } else {
                    // 没有新任务，等待一段时间
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }

            log::info!("任务轮询循环结束 for Provider: {}", node_id);
        })
    }

    /// 启动健康检查
    fn start_health_checks(&self) -> tokio::task::JoinHandle<()> {
        let consensus_client = self.consensus_client.clone();
        let _api_server = self.api_server.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            log::info!("启动健康检查 for Provider: {}", config.node_id);

            while *running.read().await {
                tokio::time::sleep(Duration::from_secs(30)).await;

                // 报告健康状态
                let health = HealthStatus {
                    overall_status: SystemStatus::Healthy,
                    cpu_usage_percent: 45.0, // 模拟值
                    memory_usage_percent: 60.0, // 模拟值
                    disk_usage_percent: 25.0, // 模拟值
                    network_connected: true,
                    active_tasks_count: 0, // 这里可以获取实际值
                    last_updated: chrono::Utc::now(),
                };

                if let Err(e) = consensus_client.report_health(&health).await {
                    log_error_with_context("健康检查报告失败", &*e);
                }

                // 更新 API 服务器的健康状态
                // 这里可以添加更多健康检查逻辑
            }

            log::info!("健康检查结束 for Provider: {}", config.node_id);
        })
    }

    /// 启动统计信息更新
    fn start_stats_updates(&self) -> tokio::task::JoinHandle<()> {
        let _stats = self.stats.clone();
        let consensus_client = self.consensus_client.clone();
        let api_server_clone = self.api_server.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            log::info!("启动统计更新 for Provider: {}", config.node_id);

            while *running.read().await {
                tokio::time::sleep(Duration::from_secs(60)).await;

                // 获取共识统计
                if let Ok(consensus_stats) = consensus_client.get_consensus_stats().await {
                    api_server_clone.update_consensus_stats(&consensus_stats).await;

                    // 记录共识事件
                    log_consensus_event(
                        "stats_update",
                        "system",
                        &format!("共识达成率: {:.2}%", consensus_stats.consensus_success_rate * 100.0),
                    );
                }

                // 清理过期结果
                consensus_client.cleanup_expired_results(3600).await; // 1小时
            }

            log::info!("统计更新结束 for Provider: {}", config.node_id);
        })
    }
}

/// 节点统计信息
#[derive(Debug, Clone)]
pub struct NodeStats {
    /// 节点ID
    pub node_id: String,
    /// 总完成任务数
    pub total_tasks_completed: usize,
    /// 成功任务数
    pub successful_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 最后任务完成时间
    pub last_task_completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 节点启动时间
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl NodeStats {
    /// 创建新的统计信息
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            total_tasks_completed: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            total_execution_time_ms: 0,
            last_task_completed_at: None,
            started_at: chrono::Utc::now(),
        }
    }

    /// 计算平均执行时间
    pub fn average_execution_time_ms(&self) -> f64 {
        if self.total_tasks_completed == 0 {
            0.0
        } else {
            self.total_execution_time_ms as f64 / self.total_tasks_completed as f64
        }
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks_completed == 0 {
            0.0
        } else {
            self.successful_tasks as f64 / self.total_tasks_completed as f64
        }
    }

    /// 计算运行时间
    pub fn uptime_seconds(&self) -> i64 {
        chrono::Utc::now().signed_duration_since(self.started_at).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;

    #[tokio::test]
    async fn test_node_stats() {
        let stats = NodeStats::new("test-provider");

        assert_eq!(stats.node_id, "test-provider");
        assert_eq!(stats.total_tasks_completed, 0);
        assert_eq!(stats.average_execution_time_ms(), 0.0);
        assert_eq!(stats.success_rate(), 0.0);
        assert!(stats.uptime_seconds() >= 0);
    }

    #[test]
    fn test_node_stats_calculations() {
        let mut stats = NodeStats::new("test-provider");
        stats.total_tasks_completed = 10;
        stats.successful_tasks = 8;
        stats.failed_tasks = 2;
        stats.total_execution_time_ms = 5000;

        assert_eq!(stats.average_execution_time_ms(), 500.0);
        assert_eq!(stats.success_rate(), 0.8);
    }
}

