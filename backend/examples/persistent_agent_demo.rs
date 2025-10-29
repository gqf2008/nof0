/// Persistent Agent Demo
/// 
/// 这个示例展示了如何使用 AgentStore 来持久化 Agent 的对话历史

use nof0_backend::config::Config;
use nof0_backend::engine::{AgentStore, TradingEngine, ToolExecutor};
use nof0_backend::llm::{ChatRequest, Message};
use nof0_backend::mcp::{McpServer, McpTool, ToolHandler};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tracing_subscriber;

// ==================== Mock Tool Handlers ====================

struct GetCryptoPriceHandler;

#[async_trait]
impl ToolHandler for GetCryptoPriceHandler {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("BTC");

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

        let order_id = format!("ORDER_{}", chrono::Utc::now().timestamp_millis());

        Ok(json!({
            "order_id": order_id,
            "symbol": symbol,
            "side": side,
            "quantity": quantity,
            "status": "FILLED",
            "message": format!("Successfully placed {} order for {} {}", side, quantity, symbol)
        }))
    }
}

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
                    }
                },
                "required": ["symbol", "side", "quantity"]
            }),
        },
    ]
}

// ==================== Scenarios ====================

async fn scenario_new_conversation(
    engine: &TradingEngine,
    executor: &ToolExecutor,
    store: &AgentStore,
    model_id: &str,
) -> Result<()> {
    println!("\n=== Scenario 1: New Conversation with Persistence ===\n");

    // 1. 创建新的会话
    let session = store.create_session(model_id).await?;
    println!("✅ Created session: {}", session.id);

    // 2. 用户输入
    let user_message = "Check BTC price and if it's below $70000, buy 0.1 BTC";
    println!("👤 User: {}", user_message);

    // 3. 添加用户消息到会话
    store
        .add_message(session.id, "user", user_message)
        .await?;

    // 4. 准备对话请求
    let initial_request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: user_message.to_string(),
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

    // 5. 执行对话
    let result = executor
        .execute_dialogue(initial_request, tools, |req, tools| {
            let engine = engine;
            async move { engine.chat_with_tools("openai", req.messages, tools).await }
        })
        .await?;

    // 6. 保存所有对话历史到数据库
    println!("\n💾 Saving conversation to database...");
    store
        .add_messages(session.id, &result.message_history[1..])
        .await?;

    println!("✅ Saved {} messages", result.message_history.len() - 1);
    println!("\n🤖 Assistant: {}", result.final_response);

    // 7. 显示会话统计
    let stats = store.get_session_stats(session.id).await?;
    println!("\n📊 Session Stats:");
    println!("  Total messages: {}", stats.total_messages);
    println!("  User messages: {}", stats.user_messages);
    println!("  Assistant messages: {}", stats.assistant_messages);
    println!("  Tool messages: {}", stats.tool_messages);
    println!("  Total characters: {}", stats.total_characters);

    Ok(())
}

async fn scenario_resume_conversation(
    engine: &TradingEngine,
    executor: &ToolExecutor,
    store: &AgentStore,
    model_id: &str,
) -> Result<()> {
    println!("\n=== Scenario 2: Resume Previous Conversation ===\n");

    // 1. 列出该 model 的所有会话
    let sessions = store.list_sessions(model_id).await?;
    
    if sessions.is_empty() {
        println!("⚠️ No previous sessions found. Creating a new one...");
        return scenario_new_conversation(engine, executor, store, model_id).await;
    }

    println!("Found {} previous sessions:", sessions.len());
    for (i, summary) in sessions.iter().enumerate() {
        println!(
            "  {}. Session {} - {} messages - First: {}",
            i + 1,
            summary.session.id,
            summary.message_count,
            summary
                .first_message
                .as_ref()
                .map(|s| if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.clone()
                })
                .unwrap_or_else(|| "No messages".to_string())
        );
    }

    // 2. 选择最近的会话
    let session = &sessions[0].session;
    println!("\n📖 Resuming session {}", session.id);

    // 3. 加载历史消息
    let history = store.get_messages(session.id).await?;
    println!("Loaded {} historical messages", history.len());

    // 显示最后几条消息
    println!("\n📜 Recent history:");
    for msg in history.iter().rev().take(3).rev() {
        let preview = if msg.content.len() > 60 {
            format!("{}...", &msg.content[..60])
        } else {
            msg.content.clone()
        };
        println!("  {} {}: {}", 
            match msg.role.as_str() {
                "user" => "👤",
                "assistant" => "🤖",
                "tool" => "🔧",
                _ => "💬",
            },
            msg.role,
            preview
        );
    }

    // 4. 添加新的用户消息
    let user_message = "What was the last action I took? And what's the current BTC price now?";
    println!("\n👤 User: {}", user_message);

    store
        .add_message(session.id, "user", user_message)
        .await?;

    // 5. 重建完整的对话历史
    let mut full_history = store.agent_messages_to_llm_messages(history);
    full_history.push(Message {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    // 6. 继续对话
    let initial_request = ChatRequest {
        messages: full_history,
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
            let engine = engine;
            async move { engine.chat_with_tools("openai", req.messages, tools).await }
        })
        .await?;

    // 7. 保存新的消息
    println!("\n💾 Saving new messages...");
    let new_messages = &result.message_history[result.message_history.len() - result.total_rounds..];
    store.add_messages(session.id, new_messages).await?;

    println!("✅ Saved {} new messages", new_messages.len());
    println!("\n🤖 Assistant: {}", result.final_response);

    Ok(())
}

