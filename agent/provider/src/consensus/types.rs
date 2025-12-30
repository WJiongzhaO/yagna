//! # 类型定义
//!
//! 这个模块定义了整个 consensus-provider 模块中使用的数据类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 任务类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    /// 矩阵乘法任务
    #[serde(rename = "matrix_multiplication")]
    MatrixMultiplication {
        /// 矩阵大小 (N x N)
        size: usize
    },

    /// 向量加法任务
    #[serde(rename = "vector_addition")]
    VectorAddition {
        /// 向量长度
        size: usize
    },

    /// 简单神经网络推理任务
    #[serde(rename = "simple_inference")]
    SimpleInference {
        /// 模型大小（隐藏层神经元数量）
        model_size: usize
    },
}

impl TaskType {
    /// 获取任务类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::MatrixMultiplication { .. } => "matrix_multiplication",
            TaskType::VectorAddition { .. } => "vector_addition",
            TaskType::SimpleInference { .. } => "simple_inference",
        }
    }

    /// 估算任务的计算复杂度（用于调度决策）
    pub fn estimated_complexity(&self) -> u64 {
        match self {
            TaskType::MatrixMultiplication { size } => (size * size * size) as u64, // O(n^3)
            TaskType::VectorAddition { size } => *size as u64, // O(n)
            TaskType::SimpleInference { model_size } => (*model_size * *model_size) as u64, // O(n^2)
        }
    }
}

/// 共识任务结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusTask {
    /// 任务唯一标识符
    pub id: String,

    /// 任务类型
    pub task_type: TaskType,

    /// 任务数据（序列化的输入参数）
    pub data: Vec<u8>,

    /// 任务创建时间
    pub created_at: DateTime<Utc>,

    /// 任务超时时间（秒）
    pub timeout_seconds: u64,

    /// 任务优先级 (0-10, 越高优先级越高)
    pub priority: u8,

    /// 任务元数据
    pub metadata: HashMap<String, String>,
}

impl ConsensusTask {
    /// 创建新任务
    pub fn new(task_type: TaskType, data: Vec<u8>, timeout_seconds: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type,
            data,
            created_at: Utc::now(),
            timeout_seconds,
            priority: 5, // 默认中等优先级
            metadata: HashMap::new(),
        }
    }

    /// 检查任务是否已过期
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.created_at);
        elapsed.num_seconds() as u64 >= self.timeout_seconds
    }

    /// 获取任务剩余时间（秒）
    pub fn remaining_seconds(&self) -> i64 {
        let elapsed = Utc::now().signed_duration_since(self.created_at);
        let remaining = self.timeout_seconds as i64 - elapsed.num_seconds();
        remaining.max(0)
    }
}

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 任务ID
    pub task_id: String,

    /// Provider 节点ID
    pub provider_id: String,

    /// 执行结果数据
    pub result: Vec<u8>,

    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,

    /// 是否执行成功
    pub success: bool,

    /// 错误信息（如果有）
    pub error_message: Option<String>,

    /// 执行完成时间
    pub completed_at: DateTime<Utc>,

    /// 资源使用统计
    pub resource_usage: ResourceUsage,
}

impl TaskResult {
    /// 创建成功的结果
    pub fn success(task_id: String, provider_id: String, result: Vec<u8>, execution_time_ms: u64, resource_usage: ResourceUsage) -> Self {
        Self {
            task_id,
            provider_id,
            result,
            execution_time_ms,
            success: true,
            error_message: None,
            completed_at: Utc::now(),
            resource_usage,
        }
    }

    /// 创建失败的结果
    pub fn failure(task_id: String, provider_id: String, error_message: String, execution_time_ms: u64, resource_usage: ResourceUsage) -> Self {
        Self {
            task_id,
            provider_id,
            result: vec![],
            execution_time_ms,
            success: false,
            error_message: Some(error_message),
            completed_at: Utc::now(),
            resource_usage,
        }
    }
}

/// 资源使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU 使用率（百分比）
    pub cpu_usage_percent: f64,

    /// 内存使用量（MB）
    pub memory_usage_mb: f64,

    /// GPU 内存使用量（MB）
    pub gpu_memory_usage_mb: Option<f64>,

    /// 网络使用量（KB）
    pub network_usage_kb: f64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            gpu_memory_usage_mb: None,
            network_usage_kb: 0.0,
        }
    }
}

