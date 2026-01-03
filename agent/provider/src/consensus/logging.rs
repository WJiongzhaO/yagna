//! # 日志模块
//!
//! 这个模块负责配置和初始化日志系统，支持控制台输出和文件输出。

use log::LevelFilter;
use std::fs::OpenOptions;
use std::path::Path;

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用文件输出
    pub file_enabled: bool,
    /// 日志文件路径
    pub file_path: String,
    /// 是否启用控制台输出
    pub console_enabled: bool,
    /// 日志格式
    pub format: LogFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: true,
            file_path: "logs/consensus-provider.log".to_string(),
            console_enabled: true,
            format: LogFormat::Detailed,
        }
    }
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// 简洁格式
    Simple,
    /// 详细格式（包含时间戳、级别、模块等）
    Detailed,
    /// JSON 格式（便于日志分析工具处理）
    Json,
}

impl LogFormat {
    /// 从字符串解析日志格式
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "simple" => Ok(LogFormat::Simple),
            "detailed" => Ok(LogFormat::Detailed),
            "json" => Ok(LogFormat::Json),
            _ => Err(format!("未知的日志格式: {}", s)),
        }
    }
}

/// 初始化日志系统
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_with_config(LogConfig::default())
}

/// 使用自定义配置初始化日志系统
pub fn init_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 解析日志级别
    let level = parse_log_level(&config.level)?;

    // 创建日志目录（如果不存在）
    if config.file_enabled {
        if let Some(parent) = Path::new(&config.file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // 配置 env_logger
    let mut builder = env_logger::Builder::new();

    // 设置全局日志级别
    builder.filter_level(level);

    // 配置输出格式
    match config.format {
        LogFormat::Simple => {
            builder.format_timestamp_secs();
        }
        LogFormat::Detailed => {
            builder.format_timestamp_millis();
            builder.format_module_path(true);
            builder.format_target(true);
        }
        LogFormat::Json => {
            builder.format(|buf, record| {
                use std::io::Write;
                let timestamp = buf.timestamp_millis();

                writeln!(
                    buf,
                    r#"{{"timestamp":"{}","level":"{}","target":"{}","message":"{}"}}"#,
                    timestamp,
                    record.level(),
                    record.target(),
                    record.args()
                )
            });
        }
    }

    // 如果启用文件输出，创建文件 appender
    if config.file_enabled {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.file_path)?;

        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }

    // 初始化日志系统
    builder.init();

    log::info!("日志系统已初始化");
    log::info!("日志级别: {}", config.level);
    log::info!("文件输出: {}", if config.file_enabled { &config.file_path } else { "禁用" });
    log::info!("控制台输出: {}", if config.console_enabled { "启用" } else { "禁用" });

    Ok(())
}

/// 解析日志级别字符串
fn parse_log_level(level: &str) -> Result<LevelFilter, Box<dyn std::error::Error>> {
    match level.to_lowercase().as_str() {
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        "off" => Ok(LevelFilter::Off),
        _ => Err(format!("无效的日志级别: {}", level).into()),
    }
}

/// 创建性能监控日志
pub fn log_performance(operation: &str, duration_ms: u64, success: bool) {
    if success {
        log::info!("性能监控 - 操作: {}, 耗时: {}ms, 状态: 成功", operation, duration_ms);
    } else {
        log::warn!("性能监控 - 操作: {}, 耗时: {}ms, 状态: 失败", operation, duration_ms);
    }
}

/// 创建任务执行日志
pub fn log_task_execution(task_id: &str, provider_id: &str, task_type: &str, success: bool, execution_time_ms: u64) {
    if success {
        log::info!(
            "任务执行 - ID: {}, Provider: {}, 类型: {}, 耗时: {}ms, 状态: 成功",
            task_id, provider_id, task_type, execution_time_ms
        );
    } else {
        log::error!(
            "任务执行 - ID: {}, Provider: {}, 类型: {}, 耗时: {}ms, 状态: 失败",
            task_id, provider_id, task_type, execution_time_ms
        );
    }
}

/// 创建共识事件日志
pub fn log_consensus_event(event_type: &str, task_id: &str, details: &str) {
    log::info!("共识事件 - 类型: {}, 任务: {}, 详情: {}", event_type, task_id, details);
}

/// 创建错误日志（带上下文）
pub fn log_error_with_context(context: &str, error: &dyn std::error::Error) {
    log::error!("错误上下文: {}, 错误信息: {}", context, error);

    // 记录完整的错误链
    let mut current_error = error.source();
    let mut depth = 1;
    while let Some(cause) = current_error {
        log::error!("  原因 {}: {}", depth, cause);
        current_error = cause.source();
        depth += 1;
    }
}

