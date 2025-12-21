//! # 配置模块
//!
//! 这个模块负责加载和解析配置文件，支持 JSON 格式的配置。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::cheating_modes::{CheatingMode, CheatingWeights, SmartCheatingConfig};

/// 应用配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// 共识相关配置
    pub consensus: ConsensusConfig,
    /// Provider 节点配置列表
    pub providers: Vec<ProviderConfig>,
    /// 任务类型配置
    pub task_types: TaskTypesConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 创建默认配置
    pub fn default_config() -> Self {
        Self {
            consensus: ConsensusConfig::default(),
            providers: vec![
                ProviderConfig::new_honest("provider-1", "Honest Provider", 8.0, 4),
                ProviderConfig::new_cheater("provider-2", "Dishonest Provider (Wrong Result)", 8.0, 4),
                ProviderConfig::new_delayer("provider-3", "Dishonest Provider (Delay)", 8.0, 4),
                ProviderConfig::new_terminator("provider-4", "Dishonest Provider (Early Termination)", 8.0, 4),
            ],
            task_types: TaskTypesConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    /// 验证配置的正确性
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证 Provider ID 唯一性
        let mut provider_ids = std::collections::HashSet::new();
        for provider in &self.providers {
            if !provider_ids.insert(&provider.node_id) {
                return Err(format!("重复的 Provider ID: {}", provider.node_id).into());
            }
        }

        // 验证端口唯一性
        let mut ports = std::collections::HashSet::new();
        for provider in &self.providers {
            if !ports.insert(provider.api_port) {
                return Err(format!("重复的 API 端口: {}", provider.api_port).into());
            }
        }

        // 验证任务类型配置
        self.task_types.validate()?;

        // 验证共识配置
        self.consensus.validate()?;

        Ok(())
    }

    /// 获取指定 Provider 的配置
    pub fn get_provider_config(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.node_id == provider_id)
    }

    /// 获取所有活跃的 Provider 配置
    pub fn get_active_providers(&self) -> Vec<&ProviderConfig> {
        self.providers.iter().filter(|p| p.enabled).collect()
    }
}

/// 共识配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsensusConfig {
    /// 共识服务器端点
    pub server_endpoint: String,
    /// 任务超时时间（秒）
    pub task_timeout_seconds: u64,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 结果提交重试次数
    pub result_submit_retries: usize,
    /// 健康检查间隔（秒）
    pub health_check_interval_seconds: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            server_endpoint: "http://localhost:3000".to_string(),
            task_timeout_seconds: 300, // 5分钟
            max_concurrent_tasks: 3,
            result_submit_retries: 3,
            health_check_interval_seconds: 60,
        }
    }
}

impl ConsensusConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.task_timeout_seconds == 0 {
            return Err("任务超时时间不能为0".into());
        }
        if self.max_concurrent_tasks == 0 {
            return Err("最大并发任务数不能为0".into());
        }
        if self.result_submit_retries == 0 {
            return Err("结果提交重试次数不能为0".into());
        }
        Ok(())
    }
}

/// Provider 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    /// 节点ID（唯一标识符）
    pub node_id: String,
    /// 显示名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 作弊模式配置
    pub cheating_mode: Option<CheatingMode>,
    /// API 服务器端口
    pub api_port: u16,
    /// GPU 内存大小（GB）
    pub gpu_memory_gb: f64,
    /// CPU 核心数
    pub cpu_cores: u32,
    /// 最大活跃任务数
    pub max_active_tasks: usize,
    /// 节点标签（用于分类和筛选）
    pub labels: HashMap<String, String>,
}

impl ProviderConfig {
    /// 创建诚实的 Provider 配置
    pub fn new_honest(node_id: &str, name: &str, gpu_memory_gb: f64, cpu_cores: u32) -> Self {
        Self {
            node_id: node_id.to_string(),
            name: name.to_string(),
            enabled: true,
            cheating_mode: None,
            api_port: 8080 + (node_id.chars().last().unwrap_or('0') as u16 - '0' as u16),
            gpu_memory_gb,
            cpu_cores,
            max_active_tasks: 3,
            labels: HashMap::new(),
        }
    }

