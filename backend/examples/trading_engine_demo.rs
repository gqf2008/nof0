use nof0_backend::engine::TradingEngine;
use nof0_backend::llm::{Message, RigOpenAIProvider};
use nof0_backend::mcp::McpServer;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Trading Engine Demo ===\n");

    // 1. 创建 MCP Server
    let mcp_server = Arc::new(McpServer::new());
    println!("✓ MCP Server created");

    // 2. 创建 Trading Engine
    let engine = TradingEngine::new(mcp_server);
    println!("✓ Trading Engine created");

    // 3. 注册 LLM Providers
    let openai_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "your-openai-key".to_string());
    
    let openai_provider = RigOpenAIProvider::new(openai_key, "gpt-4")?;
    engine
        .register_llm_provider("openai-gpt4".to_string(), Arc::new(openai_provider))
        .await;
    println!("✓ Registered OpenAI GPT-4 provider");

    // 4. 列出已注册的 Providers
    let providers = engine.list_llm_providers().await;
    println!("\nRegistered Providers: {:?}", providers);

    // 5. 测试简单的聊天
    println!("\n--- Simple Chat Test ---");
    match engine
        .simple_chat("openai-gpt4", "What is the capital of France?")
        .await
    {
        Ok(response) => println!("Response: {}", response),
        Err(e) => println!("Error: {}", e),
    }

    // 6. 测试带工具的聊天
    println!("\n--- Chat with Tools Test ---");
    
    let tools = vec![json!({
        "type": "function",
        "function": {
            "name": "get_crypto_price",
            "description": "Get the current price of a cryptocurrency",
            "parameters": {
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The crypto symbol, e.g. BTC, ETH"
                    }
                },
                "required": ["symbol"]
            }
        }
    })];

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a trading assistant with access to market data tools."
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "What's the current price of Bitcoin?".to_string(),
        },
    ];

    match engine
        .chat_with_tools("openai-gpt4", messages, tools)
        .await
    {
        Ok(response) => {
            println!("Response: {}", response.content);
            if let Some(tool_calls) = response.tool_calls {
                println!("\nTool Calls:");
                for call in tool_calls {
                    println!("  - {} ({}): {}", call.name, call.id, call.arguments);
                }
            }
            println!(
                "\nTokens: {} in, {} out",
                response.usage.prompt_tokens, response.usage.completion_tokens
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // 7. 启动引擎（目前只是演示）
    println!("\n--- Starting Trading Engine ---");
    engine.run().await?;

    println!("\n✓ Demo completed successfully!");

    Ok(())
}
