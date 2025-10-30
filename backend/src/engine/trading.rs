use crate::llm::{ChatRequest, LlmProvider, Message};
use crate::markets::MarketAdapter;
use crate::mcp::McpServer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct TradingEngine {
    mcp_server: Arc<McpServer>,
    llm_providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    markets: Arc<RwLock<HashMap<String, Arc<dyn MarketAdapter>>>>,
    // agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl TradingEngine {
    pub fn new(mcp_server: Arc<McpServer>) -> Self {
        Self {
            mcp_server,
            llm_providers: Arc::new(RwLock::new(HashMap::new())),
            markets: Arc::new(RwLock::new(HashMap::new())),
            // agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_llm_provider(&self, name: String, provider: Arc<dyn LlmProvider>) {
        info!("Registering LLM provider: {}", name);
        self.llm_providers.write().await.insert(name, provider);
    }

    pub async fn register_market(&self, name: String, adapter: Arc<dyn MarketAdapter>) {
        info!("Registering market adapter: {}", name);
        self.markets.write().await.insert(name, adapter);
    }

    pub async fn get_llm_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.llm_providers.read().await.get(name).cloned()
    }

    pub async fn get_market(&self, name: &str) -> Option<Arc<dyn MarketAdapter>> {
        self.markets.read().await.get(name).cloned()
    }

    pub async fn list_llm_providers(&self) -> Vec<String> {
        self.llm_providers.read().await.keys().cloned().collect()
    }

    pub async fn list_markets(&self) -> Vec<String> {
        self.markets.read().await.keys().cloned().collect()
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        info!("Trading engine started");

        // 列出已注册的 Providers
        let providers = self.list_llm_providers().await;
        info!("Registered LLM providers: {:?}", providers);

        let markets = self.list_markets().await;
        info!("Registered markets: {:?}", markets);

        // TODO: 主循环
        // 1. 启动 MCP Server
        // 2. 启动调度器
        // 3. 启动价格订阅
        // 4. 执行 Agent 决策

        Ok(())
    }

    /// 执行简单的 LLM 查询（用于测试和演示）
    pub async fn simple_chat(
        &self,
        provider_name: &str,
        message: &str,
    ) -> Result<String, anyhow::Error> {
        let provider = self
            .get_llm_provider(provider_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", provider_name))?;

        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(500),
        };

        let response = provider.chat(request).await?;
        Ok(response.content)
    }

    /// 使用工具执行 LLM 查询
    pub async fn chat_with_tools(
        &self,
        provider_name: &str,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<crate::llm::ChatResponse, anyhow::Error> {
        let provider = self
            .get_llm_provider(provider_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", provider_name))?;

        let request = ChatRequest {
            messages,
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };

        let response = provider.chat_with_tools(request, tools).await?;
        Ok(response)
    }

    async fn execute_agent(&self /* agent: &Agent */) -> Result<(), anyhow::Error> {
        // TODO: 执行单个 Agent
        // 1. 获取市场数据
        // 2. 构建 Prompt
        // 3. 调用 LLM
        // 4. 解析决策
        // 5. 执行交易
        // 6. 记录结果
        Ok(())
    }
}
