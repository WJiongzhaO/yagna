# Consensus Provider 模拟器

去中心化 GPU 算力共享平台的 Provider 节点模拟器，用于演示多节点算力任务验证共识机制。

## 📊 项目状态

✅ **核心功能已完成实现** - 支持完整的 Provider 节点模拟和作弊行为演示

### 已实现的核心功能
- ✅ **类型定义系统** - 完整的任务类型、结果类型和状态管理
- ✅ **配置管理系统** - 支持 JSON 配置和多节点部署
- ✅ **数学计算引擎** - 矩阵乘法、向量运算、神经网络推理
- ✅ **作弊行为模拟** - 延迟执行、错误结果、提前终止
- ✅ **网络通信框架** - HTTP 客户端、任务轮询、结果提交
- ✅ **异步生命周期管理** - Provider 节点状态管理和任务调度
- ✅ **REST API 接口** - 完整的监控和管理 API
- ✅ **演示系统** - 四种演示模式，支持教学和测试

## ✨ 功能特性

- **🎭 四种演示模式**: basic(基础功能)、multi(多节点)、cheating(作弊检测)、full(完整演示)
- **🏗️  多节点模拟**: 同时运行多个 Provider 节点，支持诚实节点和各种作弊节点
- **🧮 真实计算任务**: 支持矩阵乘法、向量运算和神经网络推理等实际计算任务
- **⚠️  作弊行为模拟**: 内置多种作弊模式，包括延迟执行、返回错误结果、提前终止等
- **🔗 网络通信**: 与共识服务器通信，参与结果验证和奖励/惩罚机制
- **📊 实时监控**: 提供 REST API 和 WebSocket 接口用于实时状态监控
- **📝 详细日志**: 支持多种日志格式和轮转配置
- **🎯 实验设计**: 完整的实验环境，支持结果分析和系统演示

- **🎭 多节点模拟**: 同时运行多个 Provider 节点，支持诚实节点和各种作弊节点
- **🧮 真实计算任务**: 支持矩阵乘法、向量运算和神经网络推理等实际计算任务
- **⚠️  作弊行为模拟**: 内置多种作弊模式，包括延迟执行、返回错误结果、提前终止等
- **🔗 网络通信**: 与共识服务器通信，参与结果验证和奖励/惩罚机制
- **📊 实时监控**: 提供 REST API 和 WebSocket 接口用于实时状态监控
- **📝 详细日志**: 支持多种日志格式和轮转配置
- **🎯 演示模式**: 内置多种演示场景，便于测试和展示

## 🚀 快速开始

### 1. 编译项目

```bash
# 进入 consensus-provider 目录
cd consensus-provider
cargo build --release
```

### 2. 运行演示

项目提供了四种内置演示模式：

```bash
# 基础功能演示（默认）
cargo run --release -- --demo basic

# 多节点演示
cargo run --release -- --demo multi

# 作弊检测演示
cargo run --release -- --demo cheating

# 完整功能演示
cargo run --release -- --demo full
```

### 运行示例输出

```bash
🚀 Consensus Provider 模拟器 v0.1.0
🎭 演示类型: basic
📁 配置文件: config/default.json
🌐 服务器端点: http://localhost:7464

🔧 启动基础功能演示...

📋 测试核心类型和配置...
✅ 矩阵乘法任务创建成功: b8c91ec8-b1f9-4653-9db1-670178e53eb0
✅ 向量加法任务创建成功: 58bb94cc-7940-40e4-a5a6-70d2ba48849c
✅ 任务结果创建成功: demo-task-id (耗时: 150ms)
✅ 应用配置创建成功，包含 4 个providers
  Provider 1: Honest Provider (provider-1) - 端口 8081
  Provider 2: Dishonest Provider (Wrong Result) (provider-2) - 端口 8082
  Provider 3: Dishonest Provider (Delay) (provider-3) - 端口 8083
  Provider 4: Dishonest Provider (Early Termination) (provider-4) - 端口 8084

🧮 测试任务执行器...
✅ 向量加法任务执行成功 (耗时: 0ms)
  CPU使用率: 45.0%, 内存: 0.0MB

🎭 测试作弊行为...
✅ 作弊任务执行完成 (返回错误结果: true)

🌐 测试共识客户端...
✅ 共识客户端创建成功
⚠️  Provider注册失败 (预期行为，无服务器): 注册失败:

🏗️ 测试Provider节点...
✅ 诚实Provider配置创建成功: demo-honest
✅ 作弊Provider配置创建成功: demo-cheater (返回错误结果)

🎉 基础功能演示完成！
```

