use crate::llm::{ChatRequest, ChatResponse, Message, ToolCall};
use crate::mcp::{McpError, McpServer};
use anyhow::{Context, Result};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 工具执行器 - 负责执行 LLM 返回的工具调用并管理多轮对话
pub struct ToolExecutor {
    mcp_server: Arc<McpServer>,
    max_rounds: usize,
}

/// 单次执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 工具调用
    pub tool_call: ToolCall,
    /// 执行结果
    pub result: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 多轮对话执行结果
#[derive(Debug, Clone)]
pub struct DialogueResult {
    /// 最终回复
    pub final_response: String,
    /// 总轮数
    pub total_rounds: usize,
    /// 所有工具执行记录
    pub executions: Vec<ExecutionResult>,
    /// 完整的消息历史
    pub message_history: Vec<Message>,
}

impl ToolExecutor {
    /// 创建新的工具执行器
    pub fn new(mcp_server: Arc<McpServer>) -> Self {
        Self {
            mcp_server,
            max_rounds: 10, // 默认最多 10 轮对话
        }
    }

    /// 设置最大轮数
    pub fn with_max_rounds(mut self, max_rounds: usize) -> Self {
        self.max_rounds = max_rounds;
        self
    }

    /// 执行单个工具调用
    pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> ExecutionResult {
        info!("Executing tool: {}", tool_call.name);
        debug!("Tool arguments: {:?}", tool_call.arguments);

        // 构造 MCP 请求
        let mcp_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_call.name,
                "arguments": tool_call.arguments
            }
        });

        // 反序列化为 McpMessage
        let mcp_message: crate::mcp::McpMessage =
            match serde_json::from_value(mcp_request.clone()) {
                Ok(msg) => msg,
                Err(e) => {
                    let error_msg = format!("Failed to parse MCP request: {}", e);
                    warn!("{}", error_msg);
                    return ExecutionResult {
                        tool_call: tool_call.clone(),
                        result: String::new(),
                        success: false,
                        error: Some(error_msg),
                    };
                }
            };

        // 调用 MCP Server
        let response = self.mcp_server.handle_request(mcp_message).await;

        // 解析响应
        if let Some(result) = response.result {
            // 成功
            let result_text = if let Some(content) = result.get("content") {
                if let Some(content_array) = content.as_array() {
                    content_array
                        .iter()
                        .filter_map(|item| {
                            item.get("text").and_then(|t| t.as_str()).map(String::from)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    result.to_string()
                }
            } else {
                result.to_string()
            };

            info!("Tool execution succeeded: {}", tool_call.name);
            debug!("Tool result: {}", result_text);

            ExecutionResult {
                tool_call: tool_call.clone(),
                result: result_text,
                success: true,
                error: None,
            }
        } else if let Some(error) = response.error {
            // 失败
            let error_msg = error.message.clone();

            warn!("Tool execution failed: {} - {}", tool_call.name, error_msg);

            ExecutionResult {
                tool_call: tool_call.clone(),
                result: String::new(),
                success: false,
                error: Some(error_msg),
            }
        } else {
            // 未知响应
            let error_msg = "Invalid MCP response: no result or error".to_string();
            warn!("{}", error_msg);

            ExecutionResult {
                tool_call: tool_call.clone(),
                result: String::new(),
                success: false,
                error: Some(error_msg),
            }
        }
    }

    /// 执行多个工具调用
    pub async fn execute_tool_calls(
        &self,
        tool_calls: &[ToolCall],
    ) -> Vec<ExecutionResult> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            let result = self.execute_tool_call(tool_call).await;
            results.push(result);
        }

        results
    }

    /// 将工具执行结果转换为聊天消息
    pub fn execution_results_to_messages(
        &self,
        executions: &[ExecutionResult],
    ) -> Vec<Message> {
        executions
            .iter()
            .map(|exec| {
                let content = if exec.success {
                    format!(
                        "Tool '{}' executed successfully:\n{}",
                        exec.tool_call.name, exec.result
                    )
                } else {
                    format!(
                        "Tool '{}' execution failed: {}",
                        exec.tool_call.name,
                        exec.error.as_ref().unwrap_or(&"Unknown error".to_string())
                    )
                };

                Message {
                    role: "tool".to_string(),
                    content,
                }
            })
            .collect()
    }

    /// 执行多轮对话 - 核心方法
    ///
    /// 这个方法会：
    /// 1. 发送初始消息给 LLM
    /// 2. 如果 LLM 返回工具调用，执行工具
    /// 3. 将工具结果返回给 LLM
    /// 4. 重复步骤 2-3 直到 LLM 返回最终回复或达到最大轮数
    pub async fn execute_dialogue<F, Fut>(
        &self,
        initial_request: ChatRequest,
        tools: Vec<serde_json::Value>,
        chat_fn: F,
    ) -> Result<DialogueResult>
    where
        F: Fn(ChatRequest, Vec<serde_json::Value>) -> Fut,
        Fut: std::future::Future<Output = Result<ChatResponse>>,
    {
        let mut message_history = initial_request.messages.clone();
        let mut all_executions = Vec::new();
        let mut round = 0;

        info!("Starting dialogue with {} available tools", tools.len());

        loop {
            round += 1;

            if round > self.max_rounds {
                warn!("Reached maximum rounds ({}), stopping dialogue", self.max_rounds);
                break;
            }

            info!("Dialogue round {}", round);

            // 构造当前请求
            let current_request = ChatRequest {
                messages: message_history.clone(),
                ..initial_request.clone()
            };

            // 调用 LLM
            let response = chat_fn(current_request, tools.clone())
                .await
                .context("Failed to call LLM")?;

            // 将 LLM 的回复添加到历史
            let assistant_message = Message {
                role: "assistant".to_string(),
                content: response.content.clone(),
            };
            message_history.push(assistant_message);

            // 检查是否有工具调用
            if let Some(tool_calls) = &response.tool_calls {
                if tool_calls.is_empty() {
                    // 没有工具调用，返回最终结果
                    info!("No more tool calls, dialogue finished in {} rounds", round);
                    return Ok(DialogueResult {
                        final_response: response.content,
                        total_rounds: round,
                        executions: all_executions,
                        message_history,
                    });
                }

                info!("LLM requested {} tool calls", tool_calls.len());

                // 执行所有工具调用
                let executions = self.execute_tool_calls(tool_calls).await;

                // 记录执行结果
                all_executions.extend(executions.clone());

                // 将工具执行结果转换为消息
                let tool_messages = self.execution_results_to_messages(&executions);

                // 添加到历史
                message_history.extend(tool_messages);

                // 继续下一轮
            } else {
                // 没有工具调用，返回最终结果
                info!("No tool calls in response, dialogue finished in {} rounds", round);
                return Ok(DialogueResult {
                    final_response: response.content,
                    total_rounds: round,
                    executions: all_executions,
                    message_history,
                });
            }
        }

        // 达到最大轮数，返回当前状态
        Ok(DialogueResult {
            final_response: "Dialogue exceeded maximum rounds".to_string(),
            total_rounds: round,
            executions: all_executions,
            message_history,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_executor_creation() {
        let mcp_server = Arc::new(McpServer::new());
        let executor = ToolExecutor::new(mcp_server);
        assert_eq!(executor.max_rounds, 10);
    }

    #[tokio::test]
    async fn test_with_max_rounds() {
        let mcp_server = Arc::new(McpServer::new());
        let executor = ToolExecutor::new(mcp_server).with_max_rounds(5);
        assert_eq!(executor.max_rounds, 5);
    }
}
