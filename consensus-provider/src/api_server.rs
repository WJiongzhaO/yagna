//! # API 服务器模块
//!
//! 这个模块提供 REST API 和 WebSocket 接口，用于监控 Provider 节点状态、
//! 查看任务执行情况和管理节点配置。

use warp::Filter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures_util::{stream::StreamExt, SinkExt};
use crate::types::*;
use crate::consensus_client::ConsensusStats;
use crate::cheating_modes::CheatingStatistics;

/// API 服务器
#[derive(Clone)]
pub struct ApiServer {
    /// 服务器端口
    port: u16,
    /// 服务器状态
    server_state: Arc<RwLock<ServerState>>,
}

impl ApiServer {
    /// 创建新的 API 服务器
    pub fn new(port: u16) -> Self {
        Self {
            port,
            server_state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    /// 启动 API 服务器
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("启动 API 服务器在端口 {}", self.port);

        // 创建路由
        let routes = self.create_routes();

        // 启动服务器
        warp::serve(routes)
            .run(([127, 0, 0, 1], self.port))
            .await;

        Ok(())
    }

    /// 创建所有 API 路由
    fn create_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let state = self.server_state.clone();

        // 健康检查端点
        let health = warp::path("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(health_handler);

        // 获取所有 Provider 状态
        let providers = warp::path("providers")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(get_providers_handler);

        // 获取特定 Provider 信息
        let provider = warp::path!("providers" / String)
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(get_provider_handler);

        // 获取 Provider 任务历史
        let provider_tasks = warp::path!("providers" / String / "tasks")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(get_provider_tasks_handler);

        // 获取系统统计信息
        let stats = warp::path("stats")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(get_stats_handler);

        // 获取共识统计信息
        let consensus_stats = warp::path!("consensus" / "stats")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(get_consensus_stats_handler);

        // 更新 Provider 配置
        let update_provider = warp::path!("providers" / String)
            .and(warp::put())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(update_provider_handler);

        // 重置统计信息
        let reset_stats = warp::path("stats")
            .and(warp::delete())
            .and(with_state(state.clone()))
            .and_then(reset_stats_handler);

        // WebSocket 端点用于实时监控
        let ws = warp::path("ws")
            .and(warp::ws())
            .and(with_state(state))
            .map(|ws: warp::ws::Ws, state| {
                ws.on_upgrade(move |websocket| handle_websocket(websocket, state))
            });

        // CORS 支持
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

        // 组合所有路由
        (health
            .or(providers)
            .or(provider)
            .or(provider_tasks)
            .or(stats)
            .or(consensus_stats)
            .or(update_provider)
            .or(reset_stats)
            .or(ws))
        .with(cors)
        .with(warp::log("api"))
    }

    /// 更新服务器状态
    pub async fn update_provider_status(&self, provider_id: String, status: ProviderStatusInfo) {
        let mut state = self.server_state.write().await;
        state.providers.insert(provider_id, status);
    }

    /// 添加任务结果
    pub async fn add_task_result(&self, result: TaskResult) {
        let mut state = self.server_state.write().await;
        state.task_history.push(result);
        // 保持历史记录不超过 1000 条
        if state.task_history.len() > 1000 {
            state.task_history.remove(0);
        }
    }

    /// 更新共识统计
    pub async fn update_consensus_stats(&self, stats: &ConsensusStats) {
        let mut state = self.server_state.write().await;
        state.consensus_stats = Some(stats.clone());
    }
}

/// 服务器状态
#[derive(Debug, Clone, Default)]
struct ServerState {
    /// 所有 Provider 的状态信息
    providers: HashMap<String, ProviderStatusInfo>,
    /// 任务执行历史
    task_history: Vec<TaskResult>,
    /// 共识统计信息
    consensus_stats: Option<ConsensusStats>,
    /// 服务器启动时间
    start_time: chrono::DateTime<chrono::Utc>,
}

/// Provider 状态信息（API 响应用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatusInfo {
    /// Provider ID
    pub provider_id: String,
    /// 显示名称
    pub name: String,
    /// 当前状态
    pub status: ProviderStatus,
    /// 活跃任务数量
    pub active_tasks: usize,
    /// 总完成任务数
    pub total_completed_tasks: usize,
    /// 作弊统计
    pub cheating_stats: Option<CheatingStatistics>,
    /// GPU 内存总量（GB）
    pub gpu_memory_gb: f64,
    /// CPU 核心数
    pub cpu_cores: u32,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ProviderStatusInfo {
    /// 创建新的 Provider 状态信息
    pub fn new(provider_id: &str, name: &str, gpu_memory_gb: f64, cpu_cores: u32) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            name: name.to_string(),
            status: ProviderStatus::Starting,
            active_tasks: 0,
            total_completed_tasks: 0,
            cheating_stats: None,
            gpu_memory_gb,
            cpu_cores,
            last_updated: chrono::Utc::now(),
        }
    }

    /// 计算任务成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_completed_tasks == 0 {
            0.0
        } else {
            // 这里需要实际的任务成功统计，暂时返回估算值
            0.95 // 假设 95% 成功率
        }
    }
}