    /// 创建返回错误结果的 Provider 配置
    pub fn new_cheater(node_id: &str, name: &str, gpu_memory_gb: f64, cpu_cores: u32) -> Self {
        Self {
            node_id: node_id.to_string(),
            name: name.to_string(),
            enabled: true,
            cheating_mode: Some(CheatingMode::WrongResult),
            api_port: 8080 + (node_id.chars().last().unwrap_or('0') as u16 - '0' as u16),
            gpu_memory_gb,
            cpu_cores,
            max_active_tasks: 3,
            labels: {
                let mut labels = HashMap::new();
                labels.insert("behavior".to_string(), "cheater".to_string());
                labels
            },
        }
    }

    /// 创建延迟执行的 Provider 配置
    pub fn new_delayer(node_id: &str, name: &str, gpu_memory_gb: f64, cpu_cores: u32) -> Self {
        Self {
            node_id: node_id.to_string(),
            name: name.to_string(),
            enabled: true,
            cheating_mode: Some(CheatingMode::Delay {
                min_ms: 1000,
                max_ms: 5000,
            }),
            api_port: 8080 + (node_id.chars().last().unwrap_or('0') as u16 - '0' as u16),
            gpu_memory_gb,
            cpu_cores,
            max_active_tasks: 3,
            labels: {
                let mut labels = HashMap::new();
                labels.insert("behavior".to_string(), "delayer".to_string());
                labels
            },
        }
    }

    /// 创建提前终止的 Provider 配置
    pub fn new_terminator(node_id: &str, name: &str, gpu_memory_gb: f64, cpu_cores: u32) -> Self {
        Self {
            node_id: node_id.to_string(),
            name: name.to_string(),
            enabled: true,
            cheating_mode: Some(CheatingMode::EarlyTermination {
                probability: 0.3,
            }),
            api_port: 8080 + (node_id.chars().last().unwrap_or('0') as u16 - '0' as u16),
            gpu_memory_gb,
            cpu_cores,
            max_active_tasks: 3,
            labels: {
                let mut labels = HashMap::new();
                labels.insert("behavior".to_string(), "terminator".to_string());
                labels
            },
        }
    }

    /// 检查是否有指定标签
    pub fn has_label(&self, key: &str, value: &str) -> bool {
        self.labels.get(key).map(|v| v == value).unwrap_or(false)
    }

    /// 获取作弊模式描述
    pub fn cheating_mode_description(&self) -> &'static str {
        match &self.cheating_mode {
            Some(mode) => mode.description(),
            None => "诚实节点",
        }
    }
}

/// 任务类型配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskTypesConfig {
    /// 矩阵乘法任务配置
    pub matrix_multiplication: TaskTypeConfig,
    /// 向量加法任务配置
    pub vector_addition: TaskTypeConfig,
    /// 简单推理任务配置
    pub simple_inference: TaskTypeConfig,
}

impl Default for TaskTypesConfig {
    fn default() -> Self {
        Self {
            matrix_multiplication: TaskTypeConfig {
                default_size: 100,
                max_size: 1000,
                min_size: 10,
                enabled: true,
            },
            vector_addition: TaskTypeConfig {
                default_size: 1000,
                max_size: 10000,
                min_size: 100,
                enabled: true,
            },
            simple_inference: TaskTypeConfig {
                default_size: 50,
                max_size: 500,
                min_size: 10,
                enabled: true,
            },
        }
    }
}

impl TaskTypesConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.matrix_multiplication.validate("matrix_multiplication")?;
        self.vector_addition.validate("vector_addition")?;
        self.simple_inference.validate("simple_inference")?;
        Ok(())
    }

    /// 获取指定任务类型的配置
    pub fn get_config(&self, task_type: &str) -> Option<&TaskTypeConfig> {
        match task_type {
            "matrix_multiplication" => Some(&self.matrix_multiplication),
            "vector_addition" => Some(&self.vector_addition),
            "simple_inference" => Some(&self.simple_inference),
            _ => None,
        }
    }
}

