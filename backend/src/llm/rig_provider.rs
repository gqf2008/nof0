use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::provider::{ChatRequest, ChatResponse, LlmProvider, Message, TokenUsage, ToolCall};

/// 直接使用 HTTP 请求的 OpenAI Provider
pub struct RigOpenAIProvider {
    name: String,
    client: Client,
    api_key: String,
    model_name: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl RigOpenAIProvider {
    pub fn new(api_key: String, model: &str) -> Result<Self> {
        Ok(Self {
            name: format!("openai-{}", model),
            client: Client::new(),
            api_key,
            model_name: model.to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl LlmProvider for RigOpenAIProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        info!("Sending chat request to {}", self.name);

        let openai_req = OpenAIRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role,
                    content: m.content,
                    tool_calls: None,
                    tool_call_id: None,
                })
                .collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: None,
            tool_choice: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }

        let openai_resp: OpenAIResponse = response.json().await?;

        let choice = openai_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

        Ok(ChatResponse {
            content: choice.message.content,
            tool_calls: None,
            usage: TokenUsage {
                prompt_tokens: openai_resp.usage.prompt_tokens,
                completion_tokens: openai_resp.usage.completion_tokens,
                total_tokens: openai_resp.usage.total_tokens,
            },
        })
    }

    async fn chat_with_tools(
        &self,
        req: ChatRequest,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse> {
        info!("Sending chat request with tools to {}", self.name);

        let openai_req = OpenAIRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role,
                    content: m.content,
                    tool_calls: None,
                    tool_call_id: None,
                })
                .collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }

        let openai_resp: OpenAIResponse = response.json().await?;

        // 提取 tool_calls（如果有）
        let tool_calls = openai_resp
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.clone())
            .map(|calls| {
                calls
                    .into_iter()
                    .map(|tc| ToolCall {
                        id: tc.id,
                        name: tc.function.name,
                        arguments: serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(serde_json::json!({})),
                    })
                    .collect()
            });

        Ok(ChatResponse {
            content: openai_resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            tool_calls,
            usage: TokenUsage {
                prompt_tokens: openai_resp.usage.prompt_tokens,
                completion_tokens: openai_resp.usage.completion_tokens,
                total_tokens: openai_resp.usage.total_tokens,
            },
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> &str {
        &self.model_name
    }
}

/// Anthropic Claude Provider
pub struct AnthropicProvider {
    name: String,
    client: Client,
    api_key: String,
    model_name: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentItem>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContentItem {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: &str) -> Result<Self> {
        Ok(Self {
            name: format!("anthropic-{}", model),
            client: Client::new(),
            api_key,
            model_name: model.to_string(),
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        info!("Sending chat request to {}", self.name);

        // 提取 system message
        let system_msg = req
            .messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let anthropic_req = AnthropicRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .filter(|m| m.role != "system") // Anthropic 单独处理 system
                .map(|m| AnthropicMessage {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            max_tokens: req.max_tokens.unwrap_or(4096),
            temperature: req.temperature,
            system: system_msg,
            tools: None,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("Anthropic API error ({}): {}", status, error_text);
        }

        let anthropic_resp: AnthropicResponse = response.json().await?;

        // 提取文本内容
        let content = anthropic_resp
            .content
            .iter()
            .filter_map(|item| match item {
                AnthropicContentItem::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ChatResponse {
            content,
            tool_calls: None,
            usage: TokenUsage {
                prompt_tokens: anthropic_resp.usage.input_tokens,
                completion_tokens: anthropic_resp.usage.output_tokens,
                total_tokens: anthropic_resp.usage.input_tokens
                    + anthropic_resp.usage.output_tokens,
            },
        })
    }

    async fn chat_with_tools(
        &self,
        req: ChatRequest,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse> {
        info!("Sending chat request with tools to {}", self.name);

        // 提取 system message
        let system_msg = req
            .messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let anthropic_req = AnthropicRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .filter(|m| m.role != "system")
                .map(|m| AnthropicMessage {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            max_tokens: req.max_tokens.unwrap_or(4096),
            temperature: req.temperature,
            system: system_msg,
            tools: Some(tools),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("Anthropic API error ({}): {}", status, error_text);
        }

        let anthropic_resp: AnthropicResponse = response.json().await?;

        // 提取文本内容和 tool_use
        let mut content_texts = Vec::new();
        let mut tool_calls = Vec::new();

        for item in anthropic_resp.content {
            match item {
                AnthropicContentItem::Text { text } => {
                    content_texts.push(text);
                }
                AnthropicContentItem::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        Ok(ChatResponse {
            content: content_texts.join("\n"),
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            usage: TokenUsage {
                prompt_tokens: anthropic_resp.usage.input_tokens,
                completion_tokens: anthropic_resp.usage.output_tokens,
                total_tokens: anthropic_resp.usage.input_tokens
                    + anthropic_resp.usage.output_tokens,
            },
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> &str {
        &self.model_name
    }
}

/// 通用的 OpenAI 兼容 Provider (支持 DeepSeek, Qwen, etc.)
pub struct OpenAICompatibleProvider {
    name: String,
    client: Client,
    api_key: String,
    model_name: String,
    base_url: String,
}

impl OpenAICompatibleProvider {
    pub fn deepseek(api_key: String, model: &str) -> Result<Self> {
        Ok(Self {
            name: format!("deepseek-{}", model),
            client: Client::new(),
            api_key,
            model_name: model.to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
        })
    }

    pub fn qwen(api_key: String, model: &str) -> Result<Self> {
        Ok(Self {
            name: format!("qwen-{}", model),
            client: Client::new(),
            api_key,
            model_name: model.to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
        })
    }

    pub fn custom(api_key: String, model: &str, base_url: String, name: String) -> Result<Self> {
        Ok(Self {
            name,
            client: Client::new(),
            api_key,
            model_name: model.to_string(),
            base_url,
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAICompatibleProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        info!("Sending chat request to {}", self.name);

        let openai_req = OpenAIRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role,
                    content: m.content,
                    tool_calls: None,
                    tool_call_id: None,
                })
                .collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: None,
            tool_choice: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("{} API error ({}): {}", self.name, status, error_text);
        }

        let openai_resp: OpenAIResponse = response.json().await?;

        let choice = openai_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No response from {}", self.name))?;

        Ok(ChatResponse {
            content: choice.message.content,
            tool_calls: None,
            usage: TokenUsage {
                prompt_tokens: openai_resp.usage.prompt_tokens,
                completion_tokens: openai_resp.usage.completion_tokens,
                total_tokens: openai_resp.usage.total_tokens,
            },
        })
    }

    async fn chat_with_tools(
        &self,
        req: ChatRequest,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse> {
        info!("Sending chat request with tools to {}", self.name);

        let openai_req = OpenAIRequest {
            model: self.model_name.clone(),
            messages: req
                .messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role,
                    content: m.content,
                    tool_calls: None,
                    tool_call_id: None,
                })
                .collect(),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("{} API error ({}): {}", self.name, status, error_text);
        }

        let openai_resp: OpenAIResponse = response.json().await?;

        // 提取 tool_calls（如果有）
        let tool_calls = openai_resp
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.clone())
            .map(|calls| {
                calls
                    .into_iter()
                    .map(|tc| ToolCall {
                        id: tc.id,
                        name: tc.function.name,
                        arguments: serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(serde_json::json!({})),
                    })
                    .collect()
            });

        Ok(ChatResponse {
            content: openai_resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            tool_calls,
            usage: TokenUsage {
                prompt_tokens: openai_resp.usage.prompt_tokens,
                completion_tokens: openai_resp.usage.completion_tokens,
                total_tokens: openai_resp.usage.total_tokens,
            },
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_openai_provider() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let provider = RigOpenAIProvider::new(api_key, "gpt-4").unwrap();

        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "Say hello!".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
        };

        let response = provider.chat(request).await.unwrap();
        assert!(!response.content.is_empty());
    }
}
