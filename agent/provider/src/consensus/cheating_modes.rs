//! # 作弊行为模拟模块
//!
//! 这个模块实现了各种 Provider 节点的作弊行为，用于测试共识机制的有效性。
//! 通过模拟不同类型的作弊行为，我们可以验证共识算法是否能正确识别和惩罚恶意节点。

use serde::{Deserialize, Serialize};
use rand::Rng;
use std::time::Duration;
use super::types::ResourceUsage;

/// 作弊模式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheatingMode {
    /// 延迟执行 - 在正常执行前增加随机延迟
    #[serde(rename = "delay")]
    Delay {
        /// 最小延迟时间（毫秒）
        min_ms: u64,
        /// 最大延迟时间（毫秒）
        max_ms: u64,
    },

    /// 返回错误结果 - 执行正确计算但故意修改结果
    #[serde(rename = "wrong_result")]
    WrongResult,

    /// 提前终止 - 在执行过程中随机终止
    #[serde(rename = "early_termination")]
    EarlyTermination {
        /// 终止概率 (0.0-1.0)
        probability: f64,
    },

    /// 混合作弊 - 随机选择多种作弊方式
    #[serde(rename = "mixed")]
    Mixed {
        /// 各种作弊行为的权重
        weights: CheatingWeights,
    },

    /// 智能作弊 - 根据任务类型选择最优作弊策略
    #[serde(rename = "smart")]
    Smart {
        /// 智能作弊配置
        config: SmartCheatingConfig,
    },
}