async fn scenario_search_history(store: &AgentStore, model_id: &str) -> Result<()> {
    println!("\n=== Scenario 3: Search Conversation History ===\n");

    let query = "BTC";
    println!("🔍 Searching for: '{}'", query);

    let messages = store.search_messages(model_id, query, 10).await?;
    
    if messages.is_empty() {
        println!("No messages found.");
        return Ok(());
    }

    println!("Found {} messages:", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        let preview = if msg.content.len() > 80 {
            format!("{}...", &msg.content[..80])
        } else {
            msg.content.clone()
        };
        println!(
            "\n{}. [Session {}] {} {}:",
            i + 1,
            msg.conversation_id,
            match msg.role.as_str() {
                "user" => "👤",
                "assistant" => "🤖",
                "tool" => "🔧",
                _ => "💬",
            },
            msg.role
        );
        println!("   {}", preview);
    }

    Ok(())
}

async fn scenario_session_management(store: &AgentStore, model_id: &str) -> Result<()> {
    println!("\n=== Scenario 4: Session Management ===\n");

    // 列出所有会话
    let sessions = store.list_sessions(model_id).await?;
    println!("📋 Total sessions: {}", sessions.len());

    for summary in &sessions {
        println!("\nSession {}:", summary.session.id);
        println!("  Messages: {}", summary.message_count);
        println!(
            "  Last activity: {}",
            summary
                .last_message_time
                .map(|t| format!("{} ms ago", chrono::Utc::now().timestamp_millis() - t))
                .unwrap_or_else(|| "Never".to_string())
        );

        // 获取统计
        let stats = store.get_session_stats(summary.session.id).await?;
        println!("  User msgs: {}, Assistant msgs: {}, Tool msgs: {}", 
            stats.user_messages,
            stats.assistant_messages,
            stats.tool_messages
        );
    }

    // 如果有超过 3 个会话，删除最旧的
    if sessions.len() > 3 {
        let oldest = &sessions[sessions.len() - 1];
        println!("\n🗑️ Deleting oldest session: {}", oldest.session.id);
        store.delete_session(oldest.session.id).await?;
        println!("✅ Deleted");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("╔══════════════════════════════════════════╗");
    println!("║   Persistent Agent Demo                 ║");
    println!("║   Conversation History Management       ║");
    println!("╚══════════════════════════════════════════╝\n");

    // 加载配置
    let _config = Config::load()?;

    // 连接数据库
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nof0:nof0@localhost:5432/nof0".to_string());

    println!("📡 Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    println!("✅ Connected to database\n");

    // 创建 AgentStore
    let agent_store = AgentStore::new(Arc::new(pool));

    // 创建 MCP Server 并注册工具
    let mut mcp_server = McpServer::new();
    let tools = create_tools();
    mcp_server.register_tool(tools[0].clone(), Box::new(GetCryptoPriceHandler));
    mcp_server.register_tool(tools[1].clone(), Box::new(PlaceOrderHandler));
    let mcp_server = Arc::new(mcp_server);

    // 创建 Trading Engine
    let engine = TradingEngine::new(mcp_server.clone());

    // 注册 LLM Provider
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");

    let openai_provider = nof0_backend::llm::RigOpenAIProvider::new(openai_key, "gpt-4")?;
    engine
        .register_llm_provider("openai".to_string(), Arc::new(openai_provider))
        .await;

    // 创建 Tool Executor
    let executor = ToolExecutor::new(mcp_server).with_max_rounds(10);

    println!("✅ All systems initialized\n");

    let model_id = "demo-agent";

    // 运行场景
    if let Err(e) = scenario_new_conversation(&engine, &executor, &agent_store, model_id).await {
        eprintln!("❌ Scenario 1 failed: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if let Err(e) = scenario_resume_conversation(&engine, &executor, &agent_store, model_id).await
    {
        eprintln!("❌ Scenario 2 failed: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if let Err(e) = scenario_search_history(&agent_store, model_id).await {
        eprintln!("❌ Scenario 3 failed: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if let Err(e) = scenario_session_management(&agent_store, model_id).await {
        eprintln!("❌ Scenario 4 failed: {}", e);
    }

    println!("\n✅ All scenarios completed!");

    Ok(())
}
