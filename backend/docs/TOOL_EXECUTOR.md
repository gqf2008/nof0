# Tool Executor - 多轮对话工具执行器

## 概述

ToolExecutor 是 nof0 交易系统的核心组件之一，负责：
1. **执行工具调用** - 接收 LLM 返回的工具调用请求并执行对应的 MCP 工具
2. **管理多轮对话** - 自动将工具执行结果返回给 LLM，实现智能的多轮交互
3. **追踪执行历史** - 记录所有工具调用和对话历史，便于调试和审计

## 核心架构

### 1. ToolExecutor 结构

```rust
pub struct ToolExecutor {
    mcp_server: Arc<McpServer>,
    max_rounds: usize,  // 防止无限循环
}
```

### 2. 数据结构

#### ExecutionResult - 单次工具执行结果
```rust
pub struct ExecutionResult {
    pub tool_call: ToolCall,     // 工具调用信息
    pub result: String,           // 执行结果
    pub success: bool,            // 是否成功
    pub error: Option<String>,    // 错误信息
}
```

#### DialogueResult - 完整对话结果
```rust
pub struct DialogueResult {
    pub final_response: String,         // LLM 最终回复
    pub total_rounds: usize,            // 总轮数
    pub executions: Vec<ExecutionResult>,  // 所有工具执行记录
    pub message_history: Vec<Message>,  // 完整消息历史
}
```

## 核心功能

### 1. 执行单个工具

```rust
pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> ExecutionResult
```

**流程**：
1. 构造 MCP 请求
2. 调用 MCP Server
3. 解析响应
4. 返回 ExecutionResult

### 2. 执行多个工具

```rust
pub async fn execute_tool_calls(&self, tool_calls: &[ToolCall]) -> Vec<ExecutionResult>
```

顺序执行多个工具调用。

### 3. 多轮对话 (核心)

```rust
pub async fn execute_dialogue<F, Fut>(
    &self,
    initial_request: ChatRequest,
    tools: Vec<serde_json::Value>,
    chat_fn: F,
) -> Result<DialogueResult>
where
    F: Fn(ChatRequest, Vec<serde_json::Value>) -> Fut,
    Fut: std::future::Future<Output = Result<ChatResponse>>,
```

**流程**：
1. 发送初始消息给 LLM
2. 检查 LLM 响应是否包含工具调用
3. 如果有工具调用：
   - 执行所有工具
   - 将结果转换为消息
   - 添加到历史
   - 继续下一轮
4. 如果没有工具调用：
   - 返回最终结果
5. 达到最大轮数自动停止

## 使用示例

### 基础用法

```rust
use nof0_backend::engine::{TradingEngine, ToolExecutor};
use nof0_backend::llm::{ChatRequest, Message};
use std::sync::Arc;

// 创建 MCP Server 和 Trading Engine
let mcp_server = Arc::new(McpServer::new());
let engine = TradingEngine::new(mcp_server.clone());

// 创建 Tool Executor (最多 10 轮)
let executor = ToolExecutor::new(mcp_server).with_max_rounds(10);

// 准备初始请求
let request = ChatRequest {
    messages: vec![Message {
        role: "user".to_string(),
        content: "查询 BTC 价格并分析是否应该买入".to_string(),
    }],
    temperature: Some(0.7),
    max_tokens: Some(1500),
};

// 准备工具列表
let tools = vec![
    json!({
        "type": "function",
        "function": {
            "name": "get_crypto_price",
            "description": "获取加密货币价格",
            "parameters": {
                "type": "object",
                "properties": {
                    "symbol": { "type": "string" }
                },
                "required": ["symbol"]
            }
        }
    }),
    // ... 更多工具
];

// 执行多轮对话
let result = executor
    .execute_dialogue(request, tools, |req, tools| {
        let engine = &engine;
        async move {
            engine.chat_with_tools("openai", req.messages, tools).await
        }
    })
    .await?;

// 查看结果
println!("最终回复: {}", result.final_response);
println!("总轮数: {}", result.total_rounds);
println!("工具执行次数: {}", result.executions.len());
```

### 完整场景示例

参见 `backend/examples/multi_round_dialogue_demo.rs`，包含三个完整场景：

#### 场景 1: 复杂交易决策
```rust
async fn scenario_complex_trading_decision()
```
- 查询多个加密货币价格
- 获取市场分析
- 检查账户余额
- AI 推荐买入策略
- 执行订单

#### 场景 2: 错误恢复
```rust
async fn scenario_error_recovery()
```
- 处理缺少参数的订单
- AI 自动查询当前价格
- 验证账户余额
- 智能决策并执行

#### 场景 3: 投资组合分析
```rust
async fn scenario_portfolio_analysis()
```
- 查询当前持仓
- 获取所有资产价格
- 计算总价值
- 提供再平衡建议

## 运行示例

### 前置条件
```bash
# 设置 OpenAI API Key
export OPENAI_API_KEY="sk-..."

# 或者使用其他兼容 OpenAI API 的服务
export OPENAI_API_KEY="your-key"
export OPENAI_BASE_URL="https://api.deepseek.com"
```

### 运行
```bash
cd backend
cargo run --example multi_round_dialogue_demo
```

