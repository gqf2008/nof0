# LLM Tool Calling 实现

## 概述

成功实现了三个 LLM Provider 的 Tool Calling 功能：

1. **RigOpenAIProvider** - OpenAI GPT-4, GPT-3.5-turbo
2. **AnthropicProvider** - Claude 3 (Opus, Sonnet, Haiku)
3. **OpenAICompatibleProvider** - DeepSeek, Qwen, 自定义端点

## 功能特性

### ✅ 基础功能
- [x] 标准聊天 API (`chat`)
- [x] Tool Calling 支持 (`chat_with_tools`)
- [x] Token 使用统计
- [x] 温度和 max_tokens 参数
- [x] 完整的错误处理
- [x] 异步 API 设计

### ✅ Tool Calling 实现
- [x] OpenAI Function Calling 格式
- [x] Anthropic Tool Use 格式
- [x] 工具结果解析和提取
- [x] 多工具调用支持
- [x] 统一的 ToolCall 接口

## API 使用示例

### OpenAI Tool Calling

```rust
use nof0_backend::llm::{RigOpenAIProvider, ChatRequest, Message, LlmProvider};
use serde_json::json;

// 定义工具
let tool = json!({
    "type": "function",
    "function": {
        "name": "get_weather",
        "description": "Get weather for a location",
        "parameters": {
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        }
    }
});

// 创建 Provider
let provider = RigOpenAIProvider::new(api_key, "gpt-4")?;

// 发送请求
let request = ChatRequest {
    messages: vec![
        Message {
            role: "user".to_string(),
            content: "What's the weather in Tokyo?".to_string(),
        },
    ],
    temperature: Some(0.7),
    max_tokens: Some(500),
};

let response = provider.chat_with_tools(request, vec![tool]).await?;

// 处理工具调用
if let Some(tool_calls) = response.tool_calls {
    for call in tool_calls {
        println!("Tool: {} - Args: {}", call.name, call.arguments);
        // 执行工具并返回结果...
    }
}
```

### Anthropic Tool Calling

```rust
use nof0_backend::llm::{AnthropicProvider, ChatRequest, LlmProvider};

// Anthropic 使用不同的工具格式
let tool = json!({
    "name": "get_weather",
    "description": "Get weather for a location",
    "input_schema": {
        "type": "object",
        "properties": {
            "location": {"type": "string"}
        },
        "required": ["location"]
    }
});

let provider = AnthropicProvider::new(api_key, "claude-3-sonnet-20240229")?;
let response = provider.chat_with_tools(request, vec![tool]).await?;
```

### OpenAI Compatible Providers

```rust
use nof0_backend::llm::OpenAICompatibleProvider;

// DeepSeek
let provider = OpenAICompatibleProvider::deepseek(api_key, "deepseek-chat")?;

// Qwen
let provider = OpenAICompatibleProvider::qwen(api_key, "qwen-turbo")?;

// 自定义端点
let provider = OpenAICompatibleProvider::custom(
    api_key,
    "custom-model",
    "https://api.custom.com/v1".to_string(),
    "custom-provider".to_string()
)?;
```

## 数据结构

### ChatRequest
```rust
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
```

### ChatResponse
```rust
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: TokenUsage,
}
```

### ToolCall
```rust
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}
```

### TokenUsage
```rust
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## Provider 特性对比

| 特性 | OpenAI | Anthropic | OpenAI Compatible |
|-----|--------|-----------|-------------------|
| Function Calling | ✅ | ✅ | ✅ |
| System Messages | ✅ | ✅ (单独字段) | ✅ |
| Streaming | ❌ | ❌ | ❌ |
| Vision | ❌ | ❌ | ❌ |
| JSON Mode | ❌ | ❌ | ❌ |

## 工具定义格式

### OpenAI 格式
```json
{
  "type": "function",
  "function": {
    "name": "tool_name",
    "description": "Tool description",
    "parameters": {
      "type": "object",
      "properties": {...},
      "required": [...]
    }
  }
}
```

### Anthropic 格式
```json
{
  "name": "tool_name",
  "description": "Tool description",
  "input_schema": {
    "type": "object",
    "properties": {...},
    "required": [...]
  }
}
```

## 运行示例

```bash
# 设置 API Keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export DEEPSEEK_API_KEY="sk-..."

# 运行基础示例
cargo run --example llm_demo

# 运行 Tool Calling 示例
cargo run --example tool_calling_demo
```

## 实现细节

### OpenAI Implementation
- 使用 `chat/completions` 端点
- 支持 `tools` 和 `tool_choice` 参数
- 返回 `tool_calls` 数组包含函数调用信息
- Tool choice 默认为 "auto"

### Anthropic Implementation  
- 使用 `messages` API v1
- System message 作为单独的 `system` 参数
- 工具响应在 `content` 数组中，类型为 `tool_use`
- 需要 `max_tokens` 参数（默认 4096）

### OpenAI Compatible Implementation
- 复用 OpenAI 的请求/响应格式
- 支持任意 OpenAI 兼容端点
- 预定义工厂方法：`deepseek()`, `qwen()`, `custom()`

## 错误处理

所有 Provider 都实现了统一的错误处理：

```rust
if !response.status().is_success() {
    let status = response.status();
    let error_text = response.text().await?;
    anyhow::bail!("API error ({}): {}", status, error_text);
}
```

## 下一步

- [ ] 实现 Streaming 支持
- [ ] 添加 Vision 能力 (GPT-4V, Claude 3)
- [ ] 实现 JSON Mode
- [ ] 添加重试逻辑和速率限制
- [ ] 实现工具结果反馈循环
- [ ] 集成到 TradingEngine
- [ ] 添加 Google Gemini Provider
- [ ] 添加 Cohere Provider

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test llm::

# 运行集成测试
cargo test --test integration_test
```

## 许可证

MIT OR Apache-2.0
