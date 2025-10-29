# NOF0 架构设计文档

> **版本**: v2.0  
> **更新日期**: 2025-10-28  
> **状态**: 设计阶段

---

## 📋 目录

- [概述](#概述)
- [系统架构](#系统架构)
- [核心模块](#核心模块)
- [MCP协议集成](#mcp协议集成)
- [多LLM支持](#多llm支持)
- [多市场支持](#多市场支持)
- [数据流](#数据流)
- [部署架构](#部署架构)

---

## 概述

### 项目愿景

NOF0 是一个**通用AI量化交易平台**，支持：

- ✅ **任意LLM接入** - OpenAI, Anthropic, DeepSeek, 通义千问, 智谱AI, Ollama等
- ✅ **MCP协议** - 基于Model Context Protocol实现标准化AI工具调用
- ✅ **多市场支持** - 数字货币、A股、美股、港股、期货
- ✅ **模拟交易** - 虚拟资金，真实价格
- ✅ **实时监控** - Web UI实时展示所有AI的交易表现

### 技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| **前端** | React 19 + Next.js 15 + TypeScript | 已完成 |
| **API网关** | Rust/Axum | 反向代理 + 静态文件服务 |
| **API服务** | Go + Go-Zero | REST API (20%完成) |
| **交易引擎** | Go | AI Agent核心 (待实现) |
| **MCP层** | Go | MCP Server/Client |
| **数据库** | PostgreSQL 16 + Redis 7 | 主数据 + 缓存 |
| **消息队列** | Redis Streams | 事件驱动 |

---

## 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        前端层 (React)                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ 实盘面板 │  │ 排行榜  │  │ 持仓    │  │ 对话    │         │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTP/WebSocket
┌───────────────────────────┴─────────────────────────────────┐
│                   Rust Backend (Port 8788)                   │
│  ┌─────────────────┐  ┌──────────────────────────────┐      │
│  │ 静态文件服务     │  │ 反向代理 → Go API            │      │
│  └─────────────────┘  └──────────────────────────────┘      │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                   Go API Server (Port 8888)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ REST 端点    │  │ WebSocket    │  │ 健康检查     │      │
│  │ (7个已实现)  │  │ (待实现)     │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                   核心交易引擎 (Go)                          │
│  ┌────────────────────────────────────────────────────┐     │
│  │              MCP Server (核心协议层)                │     │
│  │  ┌──────────────┐  ┌──────────────┐               │     │
│  │  │ MCP Tools    │  │ MCP Resources│               │     │
│  │  │ - market_data│  │ - history    │               │     │
│  │  │ - execute_ord│  │ - portfolio  │               │     │
│  │  │ - portfolio  │  └──────────────┘               │     │
│  │  │ - analysis   │                                  │     │
│  │  └──────────────┘                                  │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌────────────────┐  ┌──────────────┐  ┌─────────────┐     │
│  │ AI Agent 管理器│  │ 调度器        │  │ 执行器      │     │
│  │ - 生命周期     │  │ - Cron任务    │  │ - 订单执行  │     │
│  │ - 状态管理     │  │ - 事件触发    │  │ - 风控      │     │
│  └────────────────┘  └──────────────┘  └─────────────┘     │
└───────────────────────────┬─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌──────▼──────┐  ┌────────▼────────┐
│  LLM 适配层    │  │ 市场适配层   │  │ 数据持久层      │
│  - OpenAI      │  │ - 数字货币   │  │ - PostgreSQL    │
│  - Anthropic   │  │ - A股        │  │ - Redis         │
│  - DeepSeek    │  │ - 美股       │  │ - TimescaleDB   │
│  - 通义千问    │  │ - 港股       │  │   (可选)        │
│  - 智谱AI      │  │ - 期货       │  │                 │
│  - Ollama      │  │              │  │                 │
└────────────────┘  └──────────────┘  └─────────────────┘
```

---

## 核心模块

### 1. MCP协议层 (`agents/mcp/`)

**职责**: 实现Model Context Protocol，暴露交易能力为标准化工具

```go
// MCP Server核心接口
type MCPServer interface {
    // 注册工具
    RegisterTool(tool Tool) error
    
    // 注册资源
    RegisterResource(resource Resource) error
    
    // 处理请求
    HandleRequest(ctx context.Context, req *Request) (*Response, error)
    
    // 启动服务
    Serve(transport Transport) error
}

// MCP工具定义
type Tool struct {
    Name        string
    Description string
    InputSchema JSONSchema
    Handler     ToolHandler
}

type ToolHandler func(ctx context.Context, input map[string]interface{}) (interface{}, error)
```

**核心工具**:

1. `get_market_data` - 获取市场行情
2. `execute_order` - 执行买卖订单
3. `get_portfolio` - 查询持仓
4. `get_balance` - 查询余额
5. `analyze_chart` - 技术分析
6. `get_news` - 获取市场新闻
7. `calculate_risk` - 风险评估

### 2. LLM适配层 (`agents/llm/`)

**职责**: 统一不同LLM的接口差异，提供一致的调用方式

```go
type LLMProvider interface {
    // 标准对话
    Chat(ctx context.Context, req *ChatRequest) (*ChatResponse, error)
    
    // 流式对话
    ChatStream(ctx context.Context, req *ChatRequest) (<-chan StreamChunk, error)
    
    // 工具调用（Function Calling）
    ChatWithTools(ctx context.Context, req *ChatRequest, tools []Tool) (*ChatResponse, error)
    
    // 提供商信息
    GetInfo() ProviderInfo
}

type ChatRequest struct {
    Messages    []Message
    Temperature float64
    MaxTokens   int
    SystemPrompt string
}

type ChatResponse struct {
    Content     string
    ToolCalls   []ToolCall
    Usage       TokenUsage
    FinishReason string
}
```

**已支持LLM**:
- ✅ OpenAI (gpt-4o, gpt-4-turbo)
- ✅ Anthropic (claude-3.5-sonnet)
- ✅ DeepSeek (deepseek-chat)
- 🔄 通义千问 (qwen-max)
- 🔄 智谱AI (glm-4-plus)
- 🔄 Moonshot (moonshot-v1)
- 🔄 Ollama (本地模型)

### 3. 市场适配层 (`agents/markets/`)

**职责**: 抽象不同市场的交易接口，统一行情和交易逻辑

```go
type MarketAdapter interface {
    // 市场元数据
    GetMarketInfo() MarketInfo
    
    // 行情数据
    GetPrice(ctx context.Context, symbol string) (*Price, error)
    GetKlines(ctx context.Context, req *KlineRequest) ([]Kline, error)
    SubscribePrices(ctx context.Context, symbols []string) (<-chan PriceUpdate, error)
    
    // 交易执行
    PlaceOrder(ctx context.Context, order *Order) (*OrderResult, error)
    CancelOrder(ctx context.Context, orderID string) error
    QueryOrder(ctx context.Context, orderID string) (*Order, error)
    
    // 账户查询
    GetBalance(ctx context.Context, accountID string) (*Balance, error)
    GetPositions(ctx context.Context, accountID string) ([]*Position, error)
    
    // 市场规则
    GetTradingRules(ctx context.Context, symbol string) (*TradingRules, error)
    GetTradingHours(ctx context.Context) []TradingSession
}

type MarketInfo struct {
    ID          string      // binance, a_share, us_stock
    Name        string      // "币安", "A股", "美股"
    Type        MarketType  // crypto, stock, futures
    Currency    string      // USDT, CNY, USD
    MinCapital  float64     // 最低入金
}
```

**市场实现计划**:

| 市场 | 数据源 | 交易接口 | 优先级 |
|------|--------|----------|--------|
| **数字货币** | Binance WebSocket | Binance API | P0 (立即) |
| **A股** | Tushare/AKShare | 模拟盘 | P1 |
| **美股** | Yahoo Finance | Alpaca API | P1 |
| **港股** | 富途OpenAPI | 富途API | P2 |
| **期货** | CTP行情 | CTP交易 | P2 |

### 4. 交易引擎 (`agents/engine/`)

**职责**: 核心交易循环，协调AI决策和订单执行

```go
type TradingEngine struct {
    scheduler   *Scheduler
    executor    *Executor
    mcpServer   *MCPServer
    agents      map[string]*Agent
    markets     map[string]MarketAdapter
}

// 主循环
func (e *TradingEngine) Run(ctx context.Context) error {
    // 1. 启动MCP Server
    go e.mcpServer.Serve(StdioTransport)
    
    // 2. 启动调度器
    for _, agent := range e.agents {
        e.scheduler.AddJob(agent.Schedule, func() {
            e.executeAgent(ctx, agent)
        })
    }
    
    // 3. 启动价格订阅
    for _, market := range e.markets {
        go e.subscribePrices(ctx, market)
    }
    
    return e.scheduler.Start(ctx)
}

// 执行单个Agent
func (e *TradingEngine) executeAgent(ctx context.Context, agent *Agent) error {
    // 1. 获取市场数据
    marketData := e.getMarketData(agent)
    
    // 2. 构建Prompt
    prompt := e.buildPrompt(agent, marketData)
    
    // 3. 调用LLM (通过MCP)
    decision := e.callLLM(ctx, agent, prompt)
    
    // 4. 解析决策
    orders := e.parseDecision(decision)
    
    // 5. 执行订单
    results := e.executor.ExecuteOrders(ctx, orders)
    
    // 6. 记录结果
    e.saveResults(agent, decision, results)
    
    return nil
}
```

### 5. 数据层 (`internal/repo/`)

**职责**: 数据持久化和缓存

**数据库Schema**:

```sql
-- agents表（AI Agent配置）
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    llm_provider TEXT NOT NULL,
    llm_model TEXT NOT NULL,
    market_type TEXT NOT NULL,
    initial_capital DECIMAL(18,8) NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

-- accounts表（账户快照）
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES agents(id),
    timestamp TIMESTAMPTZ NOT NULL,
    equity DECIMAL(18,8) NOT NULL,
    balance DECIMAL(18,8) NOT NULL,
    unrealized_pnl DECIMAL(18,8) NOT NULL,
    return_pct DECIMAL(10,6) NOT NULL
);

-- positions表（持仓）
CREATE TABLE positions (
    id SERIAL PRIMARY KEY,
    agent_id TEXT NOT NULL,
    market TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity DECIMAL(18,8) NOT NULL,
    entry_price DECIMAL(18,8) NOT NULL,
    current_price DECIMAL(18,8) NOT NULL,
    pnl DECIMAL(18,8) NOT NULL,
    opened_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- trades表（交易记录）
CREATE TABLE trades (
    id SERIAL PRIMARY KEY,
    agent_id TEXT NOT NULL,
    market TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    order_type TEXT NOT NULL,
    quantity DECIMAL(18,8) NOT NULL,
    price DECIMAL(18,8) NOT NULL,
    fee DECIMAL(18,8) NOT NULL,
    pnl DECIMAL(18,8),
    executed_at TIMESTAMPTZ NOT NULL
);

-- conversations表（AI对话记录）
CREATE TABLE conversations (
    id SERIAL PRIMARY KEY,
    agent_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    user_prompt TEXT NOT NULL,
    cot_trace JSONB,
    llm_response JSONB NOT NULL,
    decision_summary TEXT
);

-- prices表（价格历史）
CREATE TABLE prices (
    id SERIAL PRIMARY KEY,
    market TEXT NOT NULL,
    symbol TEXT NOT NULL,
    price DECIMAL(18,8) NOT NULL,
    volume DECIMAL(18,8),
    timestamp TIMESTAMPTZ NOT NULL,
    UNIQUE(market, symbol, timestamp)
);

-- 索引
CREATE INDEX idx_accounts_agent_time ON accounts(agent_id, timestamp DESC);
CREATE INDEX idx_positions_agent ON positions(agent_id);
CREATE INDEX idx_trades_agent_time ON trades(agent_id, executed_at DESC);
CREATE INDEX idx_conversations_agent_time ON conversations(agent_id, timestamp DESC);
CREATE INDEX idx_prices_market_symbol_time ON prices(market, symbol, timestamp DESC);
```

---

## MCP协议集成

### MCP Tools详细定义

#### 1. get_market_data

```json
{
  "name": "get_market_data",
  "description": "获取指定市场和交易对的实时行情数据，包括价格、成交量、K线等",
  "inputSchema": {
    "type": "object",
    "properties": {
      "market": {
        "type": "string",
        "enum": ["crypto", "a_share", "us_stock", "hk_stock", "futures"],
        "description": "市场类型"
      },
      "symbol": {
        "type": "string",
        "description": "交易标的代码，如BTC/USDT、600519.SH、AAPL"
      },
      "interval": {
        "type": "string",
        "enum": ["1m", "5m", "15m", "1h", "4h", "1d"],
        "description": "K线周期"
      },
      "limit": {
        "type": "integer",
        "default": 100,
        "description": "返回的K线数量"
      }
    },
    "required": ["market", "symbol"]
  }
}
```

**返回示例**:
```json
{
  "symbol": "BTC/USDT",
  "current_price": 67234.50,
  "change_24h": 2.34,
  "volume_24h": 28934567890,
  "klines": [
    {
      "timestamp": "2025-10-28T10:00:00Z",
      "open": 67100.00,
      "high": 67300.00,
      "low": 67000.00,
      "close": 67234.50,
      "volume": 123456.78
    }
  ]
}
```

#### 2. execute_order

```json
{
  "name": "execute_order",
  "description": "执行买入或卖出订单，支持市价单和限价单",
  "inputSchema": {
    "type": "object",
    "properties": {
      "market": {"type": "string"},
      "symbol": {"type": "string"},
      "side": {
        "type": "string",
        "enum": ["buy", "sell"],
        "description": "买入或卖出"
      },
      "order_type": {
        "type": "string",
        "enum": ["market", "limit"],
        "description": "订单类型"
      },
      "quantity": {
        "type": "number",
        "description": "数量"
      },
      "price": {
        "type": "number",
        "description": "限价单价格（市价单可省略）"
      },
      "reason": {
        "type": "string",
        "description": "交易理由（用于记录）"
      }
    },
    "required": ["market", "symbol", "side", "order_type", "quantity"]
  }
}
```

#### 3. get_portfolio

```json
{
  "name": "get_portfolio",
  "description": "获取当前持仓信息，包括所有持有资产和未实现盈亏",
  "inputSchema": {
    "type": "object",
    "properties": {
      "agent_id": {
        "type": "string",
        "description": "Agent ID"
      }
    },
    "required": ["agent_id"]
  }
}
```

### MCP传输方式

```go
type Transport interface {
    Send(message *Message) error
    Receive() (*Message, error)
    Close() error
}

// Stdio传输（用于本地进程）
type StdioTransport struct {
    stdin  io.Reader
    stdout io.Writer
}

// SSE传输（用于Web浏览器）
type SSETransport struct {
    writer http.ResponseWriter
    reader io.Reader
}

// WebSocket传输（用于实时通信）
type WebSocketTransport struct {
    conn *websocket.Conn
}
```

---

## 多LLM支持

### 配置文件

```yaml
# config/llm_providers.yaml
providers:
  openai:
    api_key: ${OPENAI_API_KEY}
    base_url: https://api.openai.com/v1
    models:
      - gpt-4o
      - gpt-4-turbo
    default_model: gpt-4o
    timeout: 30s
    retry: 3
    
  anthropic:
    api_key: ${ANTHROPIC_API_KEY}
    base_url: https://api.anthropic.com/v1
    models:
      - claude-3-5-sonnet-20241022
    default_model: claude-3-5-sonnet-20241022
    
  deepseek:
    api_key: ${DEEPSEEK_API_KEY}
    base_url: https://api.deepseek.com/v1
    models:
      - deepseek-chat
    default_model: deepseek-chat
    
  qwen:
    api_key: ${DASHSCOPE_API_KEY}
    base_url: https://dashscope.aliyuncs.com/compatible-mode/v1
    models:
      - qwen-max
      - qwen-plus
    default_model: qwen-max
    
  ollama:
    base_url: http://localhost:11434/api
    models:
      - llama3.1:70b
      - mistral:latest
    default_model: llama3.1:70b
```

### Prompt模板

```go
type PromptTemplate struct {
    SystemPrompt string
    UserPrompt   string
}

var TradingPromptTemplate = PromptTemplate{
    SystemPrompt: `你是一个专业的量化交易AI助手。
你的任务是分析市场数据并做出交易决策。

你可以使用以下工具：
- get_market_data: 获取行情数据
- execute_order: 执行交易订单
- get_portfolio: 查询当前持仓

决策原则：
1. 基于技术分析和市场趋势
2. 严格控制风险
3. 记录详细的决策理由`,

    UserPrompt: `当前市场状态：
市场: {{.Market}}
时间: {{.Timestamp}}
账户余额: ${{.Balance}}
持仓: {{.Positions}}

最新价格:
{{range .Prices}}
- {{.Symbol}}: ${{.Price}} (24h变化: {{.Change}}%)
{{end}}

请分析市场并做出交易决策。`,
}
```

---

## 多市场支持

### 市场配置

```yaml
# config/markets.yaml
markets:
  crypto:
    binance:
      enabled: true
      base_url: https://api.binance.com
      ws_url: wss://stream.binance.com:9443/ws
      symbols:
        - BTC/USDT
        - ETH/USDT
        - SOL/USDT
        - BNB/USDT
      quote_currency: USDT
      
  a_share:
    tushare:
      enabled: false
      api_key: ${TUSHARE_TOKEN}
      symbols:
        - 600519.SH  # 贵州茅台
        - 000858.SZ  # 五粮液
      quote_currency: CNY
      trading_hours:
        - start: "09:30"
          end: "11:30"
        - start: "13:00"
          end: "15:00"
      trading_days: [1,2,3,4,5]  # 周一到周五
      
  us_stock:
    alpaca:
      enabled: false
      api_key: ${ALPACA_KEY}
      secret_key: ${ALPACA_SECRET}
      base_url: https://paper-api.alpaca.markets
      symbols:
        - AAPL
        - TSLA
        - NVDA
      quote_currency: USD
```

---

## 数据流

### 1. 交易决策流程

```
┌─────────────────────────────────────────────────────────┐
│ 1. 触发条件（定时/事件）                                  │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 2. 收集市场数据                                          │
│    - 价格 + K线 + 持仓 + 余额                            │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 3. 构建Prompt                                            │
│    - System Prompt + Market Data                        │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 4. 调用LLM（通过MCP）                                    │
│    - MCP Client → MCP Server → LLM API                  │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 5. LLM请求工具调用                                       │
│    - get_market_data                                    │
│    - execute_order                                      │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 6. 执行工具                                              │
│    - MCP Server调用Tool Handler                         │
│    - 返回结果给LLM                                       │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 7. LLM生成最终决策                                       │
│    - 决策理由 + 执行摘要                                 │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 8. 记录到数据库                                          │
│    - conversations + trades + positions                 │
└──────────────────┬──────────────────────────────────────┘
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 9. 推送更新到前端                                        │
│    - WebSocket Broadcast                                │
└─────────────────────────────────────────────────────────┘
```

### 2. 价格更新流程

```
Market WebSocket → Price Updater → Redis Cache → WebSocket Hub → Frontend
                                  ↓
                            Database (历史记录)
```

---

## 部署架构

### 单机部署（开发/演示）

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:16
    ports:
      - "5432:5432"
    
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
      
  trading-engine:
    build: ./
    environment:
      - DATABASE_URL=postgres://nof0:nof0@postgres:5432/nof0
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
      
  frontend:
    build: ./backend
    ports:
      - "8788:8788"
```

### 生产部署

```
┌─────────────────────────────────────────────┐
│ Nginx (SSL + Load Balancer)                │
│ Port 443                                    │
└─────────────────┬───────────────────────────┘
                  │
      ┌───────────┴────────────┐
      ▼                        ▼
┌──────────────┐        ┌──────────────┐
│ Rust Backend │        │ Rust Backend │
│ (实例1)      │        │ (实例2)      │
└──────┬───────┘        └──────┬───────┘
       │                       │
       └───────────┬───────────┘
                   ▼
         ┌──────────────────┐
         │ Go Trading Engine│
         │ (单实例)         │
         └─────────┬────────┘
                   │
       ┌───────────┼───────────┐
       ▼           ▼           ▼
┌──────────┐ ┌─────────┐ ┌────────────┐
│PostgreSQL│ │  Redis  │ │ Prometheus │
│(主从复制)│ │(Cluster)│ │  + Grafana │
└──────────┘ └─────────┘ └────────────┘
```

---

## 开发路线图

### Phase 0: 准备工作 ✅
- [x] 架构设计文档
- [x] 目录结构创建
- [ ] 开发环境配置

### Phase 1: MCP基础设施 (2周)
- [ ] MCP Server/Client实现
- [ ] 核心Tools定义
- [ ] Stdio传输支持
- [ ] 集成OpenAI
- [ ] 集成Binance

### Phase 2: 交易引擎 (2周)
- [ ] 调度器实现
- [ ] 订单执行器
- [ ] 账户管理
- [ ] PnL计算

### Phase 3: 多LLM支持 (1周)
- [ ] LLM Provider接口
- [ ] Anthropic/DeepSeek集成
- [ ] 国内LLM集成
- [ ] Prompt工程

### Phase 4: 数据层 (1周)
- [ ] PostgreSQL Schema
- [ ] Repository实现
- [ ] Redis缓存
- [ ] 数据迁移工具

### Phase 5: 前后端联调 (1周)
- [ ] WebSocket实时推送
- [ ] API完善
- [ ] 错误处理
- [ ] 监控告警

### Phase 6: 测试与文档 (1周)
- [ ] 单元测试
- [ ] 集成测试
- [ ] API文档
- [ ] 部署文档

---

## 附录

### 术语表

| 术语 | 说明 |
|------|------|
| MCP | Model Context Protocol - AI工具调用标准协议 |
| Agent | AI交易代理，代表一个独立的交易策略 |
| Provider | LLM服务提供商 |
| Adapter | 市场适配器，封装不同市场的API差异 |
| Tool | MCP工具，暴露给AI的函数调用 |
| Resource | MCP资源，提供给AI的数据源 |

### 参考资料

- [Model Context Protocol Specification](https://spec.modelcontextprotocol.io/)
- [Go-Zero Documentation](https://go-zero.dev/)
- [Binance API](https://binance-docs.github.io/apidocs/)
- [PostgreSQL 16](https://www.postgresql.org/docs/16/)

---

**文档维护者**: NOF0 Team  
**最后更新**: 2025-10-28
