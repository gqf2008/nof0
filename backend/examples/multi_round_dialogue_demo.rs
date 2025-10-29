/// Multi-round Dialogue Demo
/// 
/// 这个示例展示了如何使用 ToolExecutor 实现多轮对话
/// LLM 会根据工具执行结果自动决定下一步操作

use nof0_backend::config::Config;
use nof0_backend::engine::{DialogueResult, TradingEngine, ToolExecutor};
use nof0_backend::llm::{ChatRequest, ChatResponse, LlmProvider, Message};
use nof0_backend::mcp::{McpServer, McpTool, ToolHandler};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tracing_subscriber;

// ==================== Mock Tool Handlers ====================

/// 获取加密货币价格
struct GetCryptoPriceHandler;

#[async_trait]
impl ToolHandler for GetCryptoPriceHandler {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC");

        // 模拟价格数据
        let price = match symbol {
            "BTC" => 67500.0,
            "ETH" => 3200.0,
            "SOL" => 145.0,
            _ => 0.0,
        };

        Ok(json!({
            "symbol": symbol,
            "price": price,
            "currency": "USDT",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// 获取账户余额
struct GetAccountBalanceHandler;

#[async_trait]
impl ToolHandler for GetAccountBalanceHandler {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let account_id = input
            .get("account_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // 模拟余额数据
        Ok(json!({
            "account_id": account_id,
            "balances": [
                { "asset": "USDT", "free": 10000.0, "locked": 500.0 },
                { "asset": "BTC", "free": 0.5, "locked": 0.0 },
                { "asset": "ETH", "free": 2.0, "locked": 0.1 }
            ],
            "total_value_usdt": 50000.0
        }))
    }
}

/// 下单
struct PlaceOrderHandler;

#[async_trait]
impl ToolHandler for PlaceOrderHandler {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol"))?;

        let side = input
            .get("side")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing side"))?;

        let quantity = input
            .get("quantity")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing quantity"))?;

        let price = input.get("price").and_then(|v| v.as_f64());

        // 模拟订单 ID
        let order_id = format!("ORDER_{}", chrono::Utc::now().timestamp_millis());

        Ok(json!({
            "order_id": order_id,
            "symbol": symbol,
            "side": side,
            "quantity": quantity,
            "price": price,
            "status": "FILLED",
            "message": format!("Successfully placed {} order for {} {}", side, quantity, symbol)
        }))
    }
}

/// 市场分析
struct GetMarketAnalysisHandler;

#[async_trait]
impl ToolHandler for GetMarketAnalysisHandler {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC");

        // 模拟市场分析
        Ok(json!({
            "symbol": symbol,
            "trend": "bullish",
            "indicators": {
                "rsi": 65.5,
                "macd": "positive",
                "volume": "increasing"
            },
            "support_levels": [65000, 63000],
            "resistance_levels": [70000, 72000],
            "recommendation": "The market shows bullish momentum with increasing volume. Consider buying on dips near support levels."
        }))
    }
}

// ==================== Tool Schemas ====================

fn create_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "get_crypto_price".to_string(),
            description: "Get the current price of a cryptocurrency".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The cryptocurrency symbol (e.g., BTC, ETH, SOL)"
                    }
                },
                "required": ["symbol"]
            }),
        },
        McpTool {
            name: "get_account_balance".to_string(),
            description: "Get account balance information".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "account_id": {
                        "type": "string",
                        "description": "The account ID to query"
                    }
                },
                "required": ["account_id"]
            }),
        },
        McpTool {
            name: "place_order".to_string(),
            description: "Place a trading order".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Trading pair symbol"
                    },
                    "side": {
                        "type": "string",
                        "enum": ["buy", "sell"],
                        "description": "Order side"
                    },
                    "quantity": {
                        "type": "number",
                        "description": "Order quantity"
                    },
                    "price": {
                        "type": "number",
                        "description": "Order price (optional for market orders)"
                    }
                },
                "required": ["symbol", "side", "quantity"]
            }),
        },
        McpTool {
            name: "get_market_analysis".to_string(),
            description: "Get technical analysis and market insights for a cryptocurrency".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The cryptocurrency symbol to analyze"
                    }
                },
                "required": ["symbol"]
            }),
        },
    ]
}

// ==================== Scenarios ====================

async fn scenario_complex_trading_decision(
    engine: &TradingEngine,
    executor: &ToolExecutor,
) -> Result<()> {
    println!("\n=== Scenario: Complex Trading Decision with Multi-round Dialogue ===\n");

    let provider_name = "openai";

    // 构造初始请求
    let initial_request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: "I want to invest in cryptocurrency. Please help me: 1) Check BTC and ETH prices, 2) Analyze their market trends, 3) Check my account balance, 4) Recommend which one to buy and how much, 5) Execute the order if it makes sense.".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(1500),
    };

    // 获取工具列表
    let tools = create_tools()
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                }
            })
        })
        .collect::<Vec<_>>();

    // 执行多轮对话
    let result = executor
        .execute_dialogue(initial_request, tools, |req, tools| {
            let engine = engine.clone();
            let provider_name = provider_name.to_string();
            async move { engine.chat_with_tools(&provider_name, req.messages, tools).await }
        })
        .await?;

    // 打印结果
    print_dialogue_result(&result);

    Ok(())
}

