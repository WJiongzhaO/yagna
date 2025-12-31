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

// Yagna Market API 数据结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketOffer {
    pub properties: serde_json::Value,
    pub constraints: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agreement {
    pub agreement_id: String,
    pub demand: serde_json::Value,
    pub offer: serde_json::Value,
    pub state: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activity {
    pub activity_id: String,
    pub agreement_id: String,
}

/// 共识客户端 - 集成到 Yagna Market 系统
pub struct ConsensusClient {
    /// HTTP 客户端
    client: Client,
    /// yagna API 端点
    server_endpoint: String,
    /// 本地 Provider ID
    provider_id: String,
    /// 市场订阅 ID
    subscription_id: Option<String>,
    /// 活跃的协议列表 (agreement_id -> activity_id)
    active_agreements: Arc<Mutex<HashMap<String, String>>>,
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
            subscription_id: self.subscription_id.clone(),
            active_agreements: Arc::new(Mutex::new(HashMap::new())), // 不克隆协议
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
            subscription_id: None,
            active_agreements: Arc::new(Mutex::new(HashMap::new())),
            pending_tasks: Arc::new(Mutex::new(HashMap::new())),
            completed_results: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 发布 Market Offer - 替代简单的provider注册
    pub async fn register_provider(&mut self, provider_id: &str) -> Result<(), Error> {
        self.provider_id = provider_id.to_string();

        // 创建market offer
        let offer = self.build_market_offer(provider_id)?;

        // 发布到yagna market
        let url = format!("{}/market/offers", self.server_endpoint);
        let response = self.client
            .post(&url)
            .json(&offer)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription_result: serde_json::Value = response.json().await?;
            if let Some(subscription_id) = subscription_result.get("subscriptionId") {
                self.subscription_id = Some(subscription_id.as_str().unwrap_or("").to_string());
            }
            log::info!("Provider {} 成功发布market offer", provider_id);
            Ok(())
        } else {
            let error_msg = response.text().await?;
            Err(anyhow::anyhow!("发布market offer失败: {}", error_msg))
        }
    }

    /// 构建market offer
    fn build_market_offer(&self, provider_id: &str) -> Result<MarketOffer, Error> {
        let properties = serde_json::json!({
            "golem.node.id.name": provider_id,
            "golem.srv.comp.task_package": format!("hash:sha3:{}", provider_id),
            "golem.srv.comp.expiration": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
            "golem.srv.caps.multi-activity": true,
            "golem.inf.cpu.cores.available": 4,
            "golem.inf.mem.gib": 8.0,
            "golem.inf.storage.gib": 100.0,
            "golem.runtime.name": "consensus-provider",
            "golem.runtime.version": "0.1.0"
        });

        Ok(MarketOffer {
            properties,
            constraints: r#"(&
                (golem.inf.mem.gib>0.5)
                (golem.inf.storage.gib>1.0)
                (golem.inf.cpu.cores.available>0)
            )"#.to_string(),
        })
    }

    /// 轮询协议和任务 - 基于yagna market协议
    pub async fn poll_task(&self) -> Option<ConsensusTask> {
        // 首先检查是否有新的协议
        if let Some(agreement) = self.poll_agreements().await {
            // 为协议创建activity
            if let Ok(activity_id) = self.create_activity(&agreement.agreement_id).await {
                // 从协议中提取任务信息
                if let Some(task) = self.extract_task_from_agreement(&agreement, &activity_id).await {
                    let mut pending = self.pending_tasks.lock().await;
                    pending.insert(task.id.clone(), task.clone());

                    let mut agreements = self.active_agreements.lock().await;
                    agreements.insert(agreement.agreement_id, activity_id);

                    return Some(task);
                }
            }
        }

        None
    }

    /// 轮询新的协议
    async fn poll_agreements(&self) -> Option<Agreement> {
        let url = format!("{}/market/agreements?state=Pending", self.server_endpoint);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(agreements) = response.json::<Vec<Agreement>>().await {
                        for agreement in agreements {
                            // 自动批准协议
                            if let Ok(_) = self.approve_agreement(&agreement.agreement_id).await {
                                log::info!("批准协议: {}", agreement.agreement_id);
                                return Some(agreement);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("轮询协议失败: {}", e);
            }
        }
        None
    }

    /// 批准协议
    async fn approve_agreement(&self, agreement_id: &str) -> Result<(), Error> {
        let url = format!("{}/market/agreements/{}/approve", self.server_endpoint, agreement_id);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("批准协议失败"))
        }
    }

    /// 为协议创建activity
    async fn create_activity(&self, agreement_id: &str) -> Result<String, Error> {
        let url = format!("{}/activity/agreements/{}", self.server_endpoint, agreement_id);
        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            let activity: Activity = response.json().await?;
            log::info!("创建activity: {}", activity.activity_id);
            Ok(activity.activity_id)
        } else {
            Err(anyhow::anyhow!("创建activity失败"))
        }
    }

    /// 从协议中提取任务信息
    async fn extract_task_from_agreement(&self, agreement: &Agreement, _activity_id: &str) -> Option<ConsensusTask> {
        // 从协议的demand中提取任务参数
        if let Some(task_type) = agreement.demand.get("task_type") {
            let task_type_str = task_type.as_str().unwrap_or("matrix_multiplication");

            let task_type_enum = match task_type_str {
                "matrix_multiplication" => {
                    let size = agreement.demand.get("size").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
                    TaskType::MatrixMultiplication { size }
                }
                "vector_addition" => {
                    let size = agreement.demand.get("size").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                    TaskType::VectorAddition { size }
                }
                "simple_inference" => {
                    let size = agreement.demand.get("model_size").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                    TaskType::SimpleInference { model_size: size }
                }
                _ => TaskType::MatrixMultiplication { size: 2 }
            };

            // 生成任务数据（这里使用简化版本，实际应该从协议参数中解析）
            let data = match &task_type_enum {
                TaskType::MatrixMultiplication { size: _ } => {
                    vec![1.0f64.to_le_bytes(), 2.0f64.to_le_bytes(), 3.0f64.to_le_bytes(), 4.0f64.to_le_bytes(),
                         5.0f64.to_le_bytes(), 6.0f64.to_le_bytes(), 7.0f64.to_le_bytes(), 8.0f64.to_le_bytes()].concat()
                }
                TaskType::VectorAddition { size: _ } => {
                    vec![1.0f64.to_le_bytes(), 2.0f64.to_le_bytes(), 3.0f64.to_le_bytes(),
                         4.0f64.to_le_bytes(), 5.0f64.to_le_bytes(), 6.0f64.to_le_bytes()].concat()
                }
                TaskType::SimpleInference { model_size: _ } => {
                    (0..50).flat_map(|_| 1.0f64.to_le_bytes()).collect::<Vec<u8>>() // 简化的输入数据
                }
            };

            let task = ConsensusTask::new(
                task_type_enum,
                data,
                300, // 5分钟超时
            );

            log::info!("从协议 {} 提取任务: {} (类型: {})", agreement.agreement_id, task.id, task.task_type.as_str());
            Some(task)
        } else {
            None
        }
    }

    /// 提交任务执行结果 - 通过activity API
    pub async fn submit_result(&self, task_id: &str, result: &TaskResult) -> Result<(), Error> {
        // 查找对应的activity
        let agreements = self.active_agreements.lock().await;
        if let Some(activity_id) = agreements.get(task_id) {
            // 提交结果到activity
            let url = format!("{}/activity/{}/results", self.server_endpoint, activity_id);

            #[derive(serde::Serialize)]
            struct ActivityResult {
                pub result: Vec<u8>,
                pub stdout: Option<String>,
                pub stderr: Option<String>,
            }

            let activity_result = ActivityResult {
                result: result.result.clone(),
                stdout: Some(format!("任务执行成功，耗时: {}ms", result.execution_time_ms)),
                stderr: if result.success {
                    None
                } else {
                    Some(result.error_message.clone().unwrap_or_default())
                },
            };

            let response = self.client
                .post(&url)
                .json(&activity_result)
                .send()
                .await?;

            if response.status().is_success() {
                log::info!("任务 {} 结果提交成功 (activity: {})", task_id, activity_id);

                // 清理资源
                let mut pending = self.pending_tasks.lock().await;
                pending.remove(task_id);
                let mut completed = self.completed_results.lock().await;
                completed.insert(task_id.to_string(), result.clone());

                // 销毁activity
                let _ = self.destroy_activity(activity_id).await;

                Ok(())
            } else {
                let error_msg = response.text().await?;
                log::error!("提交activity结果失败: {}", error_msg);
                Err(anyhow::anyhow!("提交activity结果失败: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("找不到任务对应的activity"))
        }
    }

    /// 销毁activity
    async fn destroy_activity(&self, activity_id: &str) -> Result<(), Error> {
        let url = format!("{}/activity/{}", self.server_endpoint, activity_id);
        let response = self.client.delete(&url).send().await?;

        if response.status().is_success() {
            log::info!("销毁activity: {}", activity_id);
            Ok(())
        } else {
            log::error!("销毁activity失败");
            Err(anyhow::anyhow!("销毁activity失败"))
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
            server_endpoint: "http://localhost:7465".to_string(),
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
        let client = ConsensusClient::new("http://localhost:7465").await;
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

