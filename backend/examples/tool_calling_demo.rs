use nof0_backend::llm::{
    AnthropicProvider, ChatRequest, LlmProvider, Message, OpenAICompatibleProvider,
    RigOpenAIProvider,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Tool Calling Demo ===\n");

    // 定义一个简单的 MCP 工具：获取天气
    let weather_tool = json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get the current weather in a given location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "The temperature unit"
                    }
                },
                "required": ["location"]
            }
        }
    });

    let calculator_tool = json!({
        "type": "function",
        "function": {
            "name": "calculate",
            "description": "Perform a mathematical calculation",
            "parameters": {
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "The mathematical expression to evaluate, e.g. '2 + 2'"
                    }
                },
                "required": ["expression"]
            }
        }
    });

    let tools = vec![weather_tool, calculator_tool];

    // 示例 1: OpenAI with Tool Calling
    println!("--- OpenAI GPT-4 with Tools ---");
    demo_openai_tools(&tools).await?;

    println!("\n--- Anthropic Claude with Tools ---");
    demo_anthropic_tools(&tools).await?;

    println!("\n--- DeepSeek with Tools ---");
    demo_deepseek_tools(&tools).await?;

    Ok(())
}

async fn demo_openai_tools(tools: &[serde_json::Value]) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "your-api-key".to_string());

    let provider = RigOpenAIProvider::new(api_key, "gpt-4")?;

    let request = ChatRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant with access to tools.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "What's the weather like in San Francisco? And what's 15 * 23?"
                    .to_string(),
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(500),
    };

    println!("Sending request to OpenAI...");
    let response = provider.chat_with_tools(request, tools.to_vec()).await?;

    println!("Response: {}", response.content);
    if let Some(tool_calls) = response.tool_calls {
        println!("\nTool Calls:");
        for call in tool_calls {
            println!("  - {} ({}): {}", call.name, call.id, call.arguments);
        }
    }
    println!(
        "Tokens: {} in, {} out, {} total",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );

    Ok(())
}

async fn demo_anthropic_tools(
    tools: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "your-api-key".to_string());

    // Anthropic 使用不同的工具格式
    let anthropic_tools: Vec<serde_json::Value> = tools
        .iter()
        .map(|tool| {
            let func = &tool["function"];
            json!({
                "name": func["name"],
                "description": func["description"],
                "input_schema": func["parameters"]
            })
        })
        .collect();

    let provider = AnthropicProvider::new(api_key, "claude-3-sonnet-20240229")?;

    let request = ChatRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant with access to tools.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "I need the weather in Tokyo and also calculate 100 / 4".to_string(),
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(500),
    };

    println!("Sending request to Anthropic...");
    let response = provider.chat_with_tools(request, anthropic_tools).await?;

    println!("Response: {}", response.content);
    if let Some(tool_calls) = response.tool_calls {
        println!("\nTool Calls:");
        for call in tool_calls {
            println!("  - {} ({}): {}", call.name, call.id, call.arguments);
        }
    }
    println!(
        "Tokens: {} in, {} out, {} total",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );

    Ok(())
}

async fn demo_deepseek_tools(
    tools: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "your-api-key".to_string());

    let provider = OpenAICompatibleProvider::deepseek(api_key, "deepseek-chat")?;

    let request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: "Calculate 42 * 17 for me please".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(200),
    };

    println!("Sending request to DeepSeek...");
    let response = provider.chat_with_tools(request, tools.to_vec()).await?;

    println!("Response: {}", response.content);
    if let Some(tool_calls) = response.tool_calls {
        println!("\nTool Calls:");
        for call in tool_calls {
            println!("  - {} ({}): {}", call.name, call.id, call.arguments);
        }
    }
    println!(
        "Tokens: {} in, {} out, {} total",
        response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens
    );

    Ok(())
}
