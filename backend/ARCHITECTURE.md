# NOF0 Backend (Rust) - 架构设计文档

> **版本**: v2.0  
> **语言**: Rust  
> **框架**: Axum + Tokio  
> **更新日期**: 2025-10-28

---

## 概述

NOF0 Backend 是一个基于 Rust 的高性能 AI 量化交易后端，支持：

- ✅ **MCP 协议** - Model Context Protocol 标准实现
- ✅ **多 LLM 集成** - OpenAI, Anthropic, DeepSeek, 通义千问等
- ✅ **多市场支持** - 数字货币、A股、美股、港股、期货
- ✅ **实时交易引擎** - 异步、高并发、低延迟
- ✅ **WebSocket 推送** - 实时数据更新

## 技术栈

| 组件 | 技术选型 | 版本 |
|------|---------|------|
| **Web框架** | Axum | 0.7 |
| **异步运行时** | Tokio | 1.40 |
| **HTTP客户端** | Reqwest | 0.12 |
| **WebSocket** | Tokio-Tungstenite | - |
| **序列化** | Serde | 1.0 |
| **数据库** | SQLx + PostgreSQL | - |
| **缓存** | Redis (fred) | - |
| **日志** | Tracing | 0.1 |
| **错误处理** | Anyhow / Thiserror | 1.0 |

---

## 项目结构

```
backend/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.rs              # 入口文件
    ├── config.rs            # 配置管理
    ├── error.rs             # 错误定义
    ├── mcp/                 # MCP 协议实现
    │   ├── mod.rs
    │   ├── server.rs        # MCP Server
    │   ├── transport.rs     # 传输层 (Stdio/SSE/WebSocket)
    │   ├── tools.rs         # MCP Tools 定义
    │   └── types.rs         # MCP 类型定义
    ├── llm/                 # LLM 适配层
    │   ├── mod.rs
    │   ├── provider.rs      # Provider trait
    │   ├── openai.rs        # OpenAI 实现
    │   ├── anthropic.rs     # Anthropic 实现
    │   ├── deepseek.rs      # DeepSeek 实现
    │   └── qwen.rs          # 通义千问实现
    ├── markets/             # 市场适配层
    │   ├── mod.rs
    │   ├── adapter.rs       # Adapter trait
    │   ├── crypto/          # 数字货币
    │   │   ├── mod.rs
    │   │   └── binance.rs
    │   ├── stock/           # 股票
    │   │   ├── mod.rs
    │   │   ├── a_share.rs   # A股
    │   │   └── us_stock.rs  # 美股
    │   └── futures/         # 期货
    │       ├── mod.rs
    │       └── ctp.rs
    ├── engine/              # 交易引擎
    │   ├── mod.rs
    │   ├── trading.rs       # 交易主循环
    │   ├── scheduler.rs     # 调度器
    │   ├── executor.rs      # 订单执行器
    │   └── agent.rs         # AI Agent 管理
    ├── portfolio/           # 账户管理
    │   ├── mod.rs
    │   ├── account.rs       # 账户
    │   ├── position.rs      # 持仓
    │   └── pnl.rs           # 损益计算
    ├── db/                  # 数据库层
    │   ├── mod.rs
    │   ├── pool.rs          # 连接池
    │   ├── models.rs        # 数据模型
    │   └── repositories/    # Repository 层
    │       ├── mod.rs
    │       ├── agents.rs
    │       ├── trades.rs
    │       ├── positions.rs
    │       └── conversations.rs
    ├── api/                 # REST API
    │   ├── mod.rs
    │   ├── routes.rs        # 路由定义
    │   ├── handlers/        # 处理器
    │   │   ├── mod.rs
    │   │   ├── prices.rs
    │   │   ├── trades.rs
    │   │   ├── positions.rs
    │   │   └── agents.rs
    │   └── ws.rs            # WebSocket 处理
    └── utils/               # 工具函数
        ├── mod.rs
        ├── time.rs
        └── format.rs
```

---

## 核心模块设计

### 1. MCP 协议层 (`src/mcp/`)

