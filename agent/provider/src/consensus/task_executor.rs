//! # 任务执行器模块
//!
//! 这个模块负责实际执行各种计算任务，包括矩阵乘法、向量运算和神经网络推理。
//! 支持真实的数学计算，同时集成了作弊行为模拟。

use ndarray::{Array2, Array1, Array};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::convert::TryInto;
use super::types::*;
use super::cheating_modes::*;

/// 任务执行器
#[derive(Clone)]
pub struct TaskExecutor {
    /// 执行器配置
    config: ExecutorConfig,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
        }
    }

    /// 执行任务
    pub async fn execute_task(&self, execution: TaskExecution) -> TaskResult {
        let start_time = Instant::now();

        log::info!(
            "开始执行任务: {} (类型: {}, Provider: {})",
            execution.task.id,
            execution.task.task_type.as_str(),
            execution.provider_id
        );

        // 检查是否应该延迟执行
        if let Some(delay) = execution.cheating_mode.as_ref().and_then(|mode| mode.get_delay_duration()) {
            log::warn!("模拟延迟执行: {}ms", delay.as_millis());
            tokio::time::sleep(delay).await;
        }

        // 检查是否应该提前终止
        if execution.cheating_mode.as_ref().map_or(false, |mode| mode.should_terminate_early()) {
            log::warn!("模拟提前终止任务: {}", execution.task.id);
            CheatingSimulator::log_cheating_behavior(
                &execution.provider_id,
                execution.cheating_mode.as_ref().unwrap(),
                &execution.task.id
            );

            return TaskResult::failure(
                execution.task.id,
                execution.provider_id,
                "任务被提前终止（模拟作弊行为）".to_string(),
                start_time.elapsed().as_millis() as u64,
                ResourceUsage::default(),
            );
        }

        // 执行实际计算
        let execution_result = match &execution.task.task_type {
            TaskType::MatrixMultiplication { size } => {
                self.perform_matrix_multiplication(execution.task.data.clone(), *size).await
            }
            TaskType::VectorAddition { size } => {
                self.perform_vector_addition(execution.task.data.clone(), *size).await
            }
            TaskType::SimpleInference { model_size } => {
                self.perform_simple_inference(execution.task.data.clone(), *model_size).await
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        // 检查是否应该修改结果
        let final_result = if execution.cheating_mode.as_ref().map_or(false, |mode| mode.should_modify_result()) {
            log::warn!("模拟返回错误结果: {}", execution.task.id);
            CheatingSimulator::log_cheating_behavior(
                &execution.provider_id,
                execution.cheating_mode.as_ref().unwrap(),
                &execution.task.id
            );

            ExecutionResult {
                data: CheatingSimulator::generate_wrong_result(&execution_result.data),
                success: true, // 作弊节点认为自己成功了
                error: None,
                resource_usage: CheatingSimulator::simulate_abnormal_resource_usage(&execution_result.resource_usage),
            }
        } else {
            execution_result
        };

        let result = if final_result.success {
            TaskResult::success(
                execution.task.id,
                execution.provider_id,
                final_result.data,
                execution_time,
                final_result.resource_usage,
            )
        } else {
            TaskResult::failure(
                execution.task.id,
                execution.provider_id,
                final_result.error.unwrap_or_else(|| "未知执行错误".to_string()),
                execution_time,
                final_result.resource_usage,
            )
        };

        log::info!(
            "任务执行完成: {} (成功: {}, 耗时: {}ms)",
            result.task_id,
            result.success,
            result.execution_time_ms
        );

        result
    }

    /// 执行矩阵乘法任务
    async fn perform_matrix_multiplication(&self, data: Vec<u8>, size: usize) -> ExecutionResult {
        log::debug!("执行矩阵乘法: 大小 {}x{}", size, size);

        // 解析输入数据（应该包含两个矩阵）
        match self.parse_matrices_from_bytes(&data, size) {
            Ok((matrix_a, matrix_b)) => {
                // 执行矩阵乘法
                let result = matrix_a.dot(&matrix_b);

                // 序列化结果
                match self.serialize_matrix_to_bytes(&result) {
                    Ok(result_bytes) => {
                        let resource_usage = ResourceUsage {
                            cpu_usage_percent: 85.0, // 矩阵乘法通常是 CPU 密集型
                            memory_usage_mb: (size * size * 8 * 3) as f64 / (1024.0 * 1024.0), // 估算内存使用
                            gpu_memory_usage_mb: Some((size * size * 8 * 3) as f64 / (1024.0 * 1024.0)),
                            network_usage_kb: 0.0,
                        };

                        ExecutionResult {
                            data: result_bytes,
                            success: true,
                            error: None,
                            resource_usage,
                        }
                    }
                    Err(e) => ExecutionResult {
                        data: vec![],
                        success: false,
                        error: Some(format!("序列化结果失败: {}", e)),
                        resource_usage: ResourceUsage::default(),
                    }
                }
            }
            Err(e) => ExecutionResult {
                data: vec![],
                success: false,
                error: Some(format!("解析矩阵数据失败: {}", e)),
                resource_usage: ResourceUsage::default(),
            }
        }
    }

    /// 执行向量加法任务
    async fn perform_vector_addition(&self, data: Vec<u8>, size: usize) -> ExecutionResult {
        log::debug!("执行向量加法: 长度 {}", size);

        match self.parse_vectors_from_bytes(&data, size) {
            Ok((vector_a, vector_b)) => {
                let result = &vector_a + &vector_b;

                match self.serialize_vector_to_bytes(&result) {
                    Ok(result_bytes) => {
                        let resource_usage = ResourceUsage {
                            cpu_usage_percent: 45.0, // 向量加法相对简单
                            memory_usage_mb: (size * 8 * 3) as f64 / (1024.0 * 1024.0),
                            gpu_memory_usage_mb: Some((size * 8 * 3) as f64 / (1024.0 * 1024.0)),
                            network_usage_kb: 0.0,
                        };

                        ExecutionResult {
                            data: result_bytes,
                            success: true,
                            error: None,
                            resource_usage,
                        }
                    }
                    Err(e) => ExecutionResult {
                        data: vec![],
                        success: false,
                        error: Some(format!("序列化结果失败: {}", e)),
                        resource_usage: ResourceUsage::default(),
                    }
                }
            }
            Err(e) => ExecutionResult {
                data: vec![],
                success: false,
                error: Some(format!("解析向量数据失败: {}", e)),
                resource_usage: ResourceUsage::default(),
            }
        }
    }

    /// 执行简单神经网络推理任务
    async fn perform_simple_inference(&self, data: Vec<u8>, model_size: usize) -> ExecutionResult {
        log::debug!("执行神经网络推理: 模型大小 {}", model_size);

        match self.parse_vector_from_bytes(&data) {
            Ok(input) => {
                // 生成权重矩阵（在实际应用中，这些应该是预训练的）
                let weights1 = self.generate_random_matrix(model_size, input.len());
                let weights2 = self.generate_random_matrix(model_size, model_size);

                // 前向传播
                let hidden = input.dot(&weights1.t().to_owned()); // 输入层到隐藏层
                let output = hidden.dot(&weights2.t().to_owned()); // 隐藏层到输出层

                // 应用激活函数（简单的 ReLU）
                let activated_output = output.mapv(|x| if x > 0.0 { x } else { 0.0 });

                match self.serialize_vector_to_bytes(&activated_output) {
                    Ok(result_bytes) => {
                        let resource_usage = ResourceUsage {
                            cpu_usage_percent: 65.0, // 神经网络推理需要一定计算
                            memory_usage_mb: (model_size * input.len() * 8 * 2) as f64 / (1024.0 * 1024.0),
                            gpu_memory_usage_mb: Some((model_size * input.len() * 8 * 2) as f64 / (1024.0 * 1024.0)),
                            network_usage_kb: 0.0,
                        };

                        ExecutionResult {
                            data: result_bytes,
                            success: true,
                            error: None,
                            resource_usage,
                        }
                    }
                    Err(e) => ExecutionResult {
                        data: vec![],
                        success: false,
                        error: Some(format!("序列化推理结果失败: {}", e)),
                        resource_usage: ResourceUsage::default(),
                    }
                }
            }
            Err(e) => ExecutionResult {
                data: vec![],
                success: false,
                error: Some(format!("解析输入数据失败: {}", e)),
                resource_usage: ResourceUsage::default(),
            }
        }
    }

    /// 从字节数据解析两个矩阵
    fn parse_matrices_from_bytes(&self, data: &[u8], size: usize) -> Result<(Array2<f64>, Array2<f64>), Box<dyn std::error::Error>> {
        let expected_size = size * size * 8 * 2; // 两个矩阵，每个元素8字节
        if data.len() != expected_size {
            return Err(format!("数据大小不匹配，期望 {} 字节，实际 {} 字节", expected_size, data.len()).into());
        }

        let matrix_a = self.bytes_to_matrix(&data[0..size*size*8], size)?;
        let matrix_b = self.bytes_to_matrix(&data[size*size*8..], size)?;

        Ok((matrix_a, matrix_b))
    }

    /// 从字节数据解析两个向量
    fn parse_vectors_from_bytes(&self, data: &[u8], size: usize) -> Result<(Array1<f64>, Array1<f64>), Box<dyn std::error::Error>> {
        let expected_size = size * 8 * 2; // 两个向量，每个元素8字节
        if data.len() != expected_size {
            return Err(format!("数据大小不匹配，期望 {} 字节，实际 {} 字节", expected_size, data.len()).into());
        }

        let vector_a = self.bytes_to_vector(&data[0..size*8])?;
        let vector_b = self.bytes_to_vector(&data[size*8..])?;

        Ok((vector_a, vector_b))
    }

    /// 从字节数据解析单个向量
    fn parse_vector_from_bytes(&self, data: &[u8]) -> Result<Array1<f64>, Box<dyn std::error::Error>> {
        self.bytes_to_vector(data)
    }

    /// 将字节数据转换为矩阵
    fn bytes_to_matrix(&self, bytes: &[u8], size: usize) -> Result<Array2<f64>, Box<dyn std::error::Error>> {
        let expected_size = size * size * 8;
        if bytes.len() != expected_size {
            return Err(format!("矩阵数据大小不匹配，期望 {} 字节，实际 {} 字节", expected_size, bytes.len()).into());
        }

        let mut values = Vec::with_capacity(size * size);
        for chunk in bytes.chunks_exact(8) {
            let value = f64::from_le_bytes(chunk.try_into()?);
            values.push(value);
        }

        Ok(Array2::from_shape_vec((size, size), values)?)
    }

    /// 将字节数据转换为向量
    fn bytes_to_vector(&self, bytes: &[u8]) -> Result<Array1<f64>, Box<dyn std::error::Error>> {
        if bytes.len() % 8 != 0 {
            return Err(format!("向量数据长度必须是8的倍数，实际长度 {}", bytes.len()).into());
        }

        let size = bytes.len() / 8;
        let mut values = Vec::with_capacity(size);
        for chunk in bytes.chunks_exact(8) {
            let value = f64::from_le_bytes(chunk.try_into()?);
            values.push(value);
        }

        Ok(Array1::from_vec(values))
    }

    /// 将矩阵序列化为字节数据
    fn serialize_matrix_to_bytes(&self, matrix: &Array2<f64>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::with_capacity(matrix.len() * 8);
        for &value in matrix.iter() {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }

    /// 将向量序列化为字节数据
    fn serialize_vector_to_bytes(&self, vector: &Array1<f64>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::with_capacity(vector.len() * 8);
        for &value in vector.iter() {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }

    /// 生成随机矩阵（用于神经网络权重）
    fn generate_random_matrix(&self, rows: usize, cols: usize) -> Array2<f64> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let values: Vec<f64> = (0..rows*cols)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        Array2::from_shape_vec((rows, cols), values).unwrap()
    }
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 是否启用详细日志
    pub verbose_logging: bool,
    /// 最大执行时间（秒）
    pub max_execution_time_seconds: u64,
    /// 是否模拟真实的资源使用
    pub simulate_real_resources: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            verbose_logging: true,
            max_execution_time_seconds: 300, // 5分钟
            simulate_real_resources: true,
        }
    }
}

