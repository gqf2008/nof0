// 示例：如何使用 LLM Provider

use nof0_backend::llm::{ChatRequest, LlmProvider, Message, RigOpenAIProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 从环境变量读取 API Key
    let api_key =
        std::env::var("OPENAI_API_KEY").expect("Please set OPENAI_API_KEY environment variable");

    // 创建 OpenAI Provider
    let provider = RigOpenAIProvider::new(api_key, "gpt-4")?;

    // 构建请求
    let request = ChatRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful crypto trading assistant.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Should I buy Bitcoin now?".to_string(),
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(500),
    };

    // 发送请求
    println!("Sending request to OpenAI...");
    let response = provider.chat(request).await?;

    // 打印结果
    println!("\n=== Response ===");
    println!("{}", response.content);
    println!("\n=== Usage ===");
    println!("Prompt tokens: {}", response.usage.prompt_tokens);
    println!("Completion tokens: {}", response.usage.completion_tokens);
    println!("Total tokens: {}", response.usage.total_tokens);

    Ok(())
}