### 预期输出
```
╔══════════════════════════════════════════╗
║   Multi-round Dialogue Demo             ║
║   Showcasing Tool Executor               ║
╚══════════════════════════════════════════╝

✅ Engine initialized with OpenAI provider
✅ MCP Server with 4 tools
✅ Tool Executor ready (max 10 rounds)

=== Scenario: Complex Trading Decision ===

┌─────────────────────────────────────────┐
│       Multi-round Dialogue Result       │
└─────────────────────────────────────────┘

📊 Total Rounds: 5
🔧 Tool Executions: 8

Tool Execution #1
  ├─ Tool: get_crypto_price
  ├─ Success: ✅
  └─ Result: {"symbol":"BTC","price":67500.0,...}

... (更多工具执行)

💬 Final Response:
─────────────────────────────────────────
Based on the analysis, I recommend buying 0.15 BTC...
─────────────────────────────────────────
```

## 设计要点

### 1. 防止无限循环
```rust
if round > self.max_rounds {
    warn!("Reached maximum rounds ({}), stopping", self.max_rounds);
    break;
}
```

### 2. 完整的消息历史
每轮对话都会保存：
- 用户消息
- 助手消息（包括工具调用）
- 工具执行结果消息

### 3. 错误处理
```rust
if exec.success {
    format!("Tool '{}' executed successfully:\n{}", ...)
} else {
    format!("Tool '{}' execution failed: {}", ...)
}
```
即使工具执行失败，也会将错误信息返回给 LLM，让其可以智能恢复。

### 4. 灵活的 LLM 提供者
通过闭包参数支持任意 LLM 提供者：
```rust
|req, tools| async move {
    engine.chat_with_tools("openai", req.messages, tools).await
}
```

## 与 TradingEngine 的集成

```rust
// TradingEngine 提供统一的 LLM 调用接口
impl TradingEngine {
    pub async fn chat_with_tools(
        &self,
        provider_name: &str,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse>
}
```

ToolExecutor 使用这个接口与 LLM 交互：
1. 不需要直接管理 LLM 连接
2. 自动处理多个 LLM 提供者
3. 统一的错误处理

## 性能考虑

### 1. 异步执行
所有工具调用都是异步的：
```rust
pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> ExecutionResult
```

### 2. Arc 共享
MCP Server 使用 `Arc` 共享，避免克隆：
```rust
pub struct ToolExecutor {
    mcp_server: Arc<McpServer>,
}
```

### 3. 顺序执行工具
当前实现是顺序执行多个工具调用：
```rust
for tool_call in tool_calls {
    let result = self.execute_tool_call(tool_call).await;
    results.push(result);
}
```

**未来优化**: 可以并行执行独立的工具调用。

## 日志和调试

### 日志级别
- `info!`: 关键事件（开始对话、工具执行成功）
- `debug!`: 详细信息（工具参数、结果）
- `warn!`: 警告（工具失败、达到最大轮数）

### 启用日志
```rust
tracing_subscriber::fmt::init();
```

### 示例日志输出
```
INFO nof0_backend::engine::tool_executor: Starting dialogue with 4 available tools
INFO nof0_backend::engine::tool_executor: Dialogue round 1
INFO nof0_backend::engine::tool_executor: LLM requested 2 tool calls
INFO nof0_backend::engine::tool_executor: Executing tool: get_crypto_price
DEBUG nof0_backend::engine::tool_executor: Tool arguments: {"symbol":"BTC"}
INFO nof0_backend::engine::tool_executor: Tool execution succeeded: get_crypto_price
```

## 下一步增强

### 1. 并行工具执行
```rust
// 使用 tokio::join! 或 futures::future::join_all
let results = futures::future::join_all(
    tool_calls.iter().map(|tc| self.execute_tool_call(tc))
).await;
```

### 2. 工具执行超时
```rust
use tokio::time::{timeout, Duration};

timeout(Duration::from_secs(30), self.execute_tool_call(tool_call)).await?
```

### 3. 重试机制
```rust
for attempt in 1..=3 {
    match self.execute_tool_call(tool_call).await {
        Ok(result) => return result,
        Err(e) if attempt < 3 => {
            warn!("Attempt {} failed, retrying...", attempt);
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### 4. 持久化对话历史
将 `DialogueResult` 保存到数据库，用于：
- 审计
- 回放
- 训练数据
- 用户查询历史

### 5. 流式响应
支持 LLM 流式输出：
```rust
pub async fn execute_dialogue_stream<F>(
    &self,
    initial_request: ChatRequest,
    tools: Vec<serde_json::Value>,
    stream_fn: F,
) -> impl Stream<Item = DialogueEvent>
```

## 测试

### 单元测试
```rust
#[tokio::test]
async fn test_tool_executor_creation() {
    let mcp_server = Arc::new(McpServer::new());
    let executor = ToolExecutor::new(mcp_server);
    assert_eq!(executor.max_rounds, 10);
}

#[tokio::test]
async fn test_with_max_rounds() {
    let mcp_server = Arc::new(McpServer::new());
    let executor = ToolExecutor::new(mcp_server).with_max_rounds(5);
    assert_eq!(executor.max_rounds, 5);
}
```

### 集成测试
参见 `examples/multi_round_dialogue_demo.rs`。

## 相关文档

- [TOOL_CALLING.md](./TOOL_CALLING.md) - 工具调用完整指南
- [TRADING_ENGINE_INTEGRATION.md](./TRADING_ENGINE_INTEGRATION.md) - 交易引擎集成
- [CTP_INTEGRATION_REFERENCE.md](./CTP_INTEGRATION_REFERENCE.md) - CTP 集成参考

## 贡献者

- wquguru - 初始实现

## 更新日志

### 2025-01-29
- ✅ 初始实现
- ✅ 支持多轮对话
- ✅ 完整的错误处理
- ✅ 三个完整场景示例
- ✅ 编译通过并可运行
