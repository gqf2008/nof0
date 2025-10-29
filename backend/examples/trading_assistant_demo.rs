use nof0_backend::engine::TradingEngine;
use nof0_backend::llm::{Message, RigOpenAIProvider};
use nof0_backend::mcp::McpServer;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Trading Assistant Demo ===\n");

    // 1. 创建 MCP Server 并注册工具
    let mcp_server = Arc::new(McpServer::new());
    println!("✓ MCP Server created");

    // 2. 创建 Trading Engine
    let engine = Arc::new(TradingEngine::new(mcp_server.clone()));
    println!("✓ Trading Engine created");

    // 3. 注册 LLM Providers
    let openai_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  OPENAI_API_KEY not set, using placeholder");
            "sk-placeholder-key".to_string()
        });

    let openai_provider = RigOpenAIProvider::new(openai_key.clone(), "gpt-4")?;
    engine
        .register_llm_provider("gpt4".to_string(), Arc::new(openai_provider))
        .await;
    println!("✓ Registered GPT-4 provider");

    // 也可以注册 GPT-3.5 作为备用
    let gpt35_provider = RigOpenAIProvider::new(openai_key, "gpt-3.5-turbo")?;
    engine
        .register_llm_provider("gpt35".to_string(), Arc::new(gpt35_provider))
        .await;
    println!("✓ Registered GPT-3.5 provider");

    // 4. 定义交易相关的 MCP 工具
    let trading_tools = vec![
        // 获取加密货币价格
        json!({
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
        }),
        // 获取账户余额
        json!({
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
        }),
        // 下单
        json!({
            "type": "function",
            "function": {
                "name": "place_order",
                "description": "Place a trading order",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Trading pair (e.g., BTC/USD)"
                        },
                        "side": {
                            "type": "string",
                            "enum": ["buy", "sell"],
                            "description": "Order side"
                        },
                        "amount": {
                            "type": "number",
                            "description": "Amount to trade"
                        }
                    },
                    "required": ["symbol", "side", "amount"]
                }
            }
        }),
        // 获取市场分析
        json!({
            "type": "function",
            "function": {
                "name": "get_market_analysis",
                "description": "Get technical analysis and market sentiment",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Symbol to analyze"
                        },
                        "timeframe": {
                            "type": "string",
                            "enum": ["1h", "4h", "1d", "1w"],
                            "description": "Analysis timeframe"
                        }
                    },
                    "required": ["symbol"]
                }
            }
        }),
    ];

    // 5. 演示场景：Trading Assistant 对话
    println!("\n=== Scenario 1: Price Query ===");
    demo_price_query(&engine, &trading_tools).await?;

    println!("\n=== Scenario 2: Trading Decision ===");
    demo_trading_decision(&engine, &trading_tools).await?;

    println!("\n=== Scenario 3: Portfolio Analysis ===");
    demo_portfolio_analysis(&engine, &trading_tools).await?;

    println!("\n✓ All demos completed!");

    Ok(())
}

async fn demo_price_query(
    engine: &Arc<TradingEngine>,
    tools: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful trading assistant. Use available tools to get real-time market data."
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "What's the current price of Bitcoin and Ethereum?".to_string(),
        },
    ];

    match engine
        .chat_with_tools("gpt4", messages, tools.to_vec())
        .await
    {
        Ok(response) => {
            println!("🤖 Assistant: {}", response.content);
            if let Some(tool_calls) = response.tool_calls {
                println!("\n🔧 Tool Calls:");
                for call in tool_calls {
                    println!("   → {} with args: {}", call.name, call.arguments);
                }
            }
            println!("📊 Tokens used: {}/{}", 
                response.usage.prompt_tokens, 
                response.usage.total_tokens
            );
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("💡 Make sure to set OPENAI_API_KEY environment variable");
        }
    }

    Ok(())
}

async fn demo_trading_decision(
    engine: &Arc<TradingEngine>,
    tools: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: r#"You are an expert trading advisor. Analyze market data and provide trading recommendations.
When asked for advice:
1. First get the current price
2. Get market analysis if needed
3. Consider risk management
4. Provide clear buy/sell/hold recommendations with reasoning"#
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Should I buy Solana right now? I have $1000 to invest.".to_string(),
        },
    ];

    match engine
        .chat_with_tools("gpt4", messages, tools.to_vec())
        .await
    {
        Ok(response) => {
            println!("🤖 Assistant: {}", response.content);
            if let Some(tool_calls) = response.tool_calls {
                println!("\n🔧 Tool Calls:");
                for call in tool_calls {
                    println!("   → {} with args: {}", call.name, call.arguments);
                }
            }
            println!("📊 Tokens used: {}/{}", 
                response.usage.prompt_tokens, 
                response.usage.total_tokens
            );
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("💡 Make sure to set OPENAI_API_KEY environment variable");
        }
    }

    Ok(())
}

async fn demo_portfolio_analysis(
    engine: &Arc<TradingEngine>,
    tools: &[serde_json::Value],
) -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a portfolio manager. Help users understand their positions and suggest optimizations."
                .to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Check my account balance and tell me if my portfolio is well diversified.".to_string(),
        },
    ];

    match engine
        .chat_with_tools("gpt35", messages, tools.to_vec())
        .await
    {
        Ok(response) => {
            println!("🤖 Assistant: {}", response.content);
            if let Some(tool_calls) = response.tool_calls {
                println!("\n🔧 Tool Calls:");
                for call in tool_calls {
                    println!("   → {} with args: {}", call.name, call.arguments);
                }
            }
            println!("📊 Tokens used: {}/{}", 
                response.usage.prompt_tokens, 
                response.usage.total_tokens
            );
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("💡 Make sure to set OPENAI_API_KEY environment variable");
        }
    }

    Ok(())
}
