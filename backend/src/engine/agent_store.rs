use crate::llm::Message;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Agent 对话会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: i64,
    pub model_id: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Agent 消息（对话历史中的一条消息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub timestamp_ms: i64,
}

/// Agent 会话摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session: AgentSession,
    pub message_count: i64,
    pub last_message_time: Option<i64>,
    pub first_message: Option<String>,
}

/// Agent Store - 负责持久化 Agent 的对话历史和状态
pub struct AgentStore {
    pool: Arc<PgPool>,
}

impl AgentStore {
    /// 创建新的 AgentStore
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建新的对话会话
    pub async fn create_session(&self, model_id: &str) -> Result<AgentSession> {
        info!("Creating new session for model: {}", model_id);

        let row = sqlx::query(
            r#"
            INSERT INTO conversations (model_id)
            VALUES ($1)
            RETURNING id, model_id
            "#,
        )
        .bind(model_id)
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to create conversation session")?;

        let session = AgentSession {
            id: row.get("id"),
            model_id: row.get("model_id"),
            created_at: None,
        };

        info!("Created session {} for model {}", session.id, model_id);
        Ok(session)
    }

    /// 添加消息到会话
    pub async fn add_message(
        &self,
        conversation_id: i64,
        role: &str,
        content: &str,
    ) -> Result<AgentMessage> {
        debug!(
            "Adding message to conversation {}: role={}",
            conversation_id, role
        );

        let timestamp_ms = chrono::Utc::now().timestamp_millis();

        let row = sqlx::query(
            r#"
            INSERT INTO conversation_messages (conversation_id, role, content, ts_ms)
            VALUES ($1, $2, $3, $4)
            RETURNING id, conversation_id, role, content, ts_ms
            "#,
        )
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(timestamp_ms)
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to add message")?;

        let message = AgentMessage {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            role: row.get("role"),
            content: row.get("content"),
            timestamp_ms: row.get("ts_ms"),
        };

        debug!("Added message {}", message.id);
        Ok(message)
    }

    /// 批量添加消息
    pub async fn add_messages(
        &self,
        conversation_id: i64,
        messages: &[Message],
    ) -> Result<Vec<AgentMessage>> {
        info!(
            "Adding {} messages to conversation {}",
            messages.len(),
            conversation_id
        );

        let mut result = Vec::new();

        for msg in messages {
            let agent_msg = self
                .add_message(conversation_id, &msg.role, &msg.content)
                .await?;
            result.push(agent_msg);
        }

        Ok(result)
    }

    /// 获取会话的所有消息
    pub async fn get_messages(&self, conversation_id: i64) -> Result<Vec<AgentMessage>> {
        debug!("Fetching messages for conversation {}", conversation_id);

        let rows = sqlx::query(
            r#"
            SELECT id, conversation_id, role, content, ts_ms
            FROM conversation_messages
            WHERE conversation_id = $1
            ORDER BY ts_ms ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to fetch messages")?;

        let messages: Vec<AgentMessage> = rows
            .into_iter()
            .map(|row| AgentMessage {
                id: row.get("id"),
                conversation_id: row.get("conversation_id"),
                role: row.get("role"),
                content: row.get("content"),
                timestamp_ms: row.get("ts_ms"),
            })
            .collect();

        debug!("Fetched {} messages", messages.len());
        Ok(messages)
    }

    /// 将 AgentMessage 转换为 LLM Message
    pub fn agent_messages_to_llm_messages(&self, messages: Vec<AgentMessage>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|msg| Message {
                role: msg.role,
                content: msg.content,
            })
            .collect()
    }

    /// 获取会话信息
    pub async fn get_session(&self, conversation_id: i64) -> Result<Option<AgentSession>> {
        debug!("Fetching session {}", conversation_id);

        let row = sqlx::query(
            r#"
            SELECT id, model_id
            FROM conversations
            WHERE id = $1
            "#,
        )
        .bind(conversation_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .context("Failed to fetch session")?;

        Ok(row.map(|r| AgentSession {
            id: r.get("id"),
            model_id: r.get("model_id"),
            created_at: None,
        }))
    }

    /// 列出某个 model 的所有会话
    pub async fn list_sessions(&self, model_id: &str) -> Result<Vec<SessionSummary>> {
        info!("Listing sessions for model: {}", model_id);

        let rows = sqlx::query(
            r#"
            SELECT 
                c.id,
                c.model_id,
                COUNT(m.id) as message_count,
                MAX(m.ts_ms) as last_message_time,
                (SELECT content FROM conversation_messages 
                 WHERE conversation_id = c.id 
                 ORDER BY ts_ms ASC LIMIT 1) as first_message
            FROM conversations c
            LEFT JOIN conversation_messages m ON c.id = m.conversation_id
            WHERE c.model_id = $1
            GROUP BY c.id, c.model_id
            ORDER BY MAX(m.ts_ms) DESC NULLS LAST
            "#,
        )
        .bind(model_id)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to list sessions")?;

        let summaries: Vec<SessionSummary> = rows
            .into_iter()
            .map(|row| SessionSummary {
                session: AgentSession {
                    id: row.get("id"),
                    model_id: row.get("model_id"),
                    created_at: None,
                },
                message_count: row.get("message_count"),
                last_message_time: row.get("last_message_time"),
                first_message: row.get("first_message"),
            })
            .collect();

        info!("Found {} sessions", summaries.len());
        Ok(summaries)
    }

    /// 删除会话（级联删除所有消息）
    pub async fn delete_session(&self, conversation_id: i64) -> Result<bool> {
        warn!("Deleting session {}", conversation_id);

        let result = sqlx::query(
            r#"
            DELETE FROM conversations
            WHERE id = $1
            "#,
        )
        .bind(conversation_id)
        .execute(self.pool.as_ref())
        .await
        .context("Failed to delete session")?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted session {}", conversation_id);
        } else {
            warn!("Session {} not found", conversation_id);
        }

        Ok(deleted)
    }

