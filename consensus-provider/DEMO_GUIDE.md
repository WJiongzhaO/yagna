# Consensus Provider 演示指南

## 🎭 演示概述

Consensus Provider 模拟器提供了多种演示案例，用于展示去中心化GPU算力共享平台的核心功能。

## 📋 演示类型

### 1. 基础功能演示 (`basic`)
展示系统的核心组件和基本功能。

```bash
# Linux/macOS
./demo.sh basic

# Windows
demo.bat basic
```

**演示内容:**
- ✅ 核心类型系统验证
- ✅ 配置管理系统测试
- ✅ 数学计算引擎演示
- ✅ 作弊行为模拟展示
- ✅ 网络通信框架测试
- ✅ 异步生命周期概念验证

### 2. 多节点演示 (`multi`)
展示多个Provider节点同时运行的分布式环境。

```bash
# 使用默认配置
./demo.sh multi

# 指定配置文件
./demo.sh multi -c config/default.json

# 指定服务器地址
./demo.sh multi -s http://192.168.1.100:3000
```

**演示内容:**
- 🚀 同时启动多个Provider节点
- 🌐 每个节点提供独立的API接口
- 📊 实时状态监控
- 🔄 任务分发和执行
- 📈 性能统计收集

### 3. 作弊检测演示 (`cheating`)
专门展示作弊行为的检测和处理机制。

```bash
# 使用专门的作弊检测配置
./demo.sh cheating -c config/cheating.json
```

**演示内容:**
- 🎭 多种作弊行为类型
- 🔍 作弊检测算法
- ⚖️ 共识验证机制
- 💰 奖励/惩罚系统
- 📝 安全事件日志

### 4. 完整功能演示 (`full`)
结合所有功能，展示完整的系统能力。

```bash
./demo.sh full
```

**演示内容:**
- 📋 基础功能测试
- 🏗️ 多节点系统搭建
- 🎭 作弊行为分析
- 📊 完整性能报告

## 🔧 环境准备

### 1. 编译项目

```bash
# 确保在项目根目录
cd consensus-provider

# 编译发布版本
cargo build --release
```

### 2. 配置文件

系统使用JSON格式的配置文件，位于 `config/` 目录：

- `config/default.json` - 默认多节点配置
- `config/cheating.json` - 作弊检测专用配置

### 3. 共识服务器

大部分演示需要共识服务器运行：

```bash
# 启动共识服务器 (假设已实现)
# consensus-server --port 3000
```

如果没有共识服务器，演示仍会运行，但无法接收实际任务。

## 🎯 演示案例详解

### 基础功能演示

这个演示验证系统的各个组件是否正常工作：

```
🚀 Consensus Provider 模拟器 v0.1.0
🔧 启动完整功能演示...

📋 测试核心类型和配置...
✅ 矩阵乘法任务创建成功: task_001
✅ 向量加法任务创建成功: task_002
✅ 应用配置创建成功，包含 3 个providers

🧮 测试任务执行器...
✅ 向量加法任务执行成功 (耗时: 45ms)
✅ 作弊任务执行完成 (返回错误结果: true)

🌐 测试共识客户端...
✅ 共识客户端创建成功
⚠️  Provider注册失败 (预期行为，无服务器)

🏗️ 测试Provider节点...
✅ 诚实Provider配置创建成功: demo-honest
✅ 作弊Provider配置创建成功: demo-cheater (返回错误结果)
ℹ️  ProviderNode异步生命周期概念验证
```

### 多节点演示

展示分布式Provider网络：

```
🏗️ 启动多节点演示...
📁 加载配置: config/default.json
🌐 服务器: http://localhost:3000

📊 配置加载成功:
   • Provider数量: 7
   • 共识服务器: http://localhost:3000

🚀 启动Provider 1/7: 诚实节点 (honest-1)
   ✅ 创建成功 - 端口: 8081

🚀 启动Provider 2/7: 延迟作弊节点 (delay-cheater)
   ✅ 创建成功 - 端口: 8085

🎯 多节点演示运行中...
📋 可用的API端点:
   • 诚实节点: http://localhost:8081/
   • 延迟作弊节点: http://localhost:8085/
```