/// 共识验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusResult {
    /// 共识达成
    #[serde(rename = "agreed")]
    Agreed {
        /// 共识结果哈希
        result_hash: String,
        /// 参与验证的节点数量
        participant_count: usize,
        /// 共识达成时间
        agreed_at: DateTime<Utc>,
    },

    /// 共识失败 - 分歧
    #[serde(rename = "disputed")]
    Disputed {
        /// 有分歧的 Provider 节点ID列表
        disputed_providers: Vec<String>,
        /// 不同结果的数量
        different_results_count: usize,
        /// 检测到分歧的时间
        disputed_at: DateTime<Utc>,
    },

    /// 共识失败 - 超时
    #[serde(rename = "timeout")]
    Timeout {
        /// 参与的节点数量
        participant_count: usize,
        /// 超时时间
        timeout_at: DateTime<Utc>,
    },
}

/// 奖励/惩罚事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncentiveEvent {
    /// 奖励事件
    #[serde(rename = "reward")]
    Reward {
        /// 受益的 Provider ID
        provider_id: String,
        /// 奖励金额
        amount: f64,
        /// 奖励原因
        reason: String,
        /// 奖励时间
        rewarded_at: DateTime<Utc>,
    },

    /// 惩罚事件
    #[serde(rename = "penalty")]
    Penalty {
        /// 被惩罚的 Provider ID
        provider_id: String,
        /// 惩罚金额
        amount: f64,
        /// 惩罚原因
        reason: String,
        /// 惩罚时间
        penalized_at: DateTime<Utc>,
    },
}

/// Provider 节点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderStatus {
    /// 节点正在启动
    #[serde(rename = "starting")]
    Starting,

    /// 节点正常运行
    #[serde(rename = "running")]
    Running,

    /// 节点忙碌（执行任务中）
    #[serde(rename = "busy")]
    Busy,

    /// 节点暂停
    #[serde(rename = "paused")]
    Paused,

    /// 节点错误
    #[serde(rename = "error")]
    Error { message: String },

    /// 节点已停止
    #[serde(rename = "stopped")]
    Stopped,
}

impl ProviderStatus {
    /// 检查节点是否可以接受新任务
    pub fn can_accept_tasks(&self) -> bool {
        matches!(self, ProviderStatus::Running)
    }

    /// 获取状态的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderStatus::Starting => "starting",
            ProviderStatus::Running => "running",
            ProviderStatus::Busy => "busy",
            ProviderStatus::Paused => "paused",
            ProviderStatus::Error { .. } => "error",
            ProviderStatus::Stopped => "stopped",
        }
    }
}

/// 系统健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 整体状态
    pub overall_status: SystemStatus,

    /// CPU 使用率
    pub cpu_usage_percent: f64,

    /// 内存使用率
    pub memory_usage_percent: f64,

    /// 磁盘使用率
    pub disk_usage_percent: f64,

    /// 网络连接状态
    pub network_connected: bool,

    /// 当前活跃任务数量
    pub active_tasks_count: usize,

    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 系统状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SystemStatus {
    /// 健康
    #[serde(rename = "healthy")]
    Healthy,

    /// 警告
    #[serde(rename = "warning")]
    Warning,

    /// 错误
    #[serde(rename = "error")]
    Error,

    /// 严重错误
    #[serde(rename = "critical")]
    Critical,
}

impl SystemStatus {
    /// 获取状态的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemStatus::Healthy => "healthy",
            SystemStatus::Warning => "warning",
            SystemStatus::Error => "error",
            SystemStatus::Critical => "critical",
        }
    }
}

/// 共识统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsensusStats {
    /// 总 Provider 数量
    pub total_providers: usize,
    /// 活跃任务数量
    pub active_tasks: usize,
    /// 已完成任务数量
    pub completed_tasks: usize,
    /// 平均共识达成时间（秒）
    pub average_consensus_time: f64,
    /// 共识成功率（0.0-1.0）
    pub consensus_success_rate: f64,
}

