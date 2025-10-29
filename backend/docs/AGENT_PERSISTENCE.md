# Agent Persistence - Agent 持久化系统

## 概述

Agent 持久化系统允许 AI Agent 的对话历史和状态被保存到数据库，实现：
- 📝 **对话历史存储** - 永久保存所有对话
- 🔄 **会话恢复** - 继续之前的对话
- 🔍 **历史搜索** - 查找历史对话内容
- 📊 **会话统计** - 分析对话数据
- 🗑️ **会话管理** - 删除或归档旧会话

## 数据库架构

使用现有的 PostgreSQL 表：

```sql
-- 会话表
CREATE TABLE conversations (
    id        bigserial PRIMARY KEY,
    model_id  text NOT NULL REFERENCES models(id)
);

-- 消息表
CREATE TABLE conversation_messages (
    id              bigserial PRIMARY KEY,
    conversation_id bigint NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            text NOT NULL CHECK (role IN ('system','user','assistant','tool')),
    content         text NOT NULL,
    ts_ms           bigint
);
```

### 关系说明

```
models (模型) 1 ──→ N conversations (会话)
                         │
                         1
                         │
                         ↓
                         N
              conversation_messages (消息)
```

## 核心组件

### 1. AgentStore

负责所有数据库操作的主要组件。

```rust
pub struct AgentStore {
    pool: Arc<PgPool>,
}

impl AgentStore {
    // 创建新会话
    pub async fn create_session(&self, model_id: &str) -> Result<AgentSession>
    
    // 添加单条消息
    pub async fn add_message(&self, conversation_id: i64, role: &str, content: &str) 
        -> Result<AgentMessage>
    
    // 批量添加消息
    pub async fn add_messages(&self, conversation_id: i64, messages: &[Message]) 
        -> Result<Vec<AgentMessage>>
    
    // 获取会话所有消息
    pub async fn get_messages(&self, conversation_id: i64) 
        -> Result<Vec<AgentMessage>>
    
    // 列出模型的所有会话
    pub async fn list_sessions(&self, model_id: &str) 
        -> Result<Vec<SessionSummary>>
    
    // 搜索消息
    pub async fn search_messages(&self, model_id: &str, query: &str, limit: i64) 
        -> Result<Vec<AgentMessage>>
    
    // 获取会话统计
    pub async fn get_session_stats(&self, conversation_id: i64) 
        -> Result<SessionStats>
    
    // 删除会话
    pub async fn delete_session(&self, conversation_id: i64) -> Result<bool>
}
```

### 2. 数据结构

#### AgentSession - 会话信息
```rust
pub struct AgentSession {
    pub id: i64,
    pub model_id: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

#### AgentMessage - 消息记录
```rust
pub struct AgentMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub timestamp_ms: i64,
}
```

#### SessionSummary - 会话摘要
```rust
pub struct SessionSummary {
    pub session: AgentSession,
    pub message_count: i64,
    pub last_message_time: Option<i64>,
    pub first_message: Option<String>,
}
```

#### SessionStats - 会话统计
```rust
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
```

## 使用示例

### 场景 1: 创建新会话

```rust
use nof0_backend::engine::AgentStore;
use sqlx::PgPool;
use std::sync::Arc;

// 连接数据库
let pool = PgPool::connect(&database_url).await?;
let store = AgentStore::new(Arc::new(pool));

// 创建会话
let session = store.create_session("gpt-4").await?;
println!("Created session: {}", session.id);

// 添加用户消息
store.add_message(session.id, "user", "Hello, AI!").await?;

// 添加 AI 回复
store.add_message(session.id, "assistant", "Hello! How can I help?").await?;
```

### 场景 2: 恢复会话

```rust
// 列出所有会话
let sessions = store.list_sessions("gpt-4").await?;

// 选择最近的会话
let session = &sessions[0].session;

// 加载历史消息
let history = store.get_messages(session.id).await?;

// 转换为 LLM 消息格式
let llm_messages = store.agent_messages_to_llm_messages(history);

// 继续对话...
```

### 场景 3: 搜索历史

```rust
// 搜索包含 "BTC" 的消息
let messages = store.search_messages("gpt-4", "BTC", 10).await?;

for msg in messages {
    println!("{}: {}", msg.role, msg.content);
}
```

### 场景 4: 会话统计

```rust
// 获取统计信息
let stats = store.get_session_stats(session.id).await?;

println!("Total messages: {}", stats.total_messages);
println!("User messages: {}", stats.user_messages);
println!("Assistant messages: {}", stats.assistant_messages);
println!("Tool messages: {}", stats.tool_messages);
```

## 与 ToolExecutor 集成

### 自动持久化多轮对话

```rust
use nof0_backend::engine::{AgentStore, ToolExecutor};