impl CheatingMode {
    /// 获取作弊模式的描述
    pub fn description(&self) -> &'static str {
        match self {
            CheatingMode::Delay { .. } => "延迟执行",
            CheatingMode::WrongResult => "返回错误结果",
            CheatingMode::EarlyTermination { .. } => "提前终止",
            CheatingMode::Mixed { .. } => "混合作弊",
            CheatingMode::Smart { .. } => "智能作弊",
        }
    }

    /// 检查是否应该作弊
    pub fn should_cheat(&self) -> bool {
        match self {
            CheatingMode::Delay { .. } => true, // 延迟模式总是延迟
            CheatingMode::WrongResult => true, // 错误结果模式总是返回错误结果
            CheatingMode::EarlyTermination { probability } => {
                rand::thread_rng().gen_bool(*probability)
            }
            CheatingMode::Mixed { weights } => {
                let total_weight: f64 = weights.delay + weights.wrong_result + weights.early_termination;
                let rand_val = rand::thread_rng().gen_range(0.0..total_weight);

                // 根据权重选择作弊类型
                if rand_val < weights.delay {
                    true // 选择延迟
                } else if rand_val < weights.delay + weights.wrong_result {
                    true // 选择错误结果
                } else {
                    rand::thread_rng().gen_bool(weights.early_termination / total_weight)
                }
            }
            CheatingMode::Smart { config } => {
                // 智能作弊：根据配置的概率决定是否作弊
                rand::thread_rng().gen_bool(config.cheating_probability)
            }
        }
    }

    /// 获取延迟时间（如果适用）
    pub fn get_delay_duration(&self) -> Option<Duration> {
        match self {
            CheatingMode::Delay { min_ms, max_ms } => {
                let delay_ms = rand::thread_rng().gen_range(*min_ms..=*max_ms);
                Some(Duration::from_millis(delay_ms))
            }
            CheatingMode::Mixed { weights } => {
                if rand::thread_rng().gen_bool(weights.delay / (weights.delay + weights.wrong_result + weights.early_termination)) {
                    let delay_ms = rand::thread_rng().gen_range(1000..=5000);
                    Some(Duration::from_millis(delay_ms))
                } else {
                    None
                }
            }
            CheatingMode::Smart { config } => {
                if rand::thread_rng().gen_bool(config.delay_probability) {
                    let delay_ms = rand::thread_rng().gen_range(config.min_delay_ms..=config.max_delay_ms);
                    Some(Duration::from_millis(delay_ms))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 检查是否应该提前终止
    pub fn should_terminate_early(&self) -> bool {
        match self {
            CheatingMode::EarlyTermination { probability } => {
                rand::thread_rng().gen_bool(*probability)
            }
            CheatingMode::Mixed { weights } => {
                let total_weight = weights.delay + weights.wrong_result + weights.early_termination;
                rand::thread_rng().gen_bool(weights.early_termination / total_weight)
            }
            CheatingMode::Smart { config } => {
                rand::thread_rng().gen_bool(config.termination_probability)
            }
            _ => false,
        }
    }

    /// 检查是否应该修改结果
    pub fn should_modify_result(&self) -> bool {
        match self {
            CheatingMode::WrongResult => true,
            CheatingMode::Mixed { weights } => {
                let total_weight = weights.delay + weights.wrong_result + weights.early_termination;
                rand::thread_rng().gen_bool(weights.wrong_result / total_weight)
            }
            CheatingMode::Smart { config } => {
                rand::thread_rng().gen_bool(config.wrong_result_probability)
            }
            _ => false,
        }
    }
}

/// 混合作弊的权重配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheatingWeights {
    /// 延迟行为的权重
    pub delay: f64,
    /// 错误结果行为的权重
    pub wrong_result: f64,
    /// 提前终止行为的权重
    pub early_termination: f64,
}

impl Default for CheatingWeights {
    fn default() -> Self {
        Self {
            delay: 1.0,
            wrong_result: 1.0,
            early_termination: 0.5,
        }
    }
}

/// 智能作弊配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartCheatingConfig {
    /// 总体作弊概率
    pub cheating_probability: f64,
    /// 延迟概率（在决定作弊后）
    pub delay_probability: f64,
    /// 最小延迟时间（毫秒）
    pub min_delay_ms: u64,
    /// 最大延迟时间（毫秒）
    pub max_delay_ms: u64,
    /// 返回错误结果概率（在决定作弊后）
    pub wrong_result_probability: f64,
    /// 提前终止概率（在决定作弊后）
    pub termination_probability: f64,
}

impl Default for SmartCheatingConfig {
    fn default() -> Self {
        Self {
            cheating_probability: 0.7,
            delay_probability: 0.4,
            min_delay_ms: 2000,
            max_delay_ms: 8000,
            wrong_result_probability: 0.4,
            termination_probability: 0.2,
        }
    }
}

/// 作弊行为模拟器
pub struct CheatingSimulator;

impl CheatingSimulator {
    /// 生成错误的计算结果
    pub fn generate_wrong_result(correct_result: &[u8]) -> Vec<u8> {
        let mut wrong_result = correct_result.to_vec();
        let mut rng = rand::thread_rng();

        // 随机修改几个字节
        let modifications = rng.gen_range(1..=3);
        for _ in 0..modifications {
            if !wrong_result.is_empty() {
                let idx = rng.gen_range(0..wrong_result.len());
                wrong_result[idx] = rng.gen();
            }
        }

        log::warn!("生成错误结果：正确结果长度={}, 修改了{}个字节",
                  correct_result.len(), modifications);
        wrong_result
    }

    /// 生成随机的延迟时间
    pub fn random_delay(min_ms: u64, max_ms: u64) -> Duration {
        let delay_ms = rand::thread_rng().gen_range(min_ms..=max_ms);
        Duration::from_millis(delay_ms)
    }

    /// 检查是否应该执行某种作弊行为
    pub fn should_perform_cheat(probability: f64) -> bool {
        rand::thread_rng().gen_bool(probability)
    }

    /// 模拟资源使用异常（作弊节点的资源使用可能不正常）
    pub fn simulate_abnormal_resource_usage(normal_usage: &ResourceUsage) -> ResourceUsage {
        let mut abnormal = normal_usage.clone();
        let mut rng = rand::thread_rng();

        // 随机增加或减少资源使用
        abnormal.cpu_usage_percent = (abnormal.cpu_usage_percent * rng.gen_range(0.5..=2.0)).min(100.0);
        abnormal.memory_usage_mb *= rng.gen_range(0.8..=1.5);
        if let Some(gpu_mem) = abnormal.gpu_memory_usage_mb {
            abnormal.gpu_memory_usage_mb = Some(gpu_mem * rng.gen_range(0.7..=1.8));
        }

        abnormal
    }

    /// 记录作弊行为到日志
    pub fn log_cheating_behavior(provider_id: &str, cheating_mode: &CheatingMode, task_id: &str) {
        log::warn!(
            "检测到 Provider {} 的作弊行为: 模式={:?}, 任务ID={}",
            provider_id,
            cheating_mode,
            task_id
        );

        // 可以在这里添加更多的作弊行为记录逻辑
        // 比如写入专门的作弊日志文件，或发送到监控系统
    }

    /// 生成作弊行为的统计信息
    pub fn generate_cheating_statistics(
        provider_id: &str,
        total_tasks: usize,
        cheated_tasks: usize,
    ) -> CheatingStatistics {
        let cheating_rate = if total_tasks > 0 {
            cheated_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        CheatingStatistics {
            provider_id: provider_id.to_string(),
            total_tasks,
            cheated_tasks,
            cheating_rate,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// 作弊统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheatingStatistics {
    /// Provider 节点ID
    pub provider_id: String,
    /// 总任务数
    pub total_tasks: usize,
    /// 作弊任务数
    pub cheated_tasks: usize,
    /// 作弊率 (0.0-1.0)
    pub cheating_rate: f64,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl CheatingStatistics {
    /// 获取作弊率的百分比表示
    pub fn cheating_rate_percent(&self) -> f64 {
        self.cheating_rate * 100.0
    }

    /// 判断是否为高风险作弊节点
    pub fn is_high_risk(&self) -> bool {
        self.cheating_rate > 0.5 || (self.total_tasks > 10 && self.cheated_tasks > 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_mode() {
        let mode = CheatingMode::Delay { min_ms: 1000, max_ms: 5000 };
        assert!(mode.get_delay_duration().is_some());
        assert!(mode.should_cheat());
        assert!(!mode.should_terminate_early());
        assert!(!mode.should_modify_result());
    }

    #[test]
    fn test_wrong_result_mode() {
        let mode = CheatingMode::WrongResult;
        assert!(mode.should_modify_result());
        assert!(mode.should_cheat());
        assert!(!mode.should_terminate_early());
    }

    #[test]
    fn test_early_termination_mode() {
        let mode = CheatingMode::EarlyTermination { probability: 0.5 };
        // 概率性测试，多次运行以确保逻辑正确
        let mut terminate_count = 0;
        for _ in 0..100 {
            if mode.should_terminate_early() {
                terminate_count += 1;
            }
        }
        // 应该有一定数量的提前终止（大概率在20-80之间）
        assert!(terminate_count > 10 && terminate_count < 90);
    }

    #[test]
    fn test_generate_wrong_result() {
        let original = vec![1, 2, 3, 4, 5];
        let wrong = CheatingSimulator::generate_wrong_result(&original);
        assert_eq!(wrong.len(), original.len());
        // 结果应该有变化
        assert_ne!(wrong, original);
    }

    #[test]
    fn test_cheating_statistics() {
        let stats = CheatingStatistics {
            provider_id: "test-provider".to_string(),
            total_tasks: 100,
            cheated_tasks: 30,
            cheating_rate: 0.3,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(stats.cheating_rate_percent(), 30.0);
        assert!(!stats.is_high_risk()); // 30% 作弊率不算高风险

        let high_risk_stats = CheatingStatistics {
            provider_id: "bad-provider".to_string(),
            total_tasks: 50,
            cheated_tasks: 40,
            cheating_rate: 0.8,
            last_updated: chrono::Utc::now(),
        };

        assert!(high_risk_stats.is_high_risk()); // 80% 作弊率算高风险
    }
}