/// 单个任务类型配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskTypeConfig {
    /// 默认大小参数
    pub default_size: usize,
    /// 最大大小参数
    pub max_size: usize,
    /// 最小大小参数
    pub min_size: usize,
    /// 是否启用
    pub enabled: bool,
}

impl TaskTypeConfig {
    /// 验证配置
    pub fn validate(&self, task_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.min_size > self.default_size {
            return Err(format!("{}: 最小大小不能大于默认大小", task_type).into());
        }
        if self.default_size > self.max_size {
            return Err(format!("{}: 默认大小不能大于最大大小", task_type).into());
        }
        if self.min_size == 0 {
            return Err(format!("{}: 最小大小不能为0", task_type).into());
        }
        Ok(())
    }

    /// 验证参数值是否在有效范围内
    pub fn validate_parameter(&self, value: usize) -> Result<(), Box<dyn std::error::Error>> {
        if value < self.min_size {
            return Err(format!("参数值 {} 小于最小值 {}", value, self.min_size).into());
        }
        if value > self.max_size {
            return Err(format!("参数值 {} 大于最大值 {}", value, self.max_size).into());
        }
        Ok(())
    }
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用文件输出
    pub file_enabled: bool,
    /// 日志文件路径
    pub file_path: String,
    /// 日志格式
    pub format: String,
    /// 日志轮转配置
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: true,
            file_path: "logs/consensus-provider.log".to_string(),
            format: "detailed".to_string(),
            rotation: LogRotationConfig::default(),
        }
    }
}

/// 日志轮转配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogRotationConfig {
    /// 最大文件大小（字节）
    pub max_size_bytes: u64,
    /// 保留的文件数量
    pub max_files: usize,
    /// 是否启用压缩
    pub compress: bool,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            compress: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default_config();
        assert_eq!(config.providers.len(), 4);
        assert!(config.consensus.validate().is_ok());
        assert!(config.task_types.validate().is_ok());
    }

    #[test]
    fn test_provider_config_creation() {
        let honest = ProviderConfig::new_honest("test-1", "Test Honest", 8.0, 4);
        assert_eq!(honest.node_id, "test-1");
        assert!(honest.cheating_mode.is_none());

        let cheater = ProviderConfig::new_cheater("test-2", "Test Cheater", 8.0, 4);
        assert!(matches!(cheater.cheating_mode, Some(CheatingMode::WrongResult)));
        assert!(cheater.has_label("behavior", "cheater"));
    }

    #[test]
    fn test_task_type_config_validation() {
        let valid_config = TaskTypeConfig {
            default_size: 100,
            max_size: 1000,
            min_size: 10,
            enabled: true,
        };
        assert!(valid_config.validate("test").is_ok());

        let invalid_config = TaskTypeConfig {
            default_size: 50,
            max_size: 100,
            min_size: 75, // min > default
            enabled: true,
        };
        assert!(invalid_config.validate("test").is_err());
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default_config();

        // 测试重复 Provider ID
        config.providers.push(ProviderConfig::new_honest("provider-1", "Duplicate", 8.0, 4));
        assert!(config.validate().is_err());

        // 重置并测试重复端口
        let mut config = AppConfig::default_config();
        if let Some(provider) = config.providers.get_mut(1) {
            provider.api_port = 8081; // 与第一个重复
        }
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_operations() {
        let config = AppConfig::default_config();

        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", serde_json::to_string_pretty(&config).unwrap()).unwrap();

        // 从文件加载配置
        let loaded_config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_config.providers.len(), config.providers.len());

        // 保存配置
        let mut temp_file2 = NamedTempFile::new().unwrap();
        config.save_to_file(temp_file2.path()).unwrap();

        // 验证保存的文件可以重新加载
        let reloaded_config = AppConfig::from_file(temp_file2.path()).unwrap();
        assert_eq!(reloaded_config.providers.len(), config.providers.len());
    }
}