```rust
// src/mcp/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// src/mcp/server.rs
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error>;
}

pub struct McpServer {
    tools: HashMap<String, Box<dyn ToolHandler>>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    pub fn register_tool(&mut self, name: String, handler: Box<dyn ToolHandler>) {
        self.tools.insert(name, handler);
    }
    
    pub async fn handle_request(&self, msg: McpMessage) -> McpMessage {
        // 处理 MCP 请求
        // tools/list, tools/call, resources/list, etc.
        todo!()
    }
}
```

### 2. LLM 适配层 (`src/llm/`)

```rust
// src/llm/provider.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: TokenUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, anyhow::Error>;
    
    async fn chat_with_tools(
        &self,
        req: ChatRequest,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse, anyhow::Error>;
    
    fn name(&self) -> &str;
    fn model(&self) -> &str;
}
```

### 3. 市场适配层 (`src/markets/`)

```rust
// src/markets/adapter.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[async_trait]
pub trait MarketAdapter: Send + Sync {
    async fn get_price(&self, symbol: &str) -> Result<Price, anyhow::Error>;
    
    async fn place_order(&self, order: Order) -> Result<String, anyhow::Error>;
    
    async fn get_balance(&self, account_id: &str) -> Result<f64, anyhow::Error>;
    
    fn market_name(&self) -> &str;
}
```

### 4. 交易引擎 (`src/engine/`)

```rust
// src/engine/trading.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TradingEngine {
    mcp_server: Arc<McpServer>,
    llm_providers: HashMap<String, Box<dyn LlmProvider>>,
    markets: HashMap<String, Box<dyn MarketAdapter>>,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl TradingEngine {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        // 1. 启动 MCP Server
        // 2. 启动调度器
        // 3. 启动价格订阅
        // 4. 主循环
        todo!()
    }
    
    async fn execute_agent(&self, agent: &Agent) -> Result<(), anyhow::Error> {
        // 1. 获取市场数据
        // 2. 构建 Prompt
        // 3. 调用 LLM
        // 4. 解析决策
        // 5. 执行交易
        // 6. 记录结果
        todo!()
    }
}
```

---

## 依赖项更新

```toml
[dependencies]
# 现有依赖
anyhow = "1.0"
axum = { version = "0.7", features = ["macros", "ws"] }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# 新增依赖
async-trait = "0.1"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }

# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "json"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# WebSocket
tokio-tungstenite = "0.21"

# 配置
config = "0.14"
dotenvy = "0.15"

# 加密
sha2 = "0.10"
hmac = "0.12"
base64 = "0.21"

# 定时任务
tokio-cron-scheduler = "0.9"
```

---

## 配置文件

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 8788

[database]
url = "postgres://nof0:nof0@localhost:5432/nof0"
max_connections = 10

[redis]
url = "redis://localhost:6379"

[[llm_providers]]
name = "openai"
model = "gpt-4o"
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"

[[llm_providers]]
name = "anthropic"
model = "claude-3-5-sonnet-20241022"
api_key = "${ANTHROPIC_API_KEY}"

[[markets]]
name = "binance"
type = "crypto"
symbols = ["BTC/USDT", "ETH/USDT"]
enabled = true
```

---

## 开发路线图

### Phase 1: 基础设施 (1周)
- [ ] 项目结构搭建
- [ ] 配置管理
- [ ] 错误处理
- [ ] 日志系统
- [ ] 数据库连接

### Phase 2: MCP 实现 (2周)
- [ ] MCP Server 核心
- [ ] Stdio Transport
- [ ] Tools 定义
- [ ] 测试用例

### Phase 3: LLM 集成 (1周)
- [ ] Provider trait
- [ ] OpenAI 实现
- [ ] Anthropic 实现
- [ ] DeepSeek 实现

### Phase 4: 市场适配 (2周)
- [ ] Adapter trait
- [ ] Binance 实现
- [ ] 价格订阅
- [ ] 订单执行

### Phase 5: 交易引擎 (2周)
- [ ] 调度器
- [ ] Agent 管理
- [ ] 交易逻辑
- [ ] 结果记录

### Phase 6: API 完善 (1周)
- [ ] REST endpoints
- [ ] WebSocket 推送
- [ ] 前后端联调

---

## 下一步

1. 创建模块目录结构
2. 添加依赖到 Cargo.toml
3. 实现核心 trait
4. 从 MCP Server 开始实现

准备好开始了吗？
