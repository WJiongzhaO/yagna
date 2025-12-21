# Consensus Provider 模拟器

去中心化 GPU 算力共享平台的 Provider 节点模拟器，用于演示多节点算力任务验证共识机制。

## 功能特性

- **多节点模拟**: 同时运行多个 Provider 节点，支持诚实节点和各种作弊节点
- **真实计算任务**: 支持矩阵乘法、向量运算和神经网络推理等实际计算任务
- **作弊行为模拟**: 内置多种作弊模式，包括延迟执行、返回错误结果、提前终止等
- **共识验证**: 与共识服务器通信，参与结果验证和奖励/惩罚机制
- **实时监控**: 提供 REST API 和 WebSocket 接口用于实时状态监控
- **详细日志**: 支持多种日志格式和轮转配置

## 快速开始

### 1. 编译项目

```bash
# 确保在 yagna 项目根目录
cd consensus-provider
cargo build --release
```

### 2. 配置节点

编辑 `config/default.json` 文件来配置 Provider 节点：

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
      "name": "诚实节点",
      "cheating_mode": null,
      "api_port": 8081,
      "gpu_memory_gb": 8.0,
      "cpu_cores": 4
    },
    {
      "node_id": "provider-2-cheater",
      "name": "作弊节点（错误结果）",
      "cheating_mode": "WrongResult",
      "api_port": 8082,
      "gpu_memory_gb": 8.0,
      "cpu_cores": 4
    }
  ]
}
```

### 3. 运行模拟器

```bash
# 启动所有启用的 Provider 节点
cargo run --release

# 启动特定的 Provider 节点
cargo run --release -- -p provider-1 -p provider-2

# 查看可用选项
cargo run --release -- --help
```

### 4. 监控节点状态

打开浏览器访问 Provider 的 API 端口：

- http://localhost:8081/health - 健康检查
- http://localhost:8081/providers - 查看所有 Provider 状态
- http://localhost:8081/stats - 系统统计信息

## 配置说明

### Provider 节点配置

每个 Provider 节点可以配置以下参数：

| 参数 | 类型 | 描述 |
|------|------|------|
| `node_id` | string | 节点唯一标识符 |
| `name` | string | 显示名称 |
| `enabled` | boolean | 是否启用节点 |
| `cheating_mode` | enum | 作弊模式（见下表） |
| `api_port` | number | API 服务器端口 |
| `gpu_memory_gb` | number | GPU 内存大小（GB） |
| `cpu_cores` | number | CPU 核心数 |

### 作弊模式

| 模式 | 描述 | 配置示例 |
|------|------|----------|
| `null` | 诚实节点 | `"cheating_mode": null` |
| `WrongResult` | 返回错误结果 | `"cheating_mode": "WrongResult"` |
| `Delay` | 延迟执行 | `"cheating_mode": {"Delay": {"min_ms": 1000, "max_ms": 5000}}` |
| `EarlyTermination` | 提前终止 | `"cheating_mode": {"EarlyTermination": {"probability": 0.3}}` |
| `Mixed` | 混合作弊 | `"cheating_mode": {"Mixed": {"weights": {...}}}` |
| `Smart` | 智能作弊 | `"cheating_mode": {"Smart": {"config": {...}}}` |

### 任务类型配置

```json
{
  "task_types": {
    "matrix_multiplication": {
      "default_size": 100,
      "max_size": 1000,
      "min_size": 10,
      "enabled": true
    },
    "vector_addition": {
      "default_size": 1000,
      "max_size": 10000,
      "min_size": 100,
      "enabled": true
    },
    "simple_inference": {
      "default_size": 50,
      "max_size": 500,
      "min_size": 10,
      "enabled": true
    }
  }
}
```

## API 接口

### REST API

#### GET /health
获取节点健康状态。

**响应示例:**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "providers_count": 4,
  "total_tasks": 150,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

#### GET /providers
获取所有 Provider 节点状态。

#### GET /providers/{id}
获取特定 Provider 节点信息。

#### GET /providers/{id}/tasks
获取 Provider 的任务历史。

#### GET /stats
获取系统统计信息。

#### GET /consensus/stats
获取共识统计信息。

### WebSocket

#### 连接地址
`ws://localhost:{port}/ws`

#### 消息格式
```json
{
  "type": "initial_state",
  "providers": [...],
  "total_tasks": 150,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## 命令行选项

```bash
Consensus Provider 模拟器 v0.1.0

用法: consensus-provider [选项]

选项:
  -c, --config <FILE>          指定配置文件路径 [默认: config/default.json]
  -p, --providers <PROVIDER_ID>...    指定要启动的 Provider 节点ID
      --list-providers          列出所有可用的 Provider 节点
      --validate-config         验证配置文件
  -v, --version                 显示版本信息
  -h, --help                    显示帮助信息
```

## 架构说明

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

## 开发说明

### 项目结构

```
consensus-provider/
├── src/
│   ├── main.rs                    # 主程序入口
│   ├── lib.rs                     # 库入口
│   ├── provider_node.rs           # Provider 节点核心
│   ├── task_executor.rs           # 任务执行器
│   ├── cheating_modes.rs          # 作弊行为模拟
│   ├── api_server.rs             # API 服务器
│   ├── consensus_client.rs        # 共识客户端
│   ├── logging.rs                 # 日志系统
│   ├── config.rs                  # 配置管理
│   └── types.rs                   # 类型定义
├── config/
│   └── default.json               # 默认配置
├── Cargo.toml                     # 项目配置
└── README.md                      # 文档
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_matrix_multiplication

# 运行文档测试
cargo test --doc
```

### 代码格式化和检查

```bash
# 格式化代码
cargo fmt

# 运行 Clippy 检查
cargo clippy

# 运行所有检查
cargo fmt && cargo clippy && cargo test
```

## 许可证

本项目采用 MIT 或 Apache-2.0 许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！请确保：

1. 代码通过了所有测试和检查
2. 添加了必要的文档和注释
3. 更新了相关的配置和示例

## 相关项目

- [Yagna](https://github.com/golemfactory/yagna) - 去中心化计算平台
- [Golem Network](https://golem.network/) - 去中心化超级计算机网络