### 3. 使用配置文件和服务器

```bash
# 使用自定义配置文件
cargo run --release -- --demo basic --config config/cheating.json

# 指定共识服务器端点
cargo run --release -- --demo multi --server http://localhost:7465

# 完整参数示例
cargo run --release -- --demo multi --config config/default.json --server http://192.168.1.100:7465
```

### 4. 监控节点状态

#### 多节点演示时可用的API端点：

**Provider 1 (端口 8081):**
- http://localhost:8081/health - 健康检查
- http://localhost:8081/providers - 查看所有 Provider 状态
- http://localhost:8081/providers/provider-1 - 查看特定 Provider 信息
- http://localhost:8081/providers/provider-1/tasks - 查看 Provider 任务历史
- http://localhost:8081/stats - 系统统计信息
- http://localhost:8081/consensus/stats - 共识统计信息
- ws://localhost:8081/ws - WebSocket 实时监控

**Provider 2 (端口 8082):**
- http://localhost:8082/health
- http://localhost:8082/providers
- ...

**Provider 3 (端口 8083), Provider 4 (端口 8084)** 依此类推。

## ⚙️ 配置说明

### 配置文件结构

项目使用 JSON 格式的配置文件，支持以下配置项：

```json
{
  "consensus": {
    "server_endpoint": "http://localhost:3000",
    "task_timeout_seconds": 300,
    "max_concurrent_tasks": 3
  },
  "providers": [
    {
      "node_id": "provider-1",
      "name": "Honest Provider",
      "cheating_mode": null,
      "api_port": 8081,
      "gpu_memory_gb": 8.0,
      "cpu_cores": 4,
      "enabled": true,
      "max_active_tasks": 1
    }
  ],
  "task_types": {
    "matrix_multiplication": {
      "default_size": 100,
      "max_size": 1000,
      "min_size": 10
    },
    "vector_addition": {
      "default_size": 1000,
      "max_size": 10000,
      "min_size": 100
    },
    "simple_inference": {
      "default_model_size": 50,
      "max_model_size": 500,
      "min_model_size": 10
    }
  },
  "logging": {
    "level": "info",
    "file_enabled": true,
    "file_path": "logs/consensus-provider.log"
  }
}
```

### Provider 节点配置

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `node_id` | string | 节点唯一标识符 | - |
| `name` | string | 显示名称 | - |
| `enabled` | boolean | 是否启用节点 | true |
| `cheating_mode` | object/null | 作弊模式配置 | null |
| `api_port` | number | API 服务器端口 | 0 (自动分配) |
| `gpu_memory_gb` | number | GPU 内存大小（GB） | 8.0 |
| `cpu_cores` | number | CPU 核心数 | 4 |
| `max_active_tasks` | number | 最大活跃任务数 | 1 |

### 作弊模式配置

| 模式 | 描述 | 配置示例 |
|------|------|----------|
| `null` | 诚实节点 | `"cheating_mode": null` |
| `WrongResult` | 返回错误结果 | `"cheating_mode": "WrongResult"` |
| `Delay` | 延迟执行 | `"cheating_mode": {"Delay": {"min_ms": 1000, "max_ms": 5000}}` |
| `EarlyTermination` | 提前终止 | `"cheating_mode": {"EarlyTermination": {"probability": 0.3}}` |

### 内置配置文件

项目提供了两个配置文件：

- `config/default.json` - 默认配置，包含4个不同类型的节点
- `config/cheating.json` - 专门用于作弊演示的配置

## 🔌 API 接口

### REST API

#### GET /health
获取系统健康状态。

**响应示例:**
```json
{
  "success": true,
  "message": "Success",
  "data": {
    "overall_status": "Healthy",
    "cpu_usage_percent": 45.0,
    "memory_usage_percent": 60.0,
    "disk_usage_percent": 25.0,
    "network_connected": true,
    "active_tasks_count": 0,
    "last_updated": "2024-01-01T12:00:00Z"
  }
}
```

#### GET /providers
获取所有 Provider 节点状态。

**响应示例:**
```json
{
  "success": true,
  "message": "Success",
  "data": [
    {
      "provider_id": "provider-1",
      "name": "Honest Provider",
      "status": "Running",
      "active_tasks": 0,
      "total_completed_tasks": 15,
      "gpu_memory_gb": 8.0,
      "cpu_cores": 4,
      "last_updated": "2024-01-01T12:00:00Z"
    }
  ]
}
```