/// 任务执行封装
#[derive(Debug, Clone)]
pub struct TaskExecution {
    /// 要执行的任务
    pub task: ConsensusTask,
    /// 执行任务的 Provider ID
    pub provider_id: String,
    /// 作弊模式（如果有）
    pub cheating_mode: Option<CheatingMode>,
}

impl TaskExecution {
    /// 创建新的任务执行
    pub fn new(task: ConsensusTask, provider_id: String, cheating_mode: Option<CheatingMode>) -> Self {
        Self {
            task,
            provider_id,
            cheating_mode,
        }
    }
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 执行结果数据
    pub data: Vec<u8>,
    /// 是否执行成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 资源使用情况
    pub resource_usage: ResourceUsage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_matrix_multiplication() {
        let executor = TaskExecutor::new();

        // 创建测试矩阵数据
        let size = 2;
        let mut data = Vec::new();

        // 矩阵 A: [[1, 2], [3, 4]]
        data.extend_from_slice(&1.0f64.to_le_bytes());
        data.extend_from_slice(&2.0f64.to_le_bytes());
        data.extend_from_slice(&3.0f64.to_le_bytes());
        data.extend_from_slice(&4.0f64.to_le_bytes());

        // 矩阵 B: [[5, 6], [7, 8]]
        data.extend_from_slice(&5.0f64.to_le_bytes());
        data.extend_from_slice(&6.0f64.to_le_bytes());
        data.extend_from_slice(&7.0f64.to_le_bytes());
        data.extend_from_slice(&8.0f64.to_le_bytes());

        let task = ConsensusTask::new(
            TaskType::MatrixMultiplication { size },
            data,
            60,
        );

        let execution = TaskExecution::new(task, "test-provider".to_string(), None);
        let result = executor.execute_task(execution).await;

        assert!(result.success);
        assert_eq!(result.provider_id, "test-provider");
        // 结果应该是 [[19, 22], [43, 50]]
        assert!(!result.result.is_empty());
    }

