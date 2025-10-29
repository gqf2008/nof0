# Trading Engine Integration

## 概述

成功将 LLM Providers 集成到 TradingEngine，实现了 AI 驱动的交易助手功能。

## 架构设计

### 核心组件

```
TradingEngine
├── MCP Server (工具管理)
├── LLM Providers (AI 能力)
│   ├── OpenAI GPT-4
│   ├── OpenAI GPT-3.5
│   ├── Anthropic Claude
│   └── OpenAI Compatible (DeepSeek, Qwen)
└── Market Adapters (市场接口)
    ├── Binance
    ├── OKX
    └── Custom exchanges
```

### 线程安全设计

使用 `Arc<RwLock<HashMap>>` 实现多线程安全的 Provider 管理：

```rust
pub struct TradingEngine {
    mcp_server: Arc<McpServer>,
    llm_providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    markets: Arc<RwLock<HashMap<String, Arc<dyn MarketAdapter>>>>,
}
```

**设计要点**：
- `Arc` - 允许跨线程共享
- `RwLock` - 读写锁，支持多读单写
- Provider 使用 `Arc<dyn Trait>` 而非 `Box<dyn Trait>`，支持克隆

## API 使用

### 初始化 Trading Engine

```rust
use nof0_backend::engine::TradingEngine;
use nof0_backend::llm::RigOpenAIProvider;
use nof0_backend::mcp::McpServer;
use std::sync::Arc;

// 1. 创建 MCP Server
let mcp_server = Arc::new(McpServer::new());

// 2. 创建 Trading Engine
let engine = TradingEngine::new(mcp_server);

// 3. 注册 LLM Provider
let provider = RigOpenAIProvider::new(api_key, "gpt-4")?;
engine.register_llm_provider("gpt4".to_string(), Arc::new(provider)).await;
```

### 注册多个 Providers

```rust
// GPT-4 for complex analysis
let gpt4 = RigOpenAIProvider::new(openai_key.clone(), "gpt-4")?;
engine.register_llm_provider("gpt4".to_string(), Arc::new(gpt4)).await;

// GPT-3.5 for quick queries
let gpt35 = RigOpenAIProvider::new(openai_key.clone(), "gpt-3.5-turbo")?;
engine.register_llm_provider("gpt35".to_string(), Arc::new(gpt35)).await;

// Claude for reasoning
let claude = AnthropicProvider::new(anthropic_key, "claude-3-sonnet-20240229")?;
engine.register_llm_provider("claude".to_string(), Arc::new(claude)).await;

// DeepSeek for cost-effective queries
let deepseek = OpenAICompatibleProvider::deepseek(deepseek_key, "deepseek-chat")?;
engine.register_llm_provider("deepseek".to_string(), Arc::new(deepseek)).await;
```

### 简单对话

```rust
let response = engine
    .simple_chat("gpt4", "What's the capital of France?")
    .await?;

println!("Response: {}", response);
```

### 使用工具的对话

```rust
use nof0_backend::llm::Message;
use serde_json::json;

// 定义工具
let tools = vec![json!({
    "type": "function",
    "function": {
        "name": "get_crypto_price",
        "description": "Get cryptocurrency price",
        "parameters": {
            "type": "object",
            "properties": {
                "symbol": {"type": "string"}
            },
            "required": ["symbol"]
        }
    }
})];

// 构建消息
let messages = vec![
    Message {
        role: "system".to_string(),
        content: "You are a trading assistant.".to_string(),
    },
    Message {
        role: "user".to_string(),
        content: "What's Bitcoin's price?".to_string(),
    },
];

// 执行
let response = engine
    .chat_with_tools("gpt4", messages, tools)
    .await?;

// 处理结果
if let Some(tool_calls) = response.tool_calls {
    for call in tool_calls {
        println!("Tool: {} - Args: {}", call.name, call.arguments);
        // 执行工具并获取结果...
    }
}
```

## 核心方法

### 管理方法

```rust
// 注册 Provider
async fn register_llm_provider(&self, name: String, provider: Arc<dyn LlmProvider>)

// 注册 Market Adapter
async fn register_market(&self, name: String, adapter: Arc<dyn MarketAdapter>)

// 获取 Provider
async fn get_llm_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>>

// 获取 Market Adapter
async fn get_market(&self, name: &str) -> Option<Arc<dyn MarketAdapter>>

// 列出所有 Providers
async fn list_llm_providers(&self) -> Vec<String>

// 列出所有 Markets
async fn list_markets(&self) -> Vec<String>
```

### 执行方法

```rust
// 简单对话
async fn simple_chat(&self, provider_name: &str, message: &str) -> Result<String>

// 带工具的对话
async fn chat_with_tools(
    &self,
    provider_name: &str,
    messages: Vec<Message>,
    tools: Vec<serde_json::Value>
) -> Result<ChatResponse>

// 运行引擎
async fn run(&self) -> Result<(), anyhow::Error>
```

## 示例程序

### 1. trading_engine_demo.rs

基础功能演示：
- 创建和配置 Trading Engine
- 注册 LLM Provider
- 简单对话测试
- Tool Calling 测试

运行：
```bash
export OPENAI_API_KEY="sk-..."
cargo run --example trading_engine_demo
```

### 2. trading_assistant_demo.rs

实际应用场景演示：
- 价格查询场景
- 交易决策场景
- 投资组合分析场景
- 多种 MCP 工具定义