async fn scenario_error_recovery(engine: &TradingEngine, executor: &ToolExecutor) -> Result<()> {
    println!("\n=== Scenario: Error Recovery ===\n");

    let provider_name = "openai";

    // 构造一个会触发错误的请求
    let initial_request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: "Place an order to buy 10 BTC. But first, I forgot to specify the price - let the AI figure it out by checking the current price and my balance.".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(1500),
    };

    let tools = create_tools()
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                }
            })
        })
        .collect::<Vec<_>>();

    let result = executor
        .execute_dialogue(initial_request, tools, |req, tools| {
            let engine = engine.clone();
            let provider_name = provider_name.to_string();
            async move { engine.chat_with_tools(&provider_name, req.messages, tools).await }
        })
        .await?;

    print_dialogue_result(&result);

    Ok(())
}

async fn scenario_portfolio_analysis(engine: &TradingEngine, executor: &ToolExecutor) -> Result<()> {
    println!("\n=== Scenario: Portfolio Analysis ===\n");

    let provider_name = "openai";

    let initial_request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: "Analyze my portfolio: check my current holdings, get current prices for all assets I hold, calculate total value, and provide recommendations for rebalancing.".to_string(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(1500),
    };

    let tools = create_tools()
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                }
            })
        })
        .collect::<Vec<_>>();

    let result = executor
        .execute_dialogue(initial_request, tools, |req, tools| {
            let engine = engine.clone();
            let provider_name = provider_name.to_string();
            async move { engine.chat_with_tools(&provider_name, req.messages, tools).await }
        })
        .await?;

    print_dialogue_result(&result);

    Ok(())
}

// ==================== Helper Functions ====================

fn print_dialogue_result(result: &DialogueResult) {
    println!("┌─────────────────────────────────────────┐");
    println!("│       Multi-round Dialogue Result       │");
    println!("└─────────────────────────────────────────┘\n");

    println!("📊 Total Rounds: {}", result.total_rounds);
    println!("🔧 Tool Executions: {}\n", result.executions.len());

    // 打印每次工具执行
    for (i, exec) in result.executions.iter().enumerate() {
        println!("Tool Execution #{}", i + 1);
        println!("  ├─ Tool: {}", exec.tool_call.name);
        println!("  ├─ Success: {}", if exec.success { "✅" } else { "❌" });

        if exec.success {
            println!("  └─ Result: {}", exec.result);
        } else if let Some(error) = &exec.error {
            println!("  └─ Error: {}", error);
        }
        println!();
    }

    // 打印最终回复
    println!("💬 Final Response:");
    println!("─────────────────────────────────────────");
    println!("{}", result.final_response);
    println!("─────────────────────────────────────────\n");

    // 打印消息历史统计
    println!("📜 Message History:");
    let user_msgs = result
        .message_history
        .iter()
        .filter(|m| m.role == "user")
        .count();
    let assistant_msgs = result
        .message_history
        .iter()
        .filter(|m| m.role == "assistant")
        .count();
    let tool_msgs = result
        .message_history
        .iter()
        .filter(|m| m.role == "tool")
        .count();

    println!("  ├─ User messages: {}", user_msgs);
    println!("  ├─ Assistant messages: {}", assistant_msgs);
    println!("  └─ Tool messages: {}\n", tool_msgs);
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("╔══════════════════════════════════════════╗");
    println!("║   Multi-round Dialogue Demo             ║");
    println!("║   Showcasing Tool Executor               ║");
    println!("╚══════════════════════════════════════════╝\n");

    // 加载配置
    let config = Config::load()?;

    // 创建 MCP Server 并注册工具
    let mut mcp_server = McpServer::new();

    let tools = create_tools();
    mcp_server.register_tool(tools[0].clone(), Box::new(GetCryptoPriceHandler));
    mcp_server.register_tool(tools[1].clone(), Box::new(GetAccountBalanceHandler));
    mcp_server.register_tool(tools[2].clone(), Box::new(PlaceOrderHandler));
    mcp_server.register_tool(tools[3].clone(), Box::new(GetMarketAnalysisHandler));

    let mcp_server = Arc::new(mcp_server);

    // 创建 Trading Engine
    let engine = TradingEngine::new(mcp_server.clone());

    // 注册 LLM Provider
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");

    let openai_provider =
        nof0_backend::llm::RigOpenAIProvider::new(openai_key, "gpt-4")?;

    engine
        .register_llm_provider("openai".to_string(), Arc::new(openai_provider))
        .await;

    // 创建 Tool Executor
    let executor = ToolExecutor::new(mcp_server).with_max_rounds(10);

    println!("✅ Engine initialized with OpenAI provider");
    println!("✅ MCP Server with {} tools", tools.len());
    println!("✅ Tool Executor ready (max 10 rounds)\n");

    // 运行场景
    if let Err(e) = scenario_complex_trading_decision(&engine, &executor).await {
        eprintln!("❌ Scenario 1 failed: {}", e);
    }

    if let Err(e) = scenario_error_recovery(&engine, &executor).await {
        eprintln!("❌ Scenario 2 failed: {}", e);
    }

    if let Err(e) = scenario_portfolio_analysis(&engine, &executor).await {
        eprintln!("❌ Scenario 3 failed: {}", e);
    }

    println!("\n✅ All scenarios completed!");

    Ok(())
}
