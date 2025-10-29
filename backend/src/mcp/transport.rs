use super::types::McpMessage;
use anyhow::Result;

pub enum Transport {
    Stdio,
    Sse,
    WebSocket,
}

impl Transport {
    pub async fn send(&self, _msg: &McpMessage) -> Result<()> {
        // TODO: 实现不同传输方式
        Ok(())
    }

    pub async fn receive(&self) -> Result<McpMessage> {
        // TODO: 实现不同传输方式
        todo!()
    }
}