// 1. 创建会话
let session = store.create_session("gpt-4").await?;

// 2. 添加用户消息
let user_msg = "Check BTC price and buy if below $70k";
store.add_message(session.id, "user", user_msg).await?;

// 3. 执行多轮对话
let request = ChatRequest {
    messages: vec![Message {
        role: "user".to_string(),
        content: user_msg.to_string(),
    }],
    temperature: Some(0.7),
    max_tokens: Some(1500),
};

let result = executor.execute_dialogue(request, tools, chat_fn).await?;

// 4. 保存所有对话历史
store.add_messages(session.id, &result.message_history[1..]).await?;

// 5. 现在所有对话都已持久化！
```

### 恢复并继续对话

```rust
// 1. 加载历史
let history = store.get_messages(session.id).await?;
let mut llm_messages = store.agent_messages_to_llm_messages(history);

// 2. 添加新消息
let new_user_msg = "What did I just ask you to do?";
store.add_message(session.id, "user", new_user_msg).await?;
llm_messages.push(Message {
    role: "user".to_string(),
    content: new_user_msg.to_string(),
});

// 3. 继续对话
let request = ChatRequest {
    messages: llm_messages,
    temperature: Some(0.7),
    max_tokens: Some(1500),
};

let result = executor.execute_dialogue(request, tools, chat_fn).await?;

// 4. 保存新消息
let new_messages = &result.message_history[history.len()..];
store.add_messages(session.id, new_messages).await?;
```

## 完整示例

参见 `backend/examples/persistent_agent_demo.rs`，包含四个完整场景：

### 场景 1: 新建对话并持久化
- 创建新会话
- 执行多轮对话
- 保存所有消息
- 显示统计信息

### 场景 2: 恢复之前的对话
- 列出所有会话
- 加载历史消息
- 继续对话
- 保存新消息

### 场景 3: 搜索历史记录
- 按关键词搜索
- 跨会话搜索
- 显示匹配结果

### 场景 4: 会话管理
- 列出所有会话
- 显示会话统计
- 删除旧会话

## 运行示例

### 前置条件

1. **启动 PostgreSQL**
```bash
# 使用 Docker
cd go
docker-compose up -d postgres

# 或者使用本地 PostgreSQL
# 确保有 nof0 数据库
```

2. **运行迁移**
```bash
cd go
# 使用你喜欢的迁移工具运行 migrations/*.sql
```

3. **设置环境变量**
```bash
# Windows PowerShell
$env:DATABASE_URL = "postgres://nof0:nof0@localhost:5432/nof0"
$env:OPENAI_API_KEY = "sk-..."

# Linux/Mac
export DATABASE_URL="postgres://nof0:nof0@localhost:5432/nof0"
export OPENAI_API_KEY="sk-..."
```

### 运行

```bash
cd backend
cargo run --example persistent_agent_demo
```

### 预期输出

```
╔══════════════════════════════════════════╗
║   Persistent Agent Demo                 ║
║   Conversation History Management       ║
╚══════════════════════════════════════════╝

📡 Connecting to database...
✅ Connected to database

✅ All systems initialized

=== Scenario 1: New Conversation with Persistence ===

✅ Created session: 42
👤 User: Check BTC price and if it's below $70000, buy 0.1 BTC

💾 Saving conversation to database...
✅ Saved 6 messages

🤖 Assistant: I checked the BTC price which is $67,500.00...

📊 Session Stats:
  Total messages: 6
  User messages: 1
  Assistant messages: 2
  Tool messages: 3
  Total characters: 1234

=== Scenario 2: Resume Previous Conversation ===

Found 1 previous sessions:
  1. Session 42 - 6 messages - First: Check BTC price...

📖 Resuming session 42
Loaded 6 historical messages

📜 Recent history:
  👤 user: Check BTC price and if it's below $70000, buy 0.1 BTC
  🔧 tool: Tool 'get_crypto_price' executed successfully...
  🤖 assistant: I checked the BTC price...

👤 User: What was the last action I took? And what's the current BTC price now?

💾 Saving new messages...
✅ Saved 3 new messages

🤖 Assistant: Your last action was to buy 0.1 BTC...

=== Scenario 3: Search Conversation History ===

🔍 Searching for: 'BTC'
Found 5 messages:

1. [Session 42] 👤 user:
   Check BTC price and if it's below $70000, buy 0.1 BTC

2. [Session 42] 🔧 tool:
   Tool 'get_crypto_price' executed successfully: {"symbol":"BTC",...}

...

=== Scenario 4: Session Management ===

