# NOF0 Backend - Rust AI Trading System

> **状态**: 架构已完成，核心模块已创建  
> **语言**: Rust  
> **框架**: Axum + Tokio  
> **目标**: AI 驱动的多市场量化交易系统

---

## 🎯 系统特性

✅ **MCP 协议** - Model Context Protocol 标准实现  
✅ **多 LLM 集成** - OpenAI, Anthropic, DeepSeek, 通义千问等  
✅ **多市场支持** - 数字货币、A股、美股、港股、期货  
✅ **实时交易引擎** - 异步、高并发、低延迟  
✅ **反向代理** - 缓存优化的 API 代理  
✅ **静态服务** - 嵌入式前端资源  
✅ **自动打开浏览器** - 启动时自动打开 Web UI

---

## 📁 模块结构

```
backend/src/
├── main.rs              # 入口: HTTP 服务器 + MCP 初始化
├── config.rs            # 配置管理
├── error.rs             # 统一错误处理
├── mcp/                 # MCP 协议层
│   ├── mod.rs
│   ├── server.rs        # MCP Server 核心
│   ├── transport.rs     # 传输层 (Stdio/SSE/WebSocket)
│   ├── tools.rs         # MCP Tools (get_price, place_order)
│   └── types.rs         # MCP 类型定义
├── llm/                 # LLM 适配层
│   ├── mod.rs
│   └── provider.rs      # LlmProvider trait + 数据结构
├── markets/             # 市场适配层
│   ├── mod.rs
│   └── adapter.rs       # MarketAdapter trait + 数据结构
└── engine/              # 交易引擎
    ├── mod.rs
    ├── trading.rs       # TradingEngine 主逻辑
    ├── agent.rs         # AI Agent 定义
    ├── executor.rs      # 订单执行器
    └── scheduler.rs     # 调度器
```

---

## 🔧 核心组件

### 1. MCP Server

```rust
let mut mcp_server = McpServer::new();
mcp_server.register_tool(GetPriceTool::schema(), Box::new(GetPriceTool));
mcp_server.register_tool(PlaceOrderTool::schema(), Box::new(PlaceOrderTool));
```

**已实现 Tools**:
- `get_price` - 获取市场价格
- `place_order` - 下单

### 2. LLM Provider

```rust
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse>;
    async fn chat_with_tools(&self, req: ChatRequest, tools: Vec<Value>) -> Result<ChatResponse>;
}
```

**计划支持**:
- OpenAI (gpt-4o, o1)
- Anthropic (claude-3-5-sonnet)
- DeepSeek (deepseek-chat)
- 通义千问 (qwen3-max)

### 3. Market Adapter

```rust
pub trait MarketAdapter: Send + Sync {
    async fn get_price(&self, symbol: &str) -> Result<Price>;
    async fn place_order(&self, order: Order) -> Result<String>;
    async fn get_balance(&self, account_id: &str) -> Result<Vec<Balance>>;
}
```

**计划支持**:
- 数字货币: Binance, OKX
- A股: 东方财富, 同花顺
- 美股: IBKR, Alpaca
- 港股: 富途, 老虎
- 期货: CTP

### 4. Trading Engine

```rust
let trading_engine = TradingEngine::new(mcp_server);
trading_engine.register_llm_provider("openai", Box::new(OpenAiProvider));
trading_engine.register_market("binance", Box::new(BinanceAdapter));
```

---

## 🚀 运行

```bash
# 开发模式
cargo run

# 生产模式
cargo build --release
./target/release/nof0-backend
```

**环境变量**:
- `PORT` - 服务器端口 (默认: 8788)
- `NOF1_API_BASE_URL` - 上游 API (默认: https://nof1.ai/api)
- `RUST_LOG` - 日志级别 (默认: info)

---

## 📝 开发计划

### Phase 1: 基础设施 ✅
- [x] 项目结构搭建
- [x] 配置管理
- [x] 错误处理
- [x] 日志系统

### Phase 2: MCP 实现 🔄
- [x] MCP Server 核心
- [x] Tools 定义 (get_price, place_order)
- [ ] Stdio Transport
- [ ] 测试用例

### Phase 3: LLM 集成
- [x] Provider trait
- [ ] OpenAI 实现
- [ ] Anthropic 实现
- [ ] DeepSeek 实现

### Phase 4: 市场适配
- [x] Adapter trait
- [ ] Binance 实现
- [ ] 价格订阅
- [ ] 订单执行

### Phase 5: 交易引擎
- [x] TradingEngine 结构
- [ ] 调度器
- [ ] Agent 管理
- [ ] 交易逻辑

### Phase 6: API 完善
- [ ] REST endpoints
- [ ] WebSocket 推送
- [ ] 前后端联调

---

## 🔗 相关资源

- [架构设计文档](./ARCHITECTURE.md)
- [前端项目](../web/)
- [Go 项目](../go/) - 另一个后端实现

---

## 📦 依赖项

```toml
async-trait = "0.1"      # Trait async 支持
axum = "0.7"             # Web 框架
tokio = "1.40"           # 异步运行时
reqwest = "0.12"         # HTTP 客户端
serde = "1.0"            # 序列化
chrono = "0.4"           # 时间处理
tracing = "0.1"          # 日志
```

---

## 📄 License

MIT