### 作弊检测演示

专门分析各种作弊行为的特征：

```
🎭 启动作弊检测演示...
📁 加载配置: config/cheating.json

🎯 作弊检测演示配置:
   • 诚实节点1: 诚实执行
   • 诚实节点2: 诚实执行
   • 错误结果作弊节点: 返回错误结果
   • 延迟执行作弊节点: 延迟执行任务
   • 提前终止作弊节点: 提前终止任务
   • 智能作弊节点: 智能作弊行为

🎭 作弊检测演示说明:
   诚实节点: 返回正确的计算结果
   作弊节点: 可能返回错误结果、延迟执行或提前终止
   共识机制: 通过多节点比较检测作弊行为
```

## 🌐 API 接口使用

每个Provider节点都提供REST API接口：

### 获取节点状态
```bash
curl http://localhost:8081/status
```

### 获取任务统计
```bash
curl http://localhost:8081/tasks/stats
```

### 获取健康状态
```bash
curl http://localhost:8081/health
```

### 获取共识统计
```bash
curl http://localhost:8081/consensus/stats
```

## 📊 监控和调试

### 日志文件
演示运行时会生成详细的日志文件：
- `logs/demo.log` - 一般演示日志
- `logs/cheating_demo.log` - 作弊检测日志

### 性能指标
系统收集以下性能指标：
- 任务执行时间
- CPU/内存/GPU使用率
- 网络延迟
- 成功/失败率

### 安全事件
作弊行为会被记录为安全事件：
- 延迟执行检测
- 结果不一致检测
- 提前终止检测
- 奖励/惩罚记录

## 🚀 高级用法

### 自定义配置

创建自定义配置文件：

```json
{
  "consensus": {
    "server_endpoint": "http://your-server:3000",
    "task_timeout_seconds": 600
  },
  "providers": [
    {
      "node_id": "custom-provider",
      "name": "自定义节点",
      "cheating_mode": "WrongResult",
      "api_port": 9090,
      "gpu_memory_gb": 16.0,
      "cpu_cores": 8
    }
  ]
}
```

### 远程服务器

连接到远程共识服务器：

```bash
./demo.sh multi -s http://remote-server:3000
```

### 集群部署

在多台机器上运行不同的Provider节点：

```bash
# 机器1: 诚实节点
./demo.sh basic

# 机器2: 作弊节点
./demo.sh cheating -c config/single-cheater.json
```

## 🔧 故障排除

### 编译错误
```bash
# 清理并重新编译
cargo clean
cargo build --release
```

### 端口冲突
```bash
# 检查端口使用情况
netstat -tlnp | grep 808
# 修改配置文件中的端口
```

### 共识服务器连接失败
```bash
# 检查服务器状态
curl http://localhost:3000/health

# 确认服务器地址
./demo.sh multi -s http://correct-server:3000
```

### 权限问题
```bash
# Linux: 添加执行权限
chmod +x demo.sh

# Windows: 以管理员身份运行
# 或者在PowerShell中执行
```

## 📈 扩展演示

### 性能测试
```bash
# 大规模矩阵运算
# 修改配置中的 matrix_multiplication.max_size

# 高并发测试
# 增加 max_concurrent_tasks
```

### 网络测试
```bash
# 跨网络部署
# 修改 server_endpoint 为公网地址

# 网络延迟模拟
# 使用 tc 命令模拟网络延迟
```

### 安全分析
```bash
# 多种作弊组合
# 创建包含多种作弊类型的配置文件

# 检测算法优化
# 分析日志中的检测准确率
```

## 🎉 总结

通过这些演示案例，您可以：

1. **理解系统架构** - 掌握去中心化算力共享的核心组件
2. **体验功能特性** - 实际运行各种计算任务和作弊模拟
3. **学习共识机制** - 观察多节点验证和奖励/惩罚过程
4. **分析性能表现** - 监控系统在不同负载下的表现
5. **探索安全特性** - 研究作弊检测和防御机制

每个演示案例都提供了独特的视角，帮助您全面了解这个创新的去中心化计算平台。