/// 创建安全事件日志（用于记录作弊行为等敏感事件）
pub fn log_security_event(event_type: &str, provider_id: &str, details: &str, severity: SecuritySeverity) {
    match severity {
        SecuritySeverity::Low => log::info!("安全事件 - 类型: {}, Provider: {}, 详情: {}, 严重程度: 低", event_type, provider_id, details),
        SecuritySeverity::Medium => log::warn!("安全事件 - 类型: {}, Provider: {}, 详情: {}, 严重程度: 中", event_type, provider_id, details),
        SecuritySeverity::High => log::error!("安全事件 - 类型: {}, Provider: {}, 详情: {}, 严重程度: 高", event_type, provider_id, details),
        SecuritySeverity::Critical => log::error!("安全事件 - 类型: {}, Provider: {}, 详情: {}, 严重程度: 严重", event_type, provider_id, details),
    }
}

/// 创建统计信息日志
pub fn log_statistics(stats_type: &str, data: serde_json::Value) {
    log::info!("统计信息 - 类型: {}, 数据: {}", stats_type, data);
}

/// 创建系统状态日志
pub fn log_system_status(component: &str, status: &str, details: Option<&str>) {
    match status {
        "healthy" | "running" => {
            if let Some(details) = details {
                log::info!("系统状态 - 组件: {}, 状态: {}, 详情: {}", component, status, details);
            } else {
                log::info!("系统状态 - 组件: {}, 状态: {}", component, status);
            }
        }
        "warning" | "degraded" => {
            if let Some(details) = details {
                log::warn!("系统状态 - 组件: {}, 状态: {}, 详情: {}", component, status, details);
            } else {
                log::warn!("系统状态 - 组件: {}, 状态: {}", component, status);
            }
        }
        "error" | "failed" | "critical" => {
            if let Some(details) = details {
                log::error!("系统状态 - 组件: {}, 状态: {}, 详情: {}", component, status, details);
            } else {
                log::error!("系统状态 - 组件: {}, 状态: {}", component, status);
            }
        }
        _ => {
            if let Some(details) = details {
                log::info!("系统状态 - 组件: {}, 状态: {}, 详情: {}", component, status, details);
            } else {
                log::info!("系统状态 - 组件: {}, 状态: {}", component, status);
            }
        }
    }
}

/// 安全事件严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    /// 低严重程度
    Low,
    /// 中等严重程度
    Medium,
    /// 高严重程度
    High,
    /// 严重程度
    Critical,
}

impl SecuritySeverity {
    /// 从字符串解析严重程度
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "low" => Ok(SecuritySeverity::Low),
            "medium" => Ok(SecuritySeverity::Medium),
            "high" => Ok(SecuritySeverity::High),
            "critical" => Ok(SecuritySeverity::Critical),
            _ => Err(format!("未知的安全严重程度: {}", s)),
        }
    }
}

/// 日志轮转配置（用于日志文件管理）
#[derive(Debug, Clone)]
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

/// 检查并轮转日志文件
pub fn check_and_rotate_log(config: &LogConfig, rotation_config: &LogRotationConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.file_enabled {
        return Ok(());
    }

    let log_path = Path::new(&config.file_path);

    // 检查文件是否存在以及大小
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() >= rotation_config.max_size_bytes {
            log::info!("日志文件达到大小限制，开始轮转");

            // 轮转现有日志文件
            rotate_log_files(log_path, rotation_config.max_files)?;

            log::info!("日志文件轮转完成");
        }
    }

    Ok(())
}

/// 执行日志文件轮转
fn rotate_log_files(log_path: &Path, max_files: usize) -> Result<(), Box<dyn std::error::Error>> {
    // 从最旧的文件开始重命名
    for i in (1..max_files).rev() {
        let old_name = format!("{}.{}", log_path.display(), i);
        let new_name = format!("{}.{}", log_path.display(), i + 1);

        let old_path = Path::new(&old_name);
        let new_path = Path::new(&new_name);

        // 如果旧文件存在，删除最旧的文件
        if i == max_files - 1 && old_path.exists() {
            std::fs::remove_file(old_path)?;
        }

        // 重命名文件
        if old_path.exists() {
            std::fs::rename(old_path, new_path)?;
        }
    }

    // 将当前日志文件重命名为 .1
    if log_path.exists() {
        let backup_path = format!("{}.1", log_path.display());
        std::fs::rename(log_path, &backup_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("error").unwrap(), LevelFilter::Error);
        assert_eq!(parse_log_level("INFO").unwrap(), LevelFilter::Info);
        assert_eq!(parse_log_level("debug").unwrap(), LevelFilter::Debug);
        assert_eq!(parse_log_level("off").unwrap(), LevelFilter::Off);
        assert!(parse_log_level("invalid").is_err());
    }

    #[test]
    fn test_log_format_from_str() {
        assert!(matches!(LogFormat::from_str("simple").unwrap(), LogFormat::Simple));
        assert!(matches!(LogFormat::from_str("DETAILED").unwrap(), LogFormat::Detailed));
        assert!(matches!(LogFormat::from_str("json").unwrap(), LogFormat::Json));
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_security_severity() {
        assert_eq!(SecuritySeverity::from_str("low").unwrap(), SecuritySeverity::Low);
        assert_eq!(SecuritySeverity::from_str("HIGH").unwrap(), SecuritySeverity::High);
        assert!(SecuritySeverity::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.file_enabled);
        assert!(config.console_enabled);
        assert!(matches!(config.format, LogFormat::Detailed));
    }
}

