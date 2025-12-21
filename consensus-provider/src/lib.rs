//! # Consensus Provider Simulator
//!
//! 这个模块实现了去中心化 GPU 算力共享平台中的 Provider 节点模拟器，
//! 专门用于演示多节点算力任务验证共识机制。

pub mod types;
pub mod config;
pub mod cheating_modes;
pub mod task_executor;
pub mod consensus_client;
pub mod provider_node;
pub mod api_server;
pub mod logging;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取版本信息
pub fn version() -> &'static str {
    VERSION
}

// 重新导出所有类型
pub use types::*;
pub use config::*;