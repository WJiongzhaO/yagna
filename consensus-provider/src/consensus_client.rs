//! # 共识客户端模块
//!
//! 这个模块负责与共识服务器通信，包括注册节点、接收任务、分发结果等功能。

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Error;
use crate::types::*;

/// 共识客户端
pub struct ConsensusClient {
    /// HTTP 客户端
    client: Client,
    /// 服务器端点
    server_endpoint: String,
    /// 本地 Provider ID
    provider_id: String,
    /// 待处理的任务队列
    pending_tasks: Arc<Mutex<HashMap<String, ConsensusTask>>>,
    /// 已完成的结果缓存
    completed_results: Arc<Mutex<HashMap<String, TaskResult>>>,
}

impl Clone for ConsensusClient {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(), // 创建新的客户端
            server_endpoint: self.server_endpoint.clone(),
            provider_id: self.provider_id.clone(),
            pending_tasks: Arc::new(Mutex::new(HashMap::new())), // 不克隆队列
            completed_results: Arc::new(Mutex::new(HashMap::new())), // 不克隆缓存
        }
    }
}

impl ConsensusClient {
    /// 创建新的共识客户端
    pub async fn new(server_endpoint: &str) -> Result<Self, Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            server_endpoint: server_endpoint.to_string(),
            provider_id: String::new(), // 将在注册时设置
            pending_tasks: Arc::new(Mutex::new(HashMap::new())),
            completed_results: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 注册 Provider 节点到共识网络
    pub async fn register_provider(&mut self, provider_id: &str) -> Result<(), Error> {
        self.provider_id = provider_id.to_string();

        let registration_data = ProviderRegistration {
            provider_id: provider_id.to_string(),
            capabilities: vec![
                "matrix_multiplication".to_string(),
                "vector_addition".to_string(),
                "simple_inference".to_string(),
            ],
            max_concurrent_tasks: 3,
            registered_at: chrono::Utc::now(),
        };

        let url = format!("{}/providers/register", self.server_endpoint);
        let response = self.client
            .post(&url)
            .json(&registration_data)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Provider {} 成功注册到共识网络", provider_id);
            Ok(())
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("注册失败: {}", error_msg))
        }
    }

    /// 轮询获取新任务
    pub async fn poll_task(&self) -> Option<ConsensusTask> {
        let url = format!("{}/tasks/poll/{}", self.server_endpoint, self.provider_id);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ConsensusTask>().await {
                        Ok(task) => {
                            log::info!("收到新任务: {} (类型: {})", task.id, task.task_type.as_str());

                            // 将任务添加到待处理队列
                            let mut pending = self.pending_tasks.lock().await;
                            pending.insert(task.id.clone(), task.clone());

                            Some(task)
                        }
                        Err(e) => {
                            log::warn!("解析任务数据失败: {}", e);
                            None
                        }
                    }
                } else if response.status() == reqwest::StatusCode::NO_CONTENT {
                    // 没有新任务
                    None
                } else {
                    log::warn!("获取任务失败: HTTP {}", response.status());
                    None
                }
            }
            Err(e) => {
                log::warn!("网络请求失败: {}", e);
                None
            }
        }
    }

    /// 提交任务执行结果
    pub async fn submit_result(&self, task_id: &str, result: &TaskResult) -> Result<(), Error> {
        let url = format!("{}/results/submit", self.server_endpoint);

        let submission = ResultSubmission {
            task_id: task_id.to_string(),
            provider_id: self.provider_id.clone(),
            result: result.clone(),
            submitted_at: chrono::Utc::now(),
        };

        let response = self.client
            .post(&url)
            .json(&submission)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("任务 {} 结果提交成功", task_id);

            // 从待处理队列移除任务
            let mut pending = self.pending_tasks.lock().await;
            pending.remove(task_id);

            // 添加到已完成结果缓存
            let mut completed = self.completed_results.lock().await;
            completed.insert(task_id.to_string(), result.clone());

            Ok(())
        } else {
            let error_msg = response.text().await?;
            log::error!("提交结果失败: {}", error_msg);
            Err(anyhow::anyhow!("提交结果失败: {}", error_msg))
        }
    }

    /// 获取当前节点状态
    pub async fn get_provider_status(&self) -> Result<ProviderStatusResponse, Error> {
        let url = format!("{}/providers/{}/status", self.server_endpoint, self.provider_id);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let status: ProviderStatusResponse = response.json().await?;
            Ok(status)
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("获取状态失败: {}", error_msg))
        }
    }

    /// 获取奖励/惩罚历史
    pub async fn get_incentive_history(&self, limit: usize) -> Result<Vec<IncentiveEvent>, Error> {
        let url = format!("{}/providers/{}/incentives?limit={}", self.server_endpoint, self.provider_id, limit);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let history: Vec<IncentiveEvent> = response.json().await?;
            Ok(history)
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("获取激励历史失败: {}", error_msg))
        }
    }

    /// 报告节点健康状态
    pub async fn report_health(&self, health: &HealthStatus) -> Result<(), Error> {
        let url = format!("{}/providers/{}/health", self.server_endpoint, self.provider_id);

        let response = self.client
            .post(&url)
            .json(health)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("报告健康状态失败: {}", error_msg))
        }
    }

    /// 获取共识统计信息
    pub async fn get_consensus_stats(&self) -> Result<ConsensusStats, Error> {
        let url = format!("{}/stats/consensus", self.server_endpoint);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let stats: ConsensusStats = response.json().await?;
            Ok(stats)
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("获取共识统计失败: {}", error_msg))
        }
    }

    /// 获取本地待处理任务数量
    pub async fn get_pending_task_count(&self) -> usize {
        let pending = self.pending_tasks.lock().await;
        pending.len()
    }

    /// 获取本地已完成结果数量
    pub async fn get_completed_result_count(&self) -> usize {
        let completed = self.completed_results.lock().await;
        completed.len()
    }

    /// 清理过期结果（避免内存泄漏）
    pub async fn cleanup_expired_results(&self, max_age_seconds: u64) {
        let mut completed = self.completed_results.lock().await;
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::seconds(max_age_seconds as i64);

        let initial_count = completed.len();
        completed.retain(|_, result| result.completed_at > cutoff);
        let expired_count = initial_count - completed.len();

        if expired_count > 0 {
            log::info!("清理了 {} 个过期结果", expired_count);
        }
    }
}