    /// 搜索包含特定内容的消息
    pub async fn search_messages(
        &self,
        model_id: &str,
        query: &str,
        limit: i64,
    ) -> Result<Vec<AgentMessage>> {
        debug!(
            "Searching messages for model {} with query: {}",
            model_id, query
        );

        let rows = sqlx::query(
            r#"
            SELECT m.id, m.conversation_id, m.role, m.content, m.ts_ms
            FROM conversation_messages m
            JOIN conversations c ON m.conversation_id = c.id
            WHERE c.model_id = $1 AND m.content ILIKE $2
            ORDER BY m.ts_ms DESC
            LIMIT $3
            "#,
        )
        .bind(model_id)
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .context("Failed to search messages")?;

        let messages: Vec<AgentMessage> = rows
            .into_iter()
            .map(|row| AgentMessage {
                id: row.get("id"),
                conversation_id: row.get("conversation_id"),
                role: row.get("role"),
                content: row.get("content"),
                timestamp_ms: row.get("ts_ms"),
            })
            .collect();

        debug!("Found {} matching messages", messages.len());
        Ok(messages)
    }

    /// 获取会话统计信息
    pub async fn get_session_stats(&self, conversation_id: i64) -> Result<SessionStats> {
        debug!("Getting stats for session {}", conversation_id);

        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_messages,
                COUNT(*) FILTER (WHERE role = 'user') as user_messages,
                COUNT(*) FILTER (WHERE role = 'assistant') as assistant_messages,
                COUNT(*) FILTER (WHERE role = 'system') as system_messages,
                COUNT(*) FILTER (WHERE role = 'tool') as tool_messages,
                MIN(ts_ms) as first_message_time,
                MAX(ts_ms) as last_message_time,
                SUM(LENGTH(content)) as total_characters
            FROM conversation_messages
            WHERE conversation_id = $1
            "#,
        )
        .bind(conversation_id)
        .fetch_one(self.pool.as_ref())
        .await
        .context("Failed to get session stats")?;

        let stats = SessionStats {
            total_messages: row.get("total_messages"),
            user_messages: row.get("user_messages"),
            assistant_messages: row.get("assistant_messages"),
            system_messages: row.get("system_messages"),
            tool_messages: row.get("tool_messages"),
            first_message_time: row.get("first_message_time"),
            last_message_time: row.get("last_message_time"),
            total_characters: row.get::<Option<i64>, _>("total_characters").unwrap_or(0),
        };

        Ok(stats)
    }
}

/// 会话统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_messages: i64,
    pub user_messages: i64,
    pub assistant_messages: i64,
    pub system_messages: i64,
    pub tool_messages: i64,
    pub first_message_time: Option<i64>,
    pub last_message_time: Option<i64>,
    pub total_characters: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意: 这些测试需要一个运行中的 PostgreSQL 数据库
    // 可以使用 docker-compose 启动测试数据库

    #[tokio::test]
    #[ignore] // 默认忽略，需要手动运行
    async fn test_agent_store_lifecycle() {
        // 连接到测试数据库
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://nof0:nof0@localhost:5432/nof0_test".to_string());

        let pool = PgPool::connect(&database_url).await.unwrap();
        let store = AgentStore::new(Arc::new(pool));

        // 创建会话
        let session = store.create_session("test-model").await.unwrap();
        assert_eq!(session.model_id, "test-model");

        // 添加消息
        let msg1 = store
            .add_message(session.id, "user", "Hello, AI!")
            .await
            .unwrap();
        assert_eq!(msg1.role, "user");
        assert_eq!(msg1.content, "Hello, AI!");

        let msg2 = store
            .add_message(session.id, "assistant", "Hello! How can I help you?")
            .await
            .unwrap();
        assert_eq!(msg2.role, "assistant");

        // 获取消息
        let messages = store.get_messages(session.id).await.unwrap();
        assert_eq!(messages.len(), 2);

        // 获取统计
        let stats = store.get_session_stats(session.id).await.unwrap();
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.user_messages, 1);
        assert_eq!(stats.assistant_messages, 1);

        // 删除会话
        let deleted = store.delete_session(session.id).await.unwrap();
        assert!(deleted);
    }
}