📋 Total sessions: 1

Session 42:
  Messages: 9
  Last activity: 5432 ms ago
  User msgs: 2, Assistant msgs: 4, Tool msgs: 3

✅ All scenarios completed!
```

## 数据库优化

### 索引

现有索引（来自 001_domain.sql）：
```sql
CREATE INDEX idx_conv_msgs_conv_ts 
ON conversation_messages(conversation_id, ts_ms);
```

### 推荐的额外索引

```sql
-- 加速按 model_id 查询会话
CREATE INDEX idx_conversations_model 
ON conversations(model_id);

-- 加速全文搜索（如果需要）
CREATE INDEX idx_conv_msgs_content_trgm 
ON conversation_messages USING gin(content gin_trgm_ops);
```

### 分区策略（大规模场景）

对于大量历史数据，可以按时间分区：

```sql
-- 按月分区
CREATE TABLE conversation_messages_2025_01 
PARTITION OF conversation_messages 
FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE TABLE conversation_messages_2025_02 
PARTITION OF conversation_messages 
FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');
```

## 性能考虑

### 1. 批量插入
使用 `add_messages` 而不是循环调用 `add_message`：

```rust
// ❌ 慢
for msg in messages {
    store.add_message(session_id, &msg.role, &msg.content).await?;
}

// ✅ 快（如果实现了真正的批量插入）
store.add_messages(session_id, &messages).await?;
```

### 2. 连接池
使用 sqlx 的连接池：

```rust
let pool = PgPool::connect(&database_url).await?;
// pool 自动管理连接
```

### 3. 分页查询
对于大量消息，使用分页：

```rust
pub async fn get_messages_paged(
    &self, 
    conversation_id: i64,
    offset: i64,
    limit: i64
) -> Result<Vec<AgentMessage>> {
    sqlx::query(...)
        .bind(conversation_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(...)
        .await
}
```

## 错误处理

所有方法都返回 `Result<T, anyhow::Error>`：

```rust
match store.create_session("gpt-4").await {
    Ok(session) => println!("Created: {}", session.id),
    Err(e) => eprintln!("Failed to create session: {}", e),
}
```

## 日志

使用 tracing 记录关键操作：

```rust
info!("Creating new session for model: {}", model_id);
debug!("Adding message to conversation {}", conversation_id);
warn!("Deleting session {}", conversation_id);
```

## 测试

### 单元测试

```rust
#[tokio::test]
#[ignore] // 需要数据库
async fn test_agent_store_lifecycle() {
    let pool = PgPool::connect(&test_db_url).await.unwrap();
    let store = AgentStore::new(Arc::new(pool));
    
    // 测试创建
    let session = store.create_session("test").await.unwrap();
    
    // 测试添加消息
    store.add_message(session.id, "user", "test").await.unwrap();
    
    // 测试查询
    let messages = store.get_messages(session.id).await.unwrap();
    assert_eq!(messages.len(), 1);
    
    // 测试删除
    store.delete_session(session.id).await.unwrap();
}
```

运行测试：
```bash
# 设置测试数据库
export TEST_DATABASE_URL="postgres://nof0:nof0@localhost:5432/nof0_test"

# 运行测试
cargo test --lib -- --ignored
```

## 未来增强

### 短期
1. **真正的批量插入** - 使用 PostgreSQL COPY 或批量 INSERT
2. **消息修改** - 编辑或删除单条消息
3. **会话标签** - 为会话添加标签便于分类
4. **导出功能** - 导出为 JSON/CSV

### 中期
1. **全文搜索** - 使用 PostgreSQL FTS 或 Elasticsearch
2. **消息嵌入** - 存储消息的向量嵌入用于语义搜索
3. **会话快照** - 定期保存会话状态快照
4. **自动归档** - 自动归档旧会话到冷存储

### 长期
1. **分布式存储** - 支持多数据库分片
2. **实时同步** - WebSocket 实时推送消息更新
3. **审计日志** - 记录所有修改操作
4. **版本控制** - 消息版本历史

## 相关文档

- [TOOL_EXECUTOR.md](./TOOL_EXECUTOR.md) - Tool Executor 文档
- [TRADING_ENGINE_INTEGRATION.md](./TRADING_ENGINE_INTEGRATION.md) - 交易引擎集成

## 数据库模式参考

完整的数据库模式见：
- `go/migrations/001_domain.sql` - 主表定义
- `go/migrations/002_refresh_helpers.sql` - 辅助函数

## 贡献

欢迎贡献！特别是：
- 性能优化
- 更多查询方法
- 测试用例
- 文档改进

---

**享受持久化的 AI 对话！** 💾🤖✨