/// Provider 注册数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistration {
    /// Provider 节点ID
    pub provider_id: String,
    /// 支持的任务类型
    pub capabilities: Vec<String>,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// 结果提交数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSubmission {
    /// 任务ID
    pub task_id: String,
    /// Provider ID
    pub provider_id: String,
    /// 执行结果
    pub result: TaskResult,
    /// 提交时间
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

/// Provider 状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatusResponse {
    /// Provider ID
    pub provider_id: String,
    /// 当前状态
    pub status: ProviderStatus,
    /// 活跃任务数量
    pub active_tasks: usize,
    /// 总完成任务数
    pub total_completed_tasks: usize,
    /// 信誉分数
    pub reputation_score: f64,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 共识统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStats {
    /// 总 Provider 节点数
    pub total_providers: usize,
    /// 活跃 Provider 节点数
    pub active_providers: usize,
    /// 总任务数
    pub total_tasks: usize,
    /// 已完成任务数
    pub completed_tasks: usize,
    /// 共识达成率
    pub consensus_success_rate: f64,
    /// 平均任务执行时间（毫秒）
    pub avg_task_execution_time_ms: f64,
    /// 检测到的作弊事件数
    pub detected_cheating_events: usize,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ConsensusStats {
    /// 获取共识成功率的百分比表示
    pub fn consensus_success_rate_percent(&self) -> f64 {
        self.consensus_success_rate * 100.0
    }

    /// 计算完成率
    pub fn completion_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed_tasks as f64 / self.total_tasks as f64
        }
    }
}

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 服务器端点
    pub server_endpoint: String,
    /// 请求超时时间（秒）
    pub request_timeout_seconds: u64,
    /// 重试次数
    pub max_retries: usize,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 健康检查间隔（秒）
    pub health_check_interval_seconds: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_endpoint: "http://localhost:3000".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            retry_interval_ms: 1000,
            health_check_interval_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // 注意：这些测试需要实际的共识服务器运行
    // 在实际测试环境中，需要启动 mock 服务器

    #[tokio::test]
    async fn test_client_creation() {
        let client = ConsensusClient::new("http://localhost:3000").await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_consensus_stats() {
        let stats = ConsensusStats {
            total_providers: 10,
            active_providers: 8,
            total_tasks: 1000,
            completed_tasks: 950,
            consensus_success_rate: 0.92,
            avg_task_execution_time_ms: 2450.0,
            detected_cheating_events: 5,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(stats.consensus_success_rate_percent(), 92.0);
        assert_eq!(stats.completion_rate(), 0.95);
    }

    #[test]
    fn test_provider_registration_data() {
        let registration = ProviderRegistration {
            provider_id: "test-provider".to_string(),
            capabilities: vec!["matrix_multiplication".to_string()],
            max_concurrent_tasks: 5,
            registered_at: chrono::Utc::now(),
        };

        assert_eq!(registration.provider_id, "test-provider");
        assert_eq!(registration.capabilities.len(), 1);
        assert_eq!(registration.max_concurrent_tasks, 5);
    }
}