运行：
```bash
export OPENAI_API_KEY="sk-..."
cargo run --example trading_assistant_demo
```

输出示例：
```
=== Trading Assistant Demo ===

✓ MCP Server created
✓ Trading Engine created
✓ Registered GPT-4 provider
✓ Registered GPT-3.5 provider

=== Scenario 1: Price Query ===
🤖 Assistant: I'll check the current prices for you...
🔧 Tool Calls:
   → get_crypto_price with args: {"symbol": "BTC"}
   → get_crypto_price with args: {"symbol": "ETH"}
📊 Tokens used: 45/120

=== Scenario 2: Trading Decision ===
🤖 Assistant: Based on the market analysis...
🔧 Tool Calls:
   → get_crypto_price with args: {"symbol": "SOL"}
   → get_market_analysis with args: {"symbol": "SOL", "timeframe": "1d"}
📊 Tokens used: 78/245
```

## MCP 工具定义

### 价格查询工具

```json
{
  "type": "function",
  "function": {
    "name": "get_crypto_price",
    "description": "Get the current price of a cryptocurrency in USD",
    "parameters": {
      "type": "object",
      "properties": {
        "symbol": {
          "type": "string",
          "description": "Crypto symbol (e.g., BTC, ETH, SOL)"
        }
      },
      "required": ["symbol"]
    }
  }
}
```

### 账户余额工具

```json
{
  "type": "function",
  "function": {
    "name": "get_account_balance",
    "description": "Get the current balance of a trading account",
    "parameters": {
      "type": "object",
      "properties": {
        "account_id": {
          "type": "string",
          "description": "The account ID"
        }
      },
      "required": ["account_id"]
    }
  }
}
```

### 下单工具

```json
{
  "type": "function",
  "function": {
    "name": "place_order",
    "description": "Place a trading order",
    "parameters": {
      "type": "object",
      "properties": {
        "symbol": {"type": "string"},
        "side": {"type": "string", "enum": ["buy", "sell"]},
        "amount": {"type": "number"}
      },
      "required": ["symbol", "side", "amount"]
    }
  }
}
```

### 市场分析工具

```json
{
  "type": "function",
  "function": {
    "name": "get_market_analysis",
    "description": "Get technical analysis and market sentiment",
    "parameters": {
      "type": "object",
      "properties": {
        "symbol": {"type": "string"},
        "timeframe": {
          "type": "string",
          "enum": ["1h", "4h", "1d", "1w"]
        }
      },
      "required": ["symbol"]
    }
  }
}
```

## 工作流程

### 完整的 Agent 执行流程

```
1. User Input
   ↓
2. Trading Engine
   ↓
3. Select LLM Provider (gpt4/claude/deepseek)
   ↓
4. Construct Messages + Tools
   ↓
5. LLM Processing
   ↓
6. Tool Calls? 
   ├─ Yes → Execute MCP Tools
   │         ↓
   │       Get Results
   │         ↓
   │       Send back to LLM
   │         ↓
   └─ No → Return Response
             ↓
7. Execute Trading Decision
   ↓
8. Log & Monitor
```

## 性能优化

### Provider 选择策略

```rust
// 复杂分析 - 使用 GPT-4
if task.complexity == High {
    engine.chat_with_tools("gpt4", messages, tools).await?
}

// 快速查询 - 使用 GPT-3.5
else if task.latency_sensitive {
    engine.chat_with_tools("gpt35", messages, tools).await?
}

// 成本优化 - 使用 DeepSeek
else if task.cost_sensitive {
    engine.chat_with_tools("deepseek", messages, tools).await?
}

// 推理任务 - 使用 Claude
else if task.reasoning_required {
    engine.chat_with_tools("claude", messages, tools).await?
}
```

### 并发处理

```rust
use futures::future::join_all;

// 并发查询多个市场
let futures = symbols.iter().map(|symbol| {
    let engine = engine.clone();
    async move {
        engine.simple_chat("gpt35", &format!("Analyze {}", symbol)).await
    }
});

let results = join_all(futures).await;
```

## 错误处理

```rust
match engine.chat_with_tools("gpt4", messages, tools).await {
    Ok(response) => {
        // 处理成功响应
        handle_response(response).await?;
    }
    Err(e) => {
        // 记录错误
        tracing::error!("LLM call failed: {}", e);
        
        // 尝试备用 Provider
        if let Ok(response) = engine.chat_with_tools("gpt35", messages, tools).await {
            handle_response(response).await?;
        } else {
            // 返回默认响应
            return Ok(default_response());
        }
    }
}
```

## 监控和日志

引擎会自动记录：
- Provider 注册事件
- API 调用
- Token 使用情况
- 工具调用
- 错误信息

查看日志：
```bash
RUST_LOG=info cargo run --example trading_assistant_demo
RUST_LOG=debug cargo run --example trading_assistant_demo
```

## 下一步

- [ ] 实现工具执行器 (Tool Executor)
- [ ] 添加工具结果缓存
- [ ] 实现多轮对话管理
- [ ] 添加 Agent 持久化
- [ ] 实现决策审计日志
- [ ] 添加风险控制规则
- [ ] 实现回测功能
- [ ] 添加性能监控面板

## 测试

```bash
# 运行所有测试
cargo test

# 运行 Engine 测试
cargo test engine::

# 运行示例
cargo run --example trading_engine_demo
cargo run --example trading_assistant_demo
```

## 许可证

MIT OR Apache-2.0
