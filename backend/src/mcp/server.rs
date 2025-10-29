use super::types::{McpMessage, McpTool};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, error, info};

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, anyhow::Error>;
}

pub struct McpServer {
    tools: HashMap<String, Box<dyn ToolHandler>>,
    tool_schemas: HashMap<String, McpTool>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            tool_schemas: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, schema: McpTool, handler: Box<dyn ToolHandler>) {
        let name = schema.name.clone();
        self.tool_schemas.insert(name.clone(), schema);
        self.tools.insert(name.clone(), handler);
        info!("Registered MCP tool: {}", name);
    }

    pub async fn handle_request(&self, msg: McpMessage) -> McpMessage {
        debug!("Handling MCP request: {:?}", msg);

        let id = msg.id.clone();
        let method = match &msg.method {
            Some(m) => m,
            None => {
                return McpMessage::error(id, -32600, "Invalid Request: missing method".to_string())
            }
        };

        match method.as_str() {
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, msg.params).await,
            "resources/list" => self.handle_resources_list(id),
            "prompts/list" => self.handle_prompts_list(id),
            _ => McpMessage::error(id, -32601, format!("Method not found: {}", method)),
        }
    }

    fn handle_tools_list(&self, id: Option<serde_json::Value>) -> McpMessage {
        let tools: Vec<&McpTool> = self.tool_schemas.values().collect();
        McpMessage::success(
            id.unwrap_or(serde_json::Value::Null),
            serde_json::json!({ "tools": tools }),
        )
    }

    async fn handle_tools_call(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> McpMessage {
        let params = match params {
            Some(p) => p,
            None => {
                return McpMessage::error(
                    id,
                    -32602,
                    "Invalid params: missing parameters".to_string(),
                )
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return McpMessage::error(
                    id,
                    -32602,
                    "Invalid params: missing tool name".to_string(),
                )
            }
        };

        let input = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let handler = match self.tools.get(tool_name) {
            Some(h) => h,
            None => return McpMessage::error(id, -32602, format!("Tool not found: {}", tool_name)),
        };

        match handler.execute(input).await {
            Ok(result) => McpMessage::success(
                id.unwrap_or(serde_json::Value::Null),
                serde_json::json!({ "content": [{ "type": "text", "text": result.to_string() }] }),
            ),
            Err(e) => {
                error!("Tool execution error: {}", e);
                McpMessage::error(id, -32603, format!("Tool execution failed: {}", e))
            }
        }
    }

    fn handle_resources_list(&self, id: Option<serde_json::Value>) -> McpMessage {
        McpMessage::success(
            id.unwrap_or(serde_json::Value::Null),
            serde_json::json!({ "resources": [] }),
        )
    }

    fn handle_prompts_list(&self, id: Option<serde_json::Value>) -> McpMessage {
        McpMessage::success(
            id.unwrap_or(serde_json::Value::Null),
            serde_json::json!({ "prompts": [] }),
        )
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