/// API 响应包装器
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Provider 更新请求
#[derive(Debug, Deserialize)]
struct UpdateProviderRequest {
    name: Option<String>,
    status: Option<ProviderStatus>,
}

/// 过滤器辅助函数
fn with_state(state: Arc<RwLock<ServerState>>) -> impl Filter<Extract = (Arc<RwLock<ServerState>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// 健康检查处理器
async fn health_handler(state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = chrono::Utc::now().signed_duration_since(state.start_time).num_seconds();

    let health = serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime,
        "providers_count": state.providers.len(),
        "total_tasks": state.task_history.len(),
        "timestamp": chrono::Utc::now()
    });

    Ok(warp::reply::json(&health))
}

/// 获取所有 Provider 处理器
async fn get_providers_handler(state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;
    let providers: Vec<&ProviderStatusInfo> = state.providers.values().collect();

    let response = ApiResponse::success(providers);
    Ok(warp::reply::json(&response))
}

/// 获取特定 Provider 处理器
async fn get_provider_handler(provider_id: String, state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;

    match state.providers.get(&provider_id) {
        Some(provider) => {
            let response = ApiResponse::success(provider);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        None => {
            let response = ApiResponse::<()>::error(format!("Provider {} not found", provider_id));
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
    }
}

/// 获取 Provider 任务历史处理器
async fn get_provider_tasks_handler(provider_id: String, state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;

    let tasks: Vec<&TaskResult> = state.task_history
        .iter()
        .filter(|task| task.provider_id == provider_id)
        .collect();

    let response = ApiResponse::success(tasks);
    Ok(warp::reply::json(&response))
}

/// 获取系统统计处理器
async fn get_stats_handler(state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;

    let stats = serde_json::json!({
        "total_providers": state.providers.len(),
        "total_tasks": state.task_history.len(),
        "successful_tasks": state.task_history.iter().filter(|t| t.success).count(),
        "failed_tasks": state.task_history.iter().filter(|t| !t.success).count(),
        "avg_execution_time_ms": calculate_avg_execution_time(&state.task_history),
        "providers_by_status": count_providers_by_status(&state.providers),
        "uptime_seconds": chrono::Utc::now().signed_duration_since(state.start_time).num_seconds()
    });

    let response = ApiResponse::success(stats);
    Ok(warp::reply::json(&response))
}

/// 获取共识统计处理器
async fn get_consensus_stats_handler(state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().await;

    match &state.consensus_stats {
        Some(stats) => {
            let response = ApiResponse::success(stats);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        None => {
            let response = ApiResponse::<()>::error("Consensus stats not available".to_string());
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
    }
}

/// 更新 Provider 处理器
async fn update_provider_handler(
    provider_id: String,
    request: UpdateProviderRequest,
    state: Arc<RwLock<ServerState>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state = state.write().await;

    if let Some(provider) = state.providers.get_mut(&provider_id) {
        if let Some(name) = request.name {
            provider.name = name;
        }
        if let Some(status) = request.status {
            provider.status = status;
        }
        provider.last_updated = chrono::Utc::now();

        let response = ApiResponse::success(provider);
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    } else {
        let response = ApiResponse::<()>::error(format!("Provider {} not found", provider_id));
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

/// 重置统计信息处理器
async fn reset_stats_handler(state: Arc<RwLock<ServerState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state = state.write().await;

    state.task_history.clear();
    state.consensus_stats = None;

    for provider in state.providers.values_mut() {
        provider.active_tasks = 0;
        provider.total_completed_tasks = 0;
        provider.cheating_stats = None;
        provider.last_updated = chrono::Utc::now();
    }

    let response = ApiResponse::success(serde_json::json!({"message": "Statistics reset successfully"}));
    Ok(warp::reply::json(&response))
}

/// WebSocket 连接处理器
async fn handle_websocket(websocket: warp::ws::WebSocket, state: Arc<RwLock<ServerState>>) {

    log::info!("新的 WebSocket 连接建立");

    let (mut ws_tx, mut ws_rx) = websocket.split();

    // 发送初始状态
    let initial_state = {
        let state = state.read().await;
        serde_json::json!({
            "type": "initial_state",
            "providers": state.providers,
            "total_tasks": state.task_history.len(),
            "timestamp": chrono::Utc::now()
        })
    };

    if let Ok(message) = serde_json::to_string(&initial_state) {
        let _ = ws_tx.send(warp::ws::Message::text(message)).await;
    }

    // 监听状态变化并推送更新（这里是简化实现）
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_close() {
                    break;
                }
                // 处理客户端消息（如果需要）
                log::debug!("收到 WebSocket 消息: {:?}", msg);
            }
            Err(e) => {
                log::error!("WebSocket 错误: {}", e);
                break;
            }
        }
    }

    log::info!("WebSocket 连接关闭");
}

/// 辅助函数：计算平均执行时间
fn calculate_avg_execution_time(tasks: &[TaskResult]) -> f64 {
    if tasks.is_empty() {
        0.0
    } else {
        let total: u64 = tasks.iter().map(|t| t.execution_time_ms).sum();
        total as f64 / tasks.len() as f64
    }
}

/// 辅助函数：按状态统计 Provider
fn count_providers_by_status(providers: &HashMap<String, ProviderStatusInfo>) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for provider in providers.values() {
        let status_str = provider.status.as_str();
        *counts.entry(status_str.to_string()).or_insert(0) += 1;
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_provider_status_info() {
        let info = ProviderStatusInfo::new("test-provider", "Test Provider", 8.0, 4);

        assert_eq!(info.provider_id, "test-provider");
        assert_eq!(info.name, "Test Provider");
        assert_eq!(info.gpu_memory_gb, 8.0);
        assert_eq!(info.cpu_cores, 4);
        assert_eq!(info.active_tasks, 0);
        assert!(matches!(info.status, ProviderStatus::Starting));
    }

    #[test]
    fn test_api_response() {
        let success_response: ApiResponse<String> = ApiResponse::success("test data".to_string());
        assert!(success_response.success);
        assert_eq!(success_response.data.unwrap(), "test data");
        assert!(success_response.error.is_none());

        let error_response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert_eq!(error_response.error.unwrap(), "test error");
    }

    #[test]
    fn test_calculate_avg_execution_time() {
        let tasks = vec![
            TaskResult::success("task1".to_string(), "provider1".to_string(), vec![], 100, Default::default()),
            TaskResult::success("task2".to_string(), "provider1".to_string(), vec![], 200, Default::default()),
            TaskResult::success("task3".to_string(), "provider1".to_string(), vec![], 300, Default::default()),
        ];

        let avg = calculate_avg_execution_time(&tasks);
        assert_eq!(avg, 200.0);

        let empty_tasks: Vec<TaskResult> = vec![];
        let avg_empty = calculate_avg_execution_time(&empty_tasks);
        assert_eq!(avg_empty, 0.0);
    }
}