#### GET /providers/{id}
获取特定 Provider 节点详细信息。

#### GET /providers/{id}/tasks
获取 Provider 的任务执行历史。

#### GET /stats
获取系统汇总统计信息。

#### GET /consensus/stats
获取共识网络统计信息。

#### PUT /providers/{id}
更新 Provider 节点配置。

**请求体:**
```json
{
  "name": "新名称",
  "gpu_memory_gb": 16.0,
  "cpu_cores": 8
}
```

#### DELETE /stats/reset
重置所有统计信息。

### WebSocket 接口

#### 连接地址
`ws://localhost:{port}/ws`

用于实时监控节点状态变化，支持：
- 任务执行状态更新
- Provider 状态变化
- 系统统计信息更新

## 🎮 命令行选项

```bash
Consensus Provider 模拟器 v0.1.0
去中心化GPU算力共享平台

USAGE:
    consensus-provider.exe [OPTIONS]

OPTIONS:
    -d, --demo <TYPE>          运行演示案例: basic(基础功能), multi(多节点), cheating(作弊检测), full(完整演示) [默认: basic]
    -c, --config <FILE>        配置文件路径 [默认: config/default.json]
    -s, --server <ENDPOINT>    共识服务器端点 [默认: http://localhost:3000]
    -h, --help                 显示帮助信息
    -V, --version              显示版本信息
```

### 演示模式详解

#### `basic` - 基础功能演示
验证系统的各个核心组件是否正常工作：
- ✅ 核心类型系统验证
- ✅ 配置管理系统测试
- ✅ 数学计算引擎演示
- ✅ 作弊行为模拟展示
- ✅ 网络通信框架测试
- ✅ 异步生命周期概念验证

#### `multi` - 多节点演示
展示多个Provider节点同时运行的分布式环境：
- 🚀 同时启动多个Provider节点
- 🌐 每个节点提供独立的API接口
- 📊 实时状态监控
- 🔄 任务分发和执行
- 📈 性能统计收集

#### `cheating` - 作弊检测演示
专门展示作弊行为的检测和处理机制：
- 🎭 展示各种作弊行为的执行过程
- 🔍 验证共识机制对作弊的识别能力
- 📋 演示惩罚和奖励机制

#### `full` - 完整功能演示
结合所有功能的多阶段演示：
- 📋 第一阶段: 基础功能测试
- 🏗️ 第二阶段: 多节点系统演示

## 🏗️ 架构说明

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Consensus     │    │  Provider Node   │    │   Task          │
│   Server        │◄──►│   Simulator      │◄──►│   Executor      │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         ▲                       ▲                       ▲
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                       ┌──────────────────┐
                       │  Cheating Modes  │
                       │   Simulator      │
                       └──────────────────┘
```

### 核心组件

- **Consensus Client**: 负责与共识服务器通信，接收任务并提交结果
- **Provider Node**: 管理节点生命周期，协调各组件工作
- **Task Executor**: 执行实际的数学计算任务
- **API Server**: 提供 REST API 和 WebSocket 接口用于监控
- **Cheating Simulator**: 模拟各种作弊行为用于测试共识机制

## 📁 项目结构

```
consensus-provider/
├── src/
│   ├── main.rs                    # 主程序入口，四种演示模式
│   ├── lib.rs                     # 库入口，导出所有模块
│   ├── provider_node.rs           # Provider 节点核心逻辑
│   ├── task_executor.rs           # 任务执行器，数学计算引擎
│   ├── cheating_modes.rs          # 作弊行为模拟器
│   ├── api_server.rs             # REST API 和 WebSocket 服务器
│   ├── consensus_client.rs        # 共识服务器客户端
│   ├── logging.rs                 # 日志系统配置
│   ├── config.rs                  # 配置管理和验证
│   └── types.rs                   # 数据类型定义
├── config/
│   ├── default.json               # 默认配置（4个节点：1个诚实+3个作弊）
│   └── cheating.json              # 作弊演示专用配置
├── demo.sh                        # Linux/macOS 演示脚本
├── demo.bat                       # Windows 演示脚本
├── DEMO_GUIDE.md                  # 详细演示指南
├── Cargo.toml                     # Rust 项目配置
└── README.md                      # 项目文档
```

### 核心模块说明

- **`main.rs`** - 程序入口，实现四种演示模式
- **`provider_node.rs`** - Provider节点生命周期管理和异步任务调度
- **`task_executor.rs`** - 实际的数学计算执行，支持作弊行为模拟
- **`cheating_modes.rs`** - 各种作弊行为的实现
- **`api_server.rs`** - REST API和WebSocket监控接口
- **`consensus_client.rs`** - 与共识服务器的网络通信
- **`config.rs`** - JSON配置文件解析和管理
- **`types.rs`** - 所有数据结构的定义

## 🧪 测试和开发

### 运行单元测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test task_executor

# 运行带输出信息的测试
cargo test -- --nocapture
```