    #[tokio::test]
    async fn test_vector_addition() {
        let executor = TaskExecutor::new();

        // 创建测试向量数据
        let size = 3;
        let mut data = Vec::new();

        // 向量 A: [1, 2, 3]
        data.extend_from_slice(&1.0f64.to_le_bytes());
        data.extend_from_slice(&2.0f64.to_le_bytes());
        data.extend_from_slice(&3.0f64.to_le_bytes());

        // 向量 B: [4, 5, 6]
        data.extend_from_slice(&4.0f64.to_le_bytes());
        data.extend_from_slice(&5.0f64.to_le_bytes());
        data.extend_from_slice(&6.0f64.to_le_bytes());

        let task = ConsensusTask::new(
            TaskType::VectorAddition { size },
            data,
            60,
        );

        let execution = TaskExecution::new(task, "test-provider".to_string(), None);
        let result = executor.execute_task(execution).await;

        assert!(result.success);
        assert_eq!(result.provider_id, "test-provider");
        // 结果应该是 [5, 7, 9]
        assert!(!result.result.is_empty());
    }

    #[tokio::test]
    async fn test_cheating_delay() {
        let executor = TaskExecutor::new();

        let task = ConsensusTask::new(
            TaskType::VectorAddition { size: 2 },
            vec![0; 32], // 占位数据
            60,
        );

        let cheating_mode = CheatingMode::Delay { min_ms: 10, max_ms: 50 };
        let execution = TaskExecution::new(
            task,
            "cheating-provider".to_string(),
            Some(cheating_mode),
        );

        let start = std::time::Instant::now();
        let result = executor.execute_task(execution).await;
        let elapsed = start.elapsed();

        // 应该有延迟
        assert!(elapsed >= Duration::from_millis(10));
        assert!(result.success); // 即使延迟也应该成功
    }

    #[test]
    fn test_data_parsing() {
        let executor = TaskExecutor::new();

        // 测试向量解析
        let mut data = Vec::new();
        data.extend_from_slice(&1.0f64.to_le_bytes());
        data.extend_from_slice(&2.0f64.to_le_bytes());
        data.extend_from_slice(&3.0f64.to_le_bytes());

        let vector = executor.parse_vector_from_bytes(&data).unwrap();
        assert_eq!(vector.len(), 3);
        assert_eq!(vector[0], 1.0);
        assert_eq!(vector[1], 2.0);
        assert_eq!(vector[2], 3.0);
    }
}