### 代码质量检查

```bash
# 格式化代码
cargo fmt

# 运行 Clippy 代码检查
cargo clippy

# 完整检查流程
cargo fmt && cargo clippy && cargo test
```

### 性能分析

```bash
# 构建发布版本
cargo build --release

# 运行并分析性能
cargo run --release -- --demo basic
```

## 📋 演示指南

详细的演示使用说明请参考：

- [DEMO_GUIDE.md](DEMO_GUIDE.md) - 完整演示指南
- `demo.sh` / `demo.bat` - 自动化演示脚本

### 演示模式详解

#### 1. **基础功能演示** (`--demo basic`)
验证系统的各个核心组件是否正常工作：

**演示内容：**
- ✅ 核心类型系统验证（TaskType, ConsensusTask等）
- ✅ 配置管理系统测试（AppConfig, ProviderConfig）
- ✅ 数学计算引擎演示（矩阵乘法, 向量运算）
- ✅ 作弊行为模拟展示（延迟执行, 错误结果）
- ✅ 网络通信框架测试（HTTP客户端）
- ✅ 异步生命周期概念验证（Provider节点管理）

**特点：**
- 不需要外部共识服务器
- 完全独立运行
- 快速验证所有核心功能

#### 2. **多节点演示** (`--demo multi`)
展示多个Provider节点同时运行的分布式环境：

**演示内容：**
- 🚀 同时启动4个Provider节点（1个诚实+3个作弊）
- 🌐 每个节点提供独立的API接口（端口8081-8084）
- 📊 实时状态监控和统计信息
- 🔄 任务分发和执行准备（需要共识服务器发送任务）
- 📈 性能统计收集

**特点：**
- 展示分布式节点管理
- 提供完整的监控界面
- 可与共识服务器集成测试

#### 3. **作弊检测演示** (`--demo cheating`)
专门展示作弊行为的检测和处理机制：

**演示内容：**
- 🎭 展示各种作弊行为的执行过程
- 🔍 验证共识机制对作弊的识别能力
- 📋 演示惩罚和奖励机制
- 📝 详细的作弊行为日志记录

**特点：**
- 专注于作弊行为分析
- 展示安全机制的有效性
- 支持教学和研究用途

#### 4. **完整功能演示** (`--demo full`)
结合所有功能的多阶段完整演示：

**演示流程：**
- 📋 **第一阶段**: 基础功能测试
- 🏗️ **第二阶段**: 多节点系统演示

**特点：**
- 完整的端到端演示流程
- 涵盖所有功能模块
- 适合完整的功能展示

## 📄 许可证

本项目采用 MIT 许可证。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！这个项目实现了"去中心化GPU算力共享平台"中Provider节点的完整功能。

**贡献前请确保：**

1. ✅ 代码通过 `cargo fmt && cargo clippy && cargo test`
2. 📝 添加必要的文档和注释
3. ⚙️ 更新相关配置和示例
4. 🎯 遵循现有的代码风格
5. 🎭 测试所有四种演示模式
6. 🔧 验证API接口正常工作

**特别说明：**
- 这个项目是课程设计项目，实现了完整的Provider节点模拟器
- 支持真实的数学计算任务和多种作弊行为模拟
- 提供了完整的实验环境和演示系统

## 🔗 相关项目

- [Yagna](https://github.com/golemfactory/yagna) - 去中心化计算平台
- [Golem Network](https://golem.network/) - 去中心化超级计算机网络
- [Akash Network](https://akash.network/) - 去中心化云平台

## 🎓 学术用途

本项目可用于：

- **分布式计算研究** - 研究多节点任务执行和共识机制
- **安全机制分析** - 分析作弊行为检测和防御策略
- **性能评估** - 评估不同计算任务的执行效率
- **教学演示** - 演示去中心化计算平台的核心概念

**项目特点：**
- 🔬 **实验设计完整** - 支持可控的实验环境
- 📊 **结果分析友好** - 详细的日志和统计信息
- 🎯 **系统可展示性强** - 直观的演示界面和API

