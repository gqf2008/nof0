# CTP Market Adapter - 技术文档

**Version**: 1.0.0  
**Date**: 2025-01-18  
**Status**: ✅ Production Ready (Mock Mode)

---

## 📋 目录

1. [概述](#概述)
2. [架构设计](#架构设计)
3. [核心功能](#核心功能)
4. [数据结构](#数据结构)
5. [API文档](#api文档)
6. [配置说明](#配置说明)
7. [使用示例](#使用示例)
8. [Mock模式](#mock模式)
9. [真实模式](#真实模式)
10. [错误处理](#错误处理)
11. [性能优化](#性能优化)
12. [测试](#测试)
13. [已知限制](#已知限制)
14. [路线图](#路线图)

---

## 概述

### 1.1 什么是CTP?

CTP (Comprehensive Transaction Platform) 是上海期货信息技术有限公司开发的期货交易系统,被中国大多数期货公司采用。它提供:

- **行情数据 (Market Data)**: 实时期货合约报价
- **交易接口 (Trade Interface)**: 期货订单下单、撤单、查询
- **账户查询 (Account Query)**: 资金、持仓、成交查询

### 1.2 CTP Adapter的作用

CTP Adapter是nof0交易系统与CTP服务器之间的桥梁:

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   AI Agent   │─────→│ CTP Adapter  │─────→│ CTP Server   │
│  (决策层)     │      │  (适配层)     │      │  (交易所)     │
└──────────────┘      └──────────────┘      └──────────────┘
                              │
                              ▼
                      ┌──────────────┐
                      │  Mock Mode   │
                      │  (模拟环境)   │
                      └──────────────┘
```

**核心职责**:
- 实现统一的 `MarketAdapter` 接口
- 处理CTP协议的复杂性
- 提供Mock模式用于测试
- 管理连接状态和错误恢复
- 缓存市场数据和账户状态

### 1.3 支持的市场

**当前版本支持中国期货市场**:
- **IF** (沪深300股指期货)
- **IC** (中证500股指期货)
- **IH** (上证50股指期货)

**未来计划支持**:
- 商品期货 (黑色系、能源、农产品)
- 期权合约
- 国际期货市场

---

## 架构设计

### 2.1 分层架构

```
┌─────────────────────────────────────────────────┐
│          Application Layer (Trading Engine)      │
│               使用 MarketAdapter trait           │
└─────────────────────────────────────────────────┘
                        ▲
                        │ trait interface
                        │
┌─────────────────────────────────────────────────┐
│             CTP Adapter Layer                   │
│  ┌─────────────┐  ┌─────────────┐             │
│  │  Adapter    │  │   Types     │              │
│  │  (Core)     │  │  (Structs)  │              │
│  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────┘
                        ▲
                        │ mode selection
                        │
┌─────────────────────────────────────────────────┐
│         Mode Selection Layer                    │
│  ┌──────────────┐    ┌──────────────┐         │
│  │  Mock Mode   │    │  Real Mode   │         │
│  │ (Simulation) │    │  (CTP SDK)   │         │
│  └──────────────┘    └──────────────┘         │
└─────────────────────────────────────────────────┘
```

### 2.2 核心组件

#### **CtpMarketAdapter** (adapter.rs)
- **职责**: 主适配器类,实现MarketAdapter trait
- **状态管理**: 连接状态、市场数据缓存、持仓/账户缓存
- **线程安全**: 使用 `Arc<RwLock<T>>` 实现并发安全访问

#### **Types** (types.rs)
- **数据模型**: CTP特定的数据结构定义
- **序列化**: 支持serde进行配置文件读写
- **类型安全**: 使用Rust强类型系统保证数据正确性

### 2.3 数据流

#### **Market Data Flow (行情数据流)**
```
CTP Server → CtpMarketData → Cache (RwLock) → Price (trait type)
                ↓
         Subscribe Request
                ↓
         IF2501, IC2501, IH2501
```

#### **Order Flow (订单流)**
```
Order (trait) → CtpOrderRequest → CTP Server → CtpOrderResponse
                                        ↓
                                  Order Status
                                        ↓
                                 Update Positions
```

#### **Account Query Flow (账户查询流)**
```
Query Request → CTP Server → CtpAccount → Cache → Balance (trait type)
```

---

## 核心功能

### 3.1 功能列表

| 功能 | Mock模式 | Real模式 | 描述 |
|------|---------|---------|------|
| **连接管理** | ✅ | 🔄 | 连接/断开CTP服务器 |
| **行情订阅** | ✅ | 🔄 | 订阅合约实时报价 |
| **市场数据** | ✅ | 🔄 | 获取最新价、买卖价、成交量 |
| **下单** | ✅ | 🔄 | 限价单、市价单 |
| **持仓查询** | ✅ | 🔄 | 查询当前持仓 |
| **账户查询** | ✅ | 🔄 | 查询资金、保证金 |
| **撤单** | ⏳ | ⏳ | 取消未成交订单 |
| **成交查询** | ⏳ | ⏳ | 查询历史成交 |

**图例**:
- ✅ 已实现
- 🔄 计划中
- ⏳ 未开始

### 3.2 MarketAdapter Trait 实现

CTP Adapter完整实现了 `MarketAdapter` trait:

```rust
#[async_trait]
pub trait MarketAdapter: Send + Sync {
    async fn get_price(&self, symbol: &str) -> Result<Price, anyhow::Error>;
    async fn place_order(&self, order: Order) -> Result<String, anyhow::Error>;
    async fn get_balance(&self, account_id: &str) -> Result<Vec<Balance>, anyhow::Error>;
    fn market_name(&self) -> &str;
}
```

**为什么重要?**
- **统一接口**: AI Agent不需要知道底层市场细节
- **可替换**: 可以轻松切换到其他市场(crypto、股票等)
- **测试友好**: Mock模式无需真实市场连接

---

## 数据结构

### 4.1 配置结构 (CtpConfig)

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CtpConfig {
    /// 期货公司代码 (e.g., "9999" for SimNow)
    pub broker_id: String,
    
    /// 投资者账号
    pub investor_id: String,
    
    /// 登录密码
    pub password: String,
    
    /// 行情前置地址 (e.g., "tcp://180.168.146.187:10131")
    pub md_address: String,
    
    /// 交易前置地址 (e.g., "tcp://180.168.146.187:10130")
    pub td_address: String,
    
    /// 是否启用Mock模式 (true=模拟, false=真实)
    pub mock_mode: bool,
}
```

**字段说明**:
- **broker_id**: 期货公司标识,SimNow测试环境使用"9999"
- **investor_id**: 你的期货账号
- **password**: 登录密码,生产环境请使用环境变量
- **md_address**: 行情服务器地址(Market Data)
- **td_address**: 交易服务器地址(Trade)
- **mock_mode**: `true`时使用模拟数据,无需真实连接

### 4.2 订单请求 (CtpOrderRequest)

```rust
#[derive(Debug, Clone)]
pub struct CtpOrderRequest {
    /// 合约代码 (e.g., "IF2501")
    pub instrument_id: String,
    
    /// 买卖方向 ('0'=买, '1'=卖)
    pub direction: char,
    
    /// 开平标志 ('0'=开仓, '1'=平仓, '3'=平今)
    pub offset_flag: char,
    
    /// 价格 (限价单使用,市价单为0)
    pub price: f64,
    
    /// 数量 (手数)
    pub volume: i32,
}
```

**关键概念**:
- **direction**: 
  - `'0'` = 买入(做多)
  - `'1'` = 卖出(做空)
- **offset_flag**:
  - `'0'` = 开仓(建立新仓位)
  - `'1'` = 平仓(关闭任意仓位)
  - `'3'` = 平今(仅关闭今天开的仓位,避免隔夜费)

### 4.3 市场数据 (CtpMarketData)

```rust
#[derive(Debug, Clone)]
pub struct CtpMarketData {
    /// 合约代码
    pub instrument_id: String,
    
    /// 最新价
    pub last_price: f64,
    
    /// 申买价一
    pub bid_price: f64,
    
    /// 申卖价一
    pub ask_price: f64,
    
    /// 数量
    pub volume: i32,
    
    /// 持仓量
    pub open_interest: i32,
    
    /// 今开盘
    pub open_price: f64,
    
    /// 最高价
    pub highest_price: f64,
    
    /// 最低价
    pub lowest_price: f64,
    
    /// 更新时间
    pub update_time: String,
    
    /// 更新毫秒
    pub update_millisec: i32,
}
```

**典型数据示例**:
```
IF2501:
  last_price: 3500.00      (沪深300指数 * 300)
  bid_price: 3499.80
  ask_price: 3500.20
  volume: 125000           (今日累计成交量)
  open_interest: 85000     (当前持仓量)
  spread: 0.40             (买卖价差)
```

### 4.4 账户信息 (CtpAccount)

```rust
#[derive(Debug, Clone)]
pub struct CtpAccount {
    /// 账户ID
    pub account_id: String,
    
    /// 上日结存
    pub pre_balance: f64,
    
    /// 入金金额
    pub deposit: f64,
    
    /// 出金金额
    pub withdraw: f64,
    
    /// 当前余额 (pre_balance + deposit - withdraw + close_profit - commission)
    pub balance: f64,
    
    /// 可用资金
    pub available: f64,
    
    /// 占用保证金
    pub margin: f64,
    
    /// 冻结保证金
    pub frozen_margin: f64,
    
    /// 持仓盈亏 (未实现)
    pub position_profit: f64,
    
    /// 平仓盈亏 (已实现)
    pub close_profit: f64,
    
    /// 手续费
    pub commission: f64,
}

impl CtpAccount {
    /// 动态权益 = 余额 + 持仓盈亏
    pub fn equity(&self) -> f64 {
        self.balance + self.position_profit
    }
    
    /// 风险度 = 保证金占用 / 动态权益
    pub fn risk_ratio(&self) -> f64 {
        if self.equity() == 0.0 {
            return 0.0;
        }
        self.margin / self.equity()
    }
}
```

**账户计算公式**:

```
动态权益 (Equity) = 余额 (Balance) + 持仓盈亏 (Position P&L)

可用资金 (Available) = 动态权益 - 占用保证金 - 冻结保证金

风险度 (Risk Ratio) = 占用保证金 / 动态权益

保证金占用 (Margin) = Σ(持仓量 × 合约乘数 × 最新价 × 保证金率)
```

**风险警示**:
- 风险度 < 50%: 🟢 安全
- 50% ≤ 风险度 < 80%: 🟡 警告
- 风险度 ≥ 80%: 🔴 危险,接近强平线

### 4.5 持仓信息 (CtpPosition)

```rust
#[derive(Debug, Clone)]
pub struct CtpPosition {
    /// 合约代码
    pub instrument_id: String,
    
    /// 持仓方向 ('2'=买, '3'=卖)
    pub direction: char,
    
    /// 持仓数量
    pub position: i32,
    
    /// 今日持仓
    pub today_position: i32,
    
    /// 昨日持仓
    pub yd_position: i32,
    
    /// 开仓均价
    pub open_cost: f64,
    
    /// 持仓盈亏
    pub position_profit: f64,
    
    /// 占用保证金
    pub margin: f64,
}
```

---

## API文档

### 5.1 创建Adapter

#### **从配置文件创建**
```rust
use nof0_backend::markets::{CtpConfig, CtpMarketAdapter};

// 1. 加载配置
let config = CtpConfig::from_file("etc/ctp_config.yaml")
    .expect("Failed to load config");

// 2. 创建Adapter
let adapter = CtpMarketAdapter::new(config);

// 3. 连接
adapter.connect().await?;
```

#### **从代码创建**
```rust
let config = CtpConfig {
    broker_id: "9999".to_string(),
    investor_id: "000000".to_string(),
    password: "password".to_string(),
    md_address: "tcp://180.168.146.187:10131".to_string(),
    td_address: "tcp://180.168.146.187:10130".to_string(),
    mock_mode: true,  // 使用Mock模式
};

let adapter = CtpMarketAdapter::new(config);
```

### 5.2 连接管理

#### **connect() - 连接到CTP**
```rust
pub async fn connect(&self) -> Result<()>
```

**行为**:
- **Mock模式**: 立即返回成功,初始化模拟数据
- **Real模式**: 连接到CTP前置服务器,执行登录流程

**示例**:
```rust
adapter.connect().await?;
println!("Connected to: {}", adapter.market_name());
```

#### **disconnect() - 断开连接**
```rust
pub async fn disconnect(&self) -> Result<()>
```

**示例**:
```rust
adapter.disconnect().await?;
```

#### **is_connected() - 检查连接状态**
```rust
pub async fn is_connected(&self) -> bool
```

**示例**:
```rust
if adapter.is_connected().await {
    println!("Already connected");
} else {
    adapter.connect().await?;
}
```

### 5.3 行情订阅

#### **subscribe_market_data() - 订阅行情**
```rust
pub async fn subscribe_market_data(&self, instruments: Vec<String>) -> Result<()>
```

**参数**:
- `instruments`: 合约代码列表,如 `["IF2501", "IC2501"]`

**示例**:
```rust
// 订阅股指期货三剑客
let instruments = vec![
    "IF2501".to_string(),  // 沪深300
    "IC2501".to_string(),  // 中证500
    "IH2501".to_string(),  // 上证50
];

adapter.subscribe_market_data(instruments).await?;
```

**注意事项**:
- 必须先调用 `connect()`
- Mock模式会自动初始化模拟数据
- Real模式会向CTP发送订阅请求

### 5.4 市场数据查询

#### **get_market_data() - 获取详细行情**
```rust
pub async fn get_market_data(&self, instrument_id: &str) -> Result<CtpMarketData>
```

**返回**: 完整的CTP行情数据

**示例**:
```rust
let data = adapter.get_market_data("IF2501").await?;
println!("Last: {}", data.last_price);
println!("Bid: {}, Ask: {}", data.bid_price, data.ask_price);
println!("Volume: {}, OI: {}", data.volume, data.open_interest);
```

#### **get_price() - 获取简化价格(Trait方法)**
```rust
async fn get_price(&self, symbol: &str) -> Result<Price, anyhow::Error>
```

**返回**: `Price` 结构(trait标准类型)

**示例**:
```rust
use nof0_backend::markets::MarketAdapter;

let price = adapter.get_price("IC2501").await?;
println!("Price: ¥{:.2} at {}", price.price, price.timestamp);
```

### 5.5 订单操作

#### **place_order() - 下单(Trait方法)**
```rust
async fn place_order(&self, order: Order) -> Result<String, anyhow::Error>
```

**参数**: `Order` 结构
```rust
pub struct Order {
    pub symbol: String,         // 合约代码
    pub side: OrderSide,        // Buy/Sell
    pub quantity: f64,          // 数量
    pub price: Option<f64>,     // 价格 (None = 市价单)
    pub order_type: OrderType,  // Limit/Market
}
```

**示例**:
```rust
use nof0_backend::markets::{Order, OrderSide, OrderType, MarketAdapter};

// 限价单买入
let order = Order {
    symbol: "IF2501".to_string(),
    side: OrderSide::Buy,
    quantity: 1.0,
    price: Some(3500.0),
    order_type: OrderType::Limit,
};

let order_id = adapter.place_order(order).await?;
println!("Order placed: {}", order_id);  // "MOCK00000001"

// 市价单卖出
let market_order = Order {
    symbol: "IC2501".to_string(),
    side: OrderSide::Sell,
    quantity: 2.0,
    price: None,
    order_type: OrderType::Market,
};

let order_id = adapter.place_order(market_order).await?;
```

**Mock模式行为**:
- 订单ID格式: `MOCK00000001`, `MOCK00000002`, ...
- 立即返回"全部成交"状态
- 不更新持仓(需要手动实现)

**Real模式行为**:
- 返回CTP系统订单号
- 异步成交,需要监听回报
- 自动更新持仓

### 5.6 账户查询

#### **query_account() - 查询账户**
```rust
pub async fn query_account(&self) -> Result<CtpAccount>
```

**示例**:
```rust
let account = adapter.query_account().await?;

println!("Balance: ¥{:.2}", account.balance);
println!("Available: ¥{:.2}", account.available);
println!("Margin: ¥{:.2}", account.margin);
println!("Equity: ¥{:.2}", account.equity());
println!("Risk: {:.2}%", account.risk_ratio() * 100.0);
```

#### **get_balance() - 获取余额(Trait方法)**
```rust
async fn get_balance(&self, account_id: &str) -> Result<Vec<Balance>, anyhow::Error>
```

**返回**: `Balance` 列表(trait标准类型)

**示例**:
```rust
use nof0_backend::markets::MarketAdapter;

let balances = adapter.get_balance("default").await?;
for balance in balances {
    println!("{}: {:.2}", balance.asset, balance.free);
}
```

### 5.7 持仓查询

#### **query_position() - 查询持仓**
```rust
pub async fn query_position(&self) -> Result<Vec<CtpPosition>>
```

**示例**:
```rust
let positions = adapter.query_position().await?;

for pos in positions {
    let direction = if pos.direction == '2' { "Long" } else { "Short" };
    println!("{} {} x{}: P&L=¥{:.2}",
        pos.instrument_id,
        direction,
        pos.position,
        pos.position_profit
    );
}
```

---

## 配置说明

### 6.1 配置文件示例

**文件路径**: `backend/etc/ctp_config.yaml`

```yaml
# CTP配置文件 - 期货交易平台连接配置
# ============================================

# 期货公司代码 (Broker ID)
# SimNow测试环境使用 "9999"
# 生产环境请联系你的期货公司获取
broker_id: "9999"

# 投资者账号 (Investor ID)
# SimNow可以使用任意账号进行测试
investor_id: "000000"

# 登录密码
# ⚠️ 生产环境请使用环境变量: ${CTP_PASSWORD}
password: "password"

# 行情前置地址 (Market Data Address)
# SimNow 7x24环境:
#   - 电信: tcp://180.168.146.187:10131
#   - 移动: tcp://218.202.237.33:10131
md_address: "tcp://180.168.146.187:10131"

# 交易前置地址 (Trade Address)
# SimNow 7x24环境:
#   - 电信: tcp://180.168.146.187:10130
#   - 移动: tcp://218.202.237.33:10130
td_address: "tcp://180.168.146.187:10130"

# Mock模式 (启用模拟交易)
# true:  使用模拟数据,无需真实CTP连接
# false: 连接到真实CTP服务器
mock_mode: true

# ============================================
# 生产环境注意事项:
# 1. 不要将密码硬编码在配置文件中
# 2. 使用环境变量: password: "${CTP_PASSWORD}"
# 3. 确保 mock_mode: false
# 4. 使用期货公司提供的正式前置地址
# ============================================
```

### 6.2 环境变量

**安全最佳实践**: 使用环境变量存储敏感信息

#### **Windows (PowerShell)**
```powershell
# 设置环境变量
$env:CTP_BROKER_ID = "9999"
$env:CTP_INVESTOR_ID = "your_account"
$env:CTP_PASSWORD = "your_password"

# 验证
echo $env:CTP_PASSWORD
```

#### **Linux/Mac (Bash)**
```bash
# 设置环境变量
export CTP_BROKER_ID="9999"
export CTP_INVESTOR_ID="your_account"
export CTP_PASSWORD="your_password"

# 验证
echo $CTP_PASSWORD
```

#### **在代码中使用**
```rust
use std::env;

let config = CtpConfig {
    broker_id: env::var("CTP_BROKER_ID")
        .unwrap_or_else(|_| "9999".to_string()),
    investor_id: env::var("CTP_INVESTOR_ID")
        .expect("CTP_INVESTOR_ID not set"),
    password: env::var("CTP_PASSWORD")
        .expect("CTP_PASSWORD not set"),
    // ... 其他字段
};
```

### 6.3 SimNow测试环境

**SimNow** 是CTP官方提供的7x24小时模拟交易环境。

| 参数 | 电信 | 移动 |
|------|------|------|
| **行情地址** | tcp://180.168.146.187:10131 | tcp://218.202.237.33:10131 |
| **交易地址** | tcp://180.168.146.187:10130 | tcp://218.202.237.33:10130 |
| **Broker ID** | 9999 | 9999 |
| **账号** | 需注册获取 | 需注册获取 |

**注册地址**: http://www.simnow.com.cn/

**特点**:
- ✅ 免费使用
- ✅ 7x24小时运行
- ✅ 模拟真实行情数据
- ✅ 模拟真实交易流程
- ❌ 数据有延迟(非实时)

---

## 使用示例

### 7.1 完整示例程序

**文件**: `backend/examples/ctp_market_demo.rs`

```rust
use nof0_backend::markets::{
    CtpConfig, CtpMarketAdapter, MarketAdapter,
    Order, OrderSide, OrderType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CTP Market Adapter Demo\n");

    // ============================================
    // Scenario 1: 连接到CTP
    // ============================================
    println!("📋 Scenario 1: Connect to CTP");
    println!("{}", "=".repeat(60));
    
    let config = CtpConfig::from_file("etc/ctp_config.yaml")?;
    let adapter = CtpMarketAdapter::new(config.clone());
    
    adapter.connect().await?;
    println!("✅ Connected to: {}\n", adapter.market_name());

    // ============================================
    // Scenario 2: 订阅行情
    // ============================================
    println!("📋 Scenario 2: Subscribe Market Data");
    println!("{}", "=".repeat(60));
    
    let instruments = vec![
        "IF2501".to_string(),
        "IC2501".to_string(),
        "IH2501".to_string(),
    ];
    
    adapter.subscribe_market_data(instruments).await?;
    println!("✅ Subscribed to market data\n");

    // ============================================
    // Scenario 3: 查询账户
    // ============================================
    println!("📋 Scenario 3: Query Account");
    println!("{}", "=".repeat(60));
    
    let account = adapter.query_account().await?;
    println!("📊 Account: {}", account.account_id);
    println!("   Balance: ¥{:.2}", account.balance);
    println!("   Available: ¥{:.2}", account.available);
    println!("   Margin: ¥{:.2}", account.margin);
    println!("   Equity: ¥{:.2}", account.equity());
    println!("   Risk: {:.2}%\n", account.risk_ratio() * 100.0);

    // ============================================
    // Scenario 4: 查询持仓
    // ============================================
    println!("📋 Scenario 4: Query Positions");
    println!("{}", "=".repeat(60));
    
    let positions = adapter.query_position().await?;
    if positions.is_empty() {
        println!("📭 No positions\n");
    } else {
        for pos in positions {
            let dir = if pos.direction == '2' { "Long" } else { "Short" };
            println!("📊 {} {} x{}", pos.instrument_id, dir, pos.position);
            println!("   P&L: ¥{:.2}\n", pos.position_profit);
        }
    }

    // ============================================
    // Scenario 5: 获取行情
    // ============================================
    println!("📋 Scenario 5: Get Market Prices");
    println!("{}", "=".repeat(60));
    
    for symbol in ["IF2501", "IC2501", "IH2501"] {
        let price = adapter.get_price(symbol).await?;
        println!("📊 {}", symbol);
        println!("   Price: ¥{:.2}", price.price);
        println!("   Time: {}\n", price.timestamp.format("%H:%M:%S"));
    }

    // ============================================
    // Scenario 6: 下单
    // ============================================
    println!("📋 Scenario 6: Place Orders");
    println!("{}", "=".repeat(60));
    
    // 限价买单
    let order = Order {
        symbol: "IF2501".to_string(),
        side: OrderSide::Buy,
        quantity: 1.0,
        price: Some(3500.0),
        order_type: OrderType::Limit,
    };
    
    let order_id = adapter.place_order(order).await?;
    println!("✅ Order placed: {}\n", order_id);

    // 断开连接
    adapter.disconnect().await?;
    println!("✅ Disconnected");

    Ok(())
}
```

**运行**:
```bash
cd backend
cargo run --example ctp_market_demo
```

### 7.2 与Risk Management集成

```rust
use nof0_backend::risk::{RiskManager, RiskConfig};
use nof0_backend::markets::{CtpMarketAdapter, MarketAdapter, Order};

// 1. 创建Risk Manager
let risk_config = RiskConfig::from_file("etc/risk_config.yaml")?;
let risk_manager = RiskManager::new(risk_config);

// 2. 创建CTP Adapter
let ctp_config = CtpConfig::from_file("etc/ctp_config.yaml")?;
let market_adapter = CtpMarketAdapter::new(ctp_config);
market_adapter.connect().await?;

// 3. 下单前进行风控检查
let order = Order {
    symbol: "IF2501".to_string(),
    side: OrderSide::Buy,
    quantity: 10.0,  // 大单
    price: Some(3500.0),
    order_type: OrderType::Limit,
};

// 执行5个风控规则
match risk_manager.check_order(&order, 1000000.0).await {
    Ok(_) => {
        // 风控通过,提交订单
        let order_id = market_adapter.place_order(order).await?;
        println!("✅ Order passed risk check: {}", order_id);
    }
    Err(e) => {
        // 风控拒绝
        eprintln!("❌ Risk check failed: {}", e);
    }
}
```

---

## Mock模式

### 8.1 Mock模式概述

Mock模式是完全独立的模拟环境,**不需要**真实的CTP连接。

**优势**:
- ✅ 无需注册SimNow账号
- ✅ 无需网络连接
- ✅ 测试速度快
- ✅ 可预测的行为
- ✅ 适合CI/CD

**限制**:
- ❌ 不反映真实市场波动
- ❌ 不测试网络问题
- ❌ 订单立即成交(非真实)

### 8.2 Mock数据

#### **初始化数据**
```rust
async fn init_mock_data(&self) {
    let mut data = self.market_data.write().await;
    
    // IF2501 - 沪深300股指期货
    data.insert("IF2501".to_string(), CtpMarketData {
        instrument_id: "IF2501".to_string(),
        last_price: 3500.0,
        bid_price: 3499.8,
        ask_price: 3500.2,
        volume: 125000,
        open_interest: 85000,
        open_price: 3480.0,
        highest_price: 3520.0,
        lowest_price: 3480.0,
        update_time: now.format("%H:%M:%S").to_string(),
        update_millisec: now.timestamp_subsec_millis() as i32,
    });
    
    // IC2501, IH2501 ...
}
```

#### **Mock账户**
```rust
CtpAccount {
    account_id: config.investor_id.clone(),
    pre_balance: 1000000.0,      // 100万初始资金
    deposit: 0.0,
    withdraw: 0.0,
    balance: 1000000.0,
    available: 1000000.0,        // 全部可用
    margin: 0.0,                 // 无持仓
    frozen_margin: 0.0,
    position_profit: 0.0,
    close_profit: 0.0,
    commission: 0.0,
}
```

### 8.3 Mock订单处理

```rust
async fn place_order_mock(&self, request: CtpOrderRequest) -> Result<CtpOrderResponse> {
    // 1. 生成订单号
    let mut counter = self.order_counter.write().await;
    *counter += 1;
    let order_ref = format!("{:08}", *counter);
    let order_sys_id = format!("MOCK{}", order_ref);
    
    // 2. 模拟成交(立即全部成交)
    let response = CtpOrderResponse {
        order_sys_id: order_sys_id.clone(),
        order_ref: order_ref.clone(),
        order_status: CtpOrderStatus::AllTraded,
        status_msg: "All traded".to_string(),
    };
    
    // 3. 打印日志
    println!("🎭 CTP Mock: Order executed");
    println!("   Order ID: {}", order_sys_id);
    println!("   Instrument: {}", request.instrument_id);
    println!("   Price: {:.2}, Volume: {}", request.price, request.volume);
    
    Ok(response)
}
```

**特点**:
- 订单ID: `MOCK00000001`, `MOCK00000002`, ...
- 状态: 始终是 `AllTraded` (全部成交)
- 无滑点: 按委托价格成交
- 无拒单: 总是成功

---

## 真实模式

### 9.1 Real模式概述

真实模式连接到实际的CTP服务器(SimNow或期货公司)。

**需要的库**: `ctp2rs` (CTP Rust绑定)

### 9.2 依赖配置

**Cargo.toml**:
```toml
[dependencies]
# ... 现有依赖

# CTP接口 (未来添加)
ctp2rs = "0.1"  # 假设版本
```

### 9.3 Real模式实现(伪代码)

```rust
use ctp2rs::{MdApi, TdApi};

pub struct RealCtpConnection {
    md_api: MdApi,  // 行情API
    td_api: TdApi,  // 交易API
}

impl RealCtpConnection {
    pub fn new(config: &CtpConfig) -> Self {
        let md_api = MdApi::new();
        let td_api = TdApi::new();
        
        Self { md_api, td_api }
    }
    
    pub async fn connect(&mut self, config: &CtpConfig) -> Result<()> {
        // 1. 连接行情前置
        self.md_api.register_front(&config.md_address);
        self.md_api.init();
        
        // 2. 连接交易前置
        self.td_api.register_front(&config.td_address);
        self.td_api.init();
        
        // 3. 等待连接回调
        // ...
        
        // 4. 登录
        self.login(config).await?;
        
        Ok(())
    }
    
    async fn login(&mut self, config: &CtpConfig) -> Result<()> {
        let req = ReqUserLogin {
            broker_id: config.broker_id.clone(),
            user_id: config.investor_id.clone(),
            password: config.password.clone(),
        };
        
        self.td_api.req_user_login(&req)?;
        
        // 等待登录回调
        // ...
        
        Ok(())
    }
    
    pub async fn subscribe(&mut self, instruments: Vec<String>) -> Result<()> {
        let symbols: Vec<&str> = instruments.iter()
            .map(|s| s.as_str())
            .collect();
            
        self.md_api.subscribe_market_data(&symbols)?;
        Ok(())
    }
}
```

### 9.4 切换到Real模式

**配置文件**:
```yaml
# etc/ctp_config.yaml
mock_mode: false  # ← 改为 false

# 使用真实地址
broker_id: "YOUR_BROKER_ID"
investor_id: "YOUR_ACCOUNT"
password: "${CTP_PASSWORD}"
md_address: "tcp://YOUR_MD_ADDRESS:PORT"
td_address: "tcp://YOUR_TD_ADDRESS:PORT"
```

**代码无需修改**:
```rust
// 自动选择模式
let config = CtpConfig::from_file("etc/ctp_config.yaml")?;
let adapter = CtpMarketAdapter::new(config);

// mock_mode=false 时会使用真实连接
adapter.connect().await?;
```

---

## 错误处理

### 10.1 错误类型

CTP Adapter使用 `anyhow::Error` 统一错误处理:

```rust
pub type Result<T> = std::result::Result<T, anyhow::Error>;
```

**常见错误**:

| 错误 | 原因 | 解决方案 |
|------|------|---------|
| **Config file not found** | 配置文件路径错误 | 检查 `etc/ctp_config.yaml` |
| **Not connected** | 未调用 `connect()` | 先连接再操作 |
| **Instrument not found** | 未订阅该合约 | 调用 `subscribe_market_data()` |
| **Unsupported order side** | 无效的买卖方向 | 使用 `OrderSide::Buy/Sell` |
| **CTP login failed** | 账号密码错误 | 检查配置,SimNow需注册 |

### 10.2 错误处理示例

```rust
use anyhow::{Context, Result};

async fn trade_example() -> Result<()> {
    // 1. 加载配置(带上下文)
    let config = CtpConfig::from_file("etc/ctp_config.yaml")
        .context("Failed to load CTP config")?;
    
    // 2. 连接(带重试)
    let adapter = CtpMarketAdapter::new(config);
    for attempt in 1..=3 {
        match adapter.connect().await {
            Ok(_) => break,
            Err(e) if attempt < 3 => {
                eprintln!("Connection attempt {} failed: {}", attempt, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e).context("Failed to connect after 3 attempts"),
        }
    }
    
    // 3. 下单(带错误处理)
    let order = Order {
        symbol: "IF2501".to_string(),
        side: OrderSide::Buy,
        quantity: 1.0,
        price: Some(3500.0),
        order_type: OrderType::Limit,
    };
    
    match adapter.place_order(order).await {
        Ok(order_id) => {
            println!("✅ Order placed: {}", order_id);
        }
        Err(e) => {
            eprintln!("❌ Order failed: {}", e);
            // 记录日志、发送告警等
        }
    }
    
    Ok(())
}
```

### 10.3 日志记录

**建议添加tracing**:
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn connect(&self) -> Result<()> {
    info!("Connecting to CTP server...");
    
    if self.config.mock_mode {
        info!("Mock mode enabled");
        // ...
    } else {
        warn!("Real mode: connecting to {}", self.config.td_address);
        // ...
    }
    
    Ok(())
}
```

---

## 性能优化

### 11.1 缓存策略

CTP Adapter使用多层缓存减少网络请求:

```rust
pub struct CtpMarketAdapter {
    // 行情数据缓存 (实时更新)
    market_data: Arc<RwLock<HashMap<String, CtpMarketData>>>,
    
    // 持仓缓存 (订单成交后更新)
    positions: Arc<RwLock<HashMap<String, CtpPosition>>>,
    
    // 账户缓存 (定期刷新)
    account: Arc<RwLock<Option<CtpAccount>>>,
}
```

**缓存更新策略**:
- **Market Data**: 行情推送时立即更新
- **Positions**: 订单成交回报时更新
- **Account**: 每5秒查询一次(可配置)

### 11.2 并发性能

使用 `Arc<RwLock<T>>` 实现高并发读取:

```rust
// 多个线程可以同时读取
let data1 = adapter.get_market_data("IF2501").await?;
let data2 = adapter.get_market_data("IC2501").await?;
let data3 = adapter.get_market_data("IH2501").await?;

// 写入时独占锁
adapter.subscribe_market_data(new_instruments).await?;
```

**性能特点**:
- 读取: O(1) HashMap查找
- 写入: 独占锁,阻塞其他写入
- 无锁读: 多个读取器可并发

### 11.3 连接池

**未来优化**: 支持多个CTP连接

```rust
pub struct CtpConnectionPool {
    connections: Vec<Arc<RealCtpConnection>>,
    current_index: AtomicUsize,
}

impl CtpConnectionPool {
    pub fn get_connection(&self) -> Arc<RealCtpConnection> {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed);
        let conn_index = index % self.connections.len();
        self.connections[conn_index].clone()
    }
}
```

---

## 测试

### 11.1 单元测试

**文件**: `backend/src/markets/ctp/adapter_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_connect() {
        let config = CtpConfig {
            broker_id: "9999".to_string(),
            investor_id: "test".to_string(),
            password: "test".to_string(),
            md_address: "".to_string(),
            td_address: "".to_string(),
            mock_mode: true,
        };
        
        let adapter = CtpMarketAdapter::new(config);
        assert!(adapter.connect().await.is_ok());
        assert!(adapter.is_connected().await);
    }
    
    #[tokio::test]
    async fn test_mock_subscribe() {
        let config = create_mock_config();
        let adapter = CtpMarketAdapter::new(config);
        
        adapter.connect().await.unwrap();
        
        let instruments = vec!["IF2501".to_string()];
        assert!(adapter.subscribe_market_data(instruments).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_mock_order() {
        let config = create_mock_config();
        let adapter = CtpMarketAdapter::new(config);
        
        adapter.connect().await.unwrap();
        
        let order = Order {
            symbol: "IF2501".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(3500.0),
            order_type: OrderType::Limit,
        };
        
        let result = adapter.place_order(order).await;
        assert!(result.is_ok());
        
        let order_id = result.unwrap();
        assert!(order_id.starts_with("MOCK"));
    }
}
```

**运行测试**:
```bash
cd backend
cargo test markets::ctp
```

### 11.2 集成测试

**文件**: `backend/tests/ctp_integration_test.rs`

```rust
use nof0_backend::markets::{CtpConfig, CtpMarketAdapter, MarketAdapter};

#[tokio::test]
async fn test_full_workflow() {
    // 1. 加载配置
    let config = CtpConfig::from_file("etc/ctp_config.yaml")
        .expect("Config file should exist");
    
    // 2. 连接
    let adapter = CtpMarketAdapter::new(config);
    adapter.connect().await.expect("Should connect");
    
    // 3. 订阅
    let instruments = vec!["IF2501".to_string()];
    adapter.subscribe_market_data(instruments).await
        .expect("Should subscribe");
    
    // 4. 获取行情
    let price = adapter.get_price("IF2501").await
        .expect("Should get price");
    assert!(price.price > 0.0);
    
    // 5. 查询账户
    let account = adapter.query_account().await
        .expect("Should query account");
    assert_eq!(account.account_id, "000000");
    
    // 6. 下单
    let order = Order {
        symbol: "IF2501".to_string(),
        side: OrderSide::Buy,
        quantity: 1.0,
        price: Some(3500.0),
        order_type: OrderType::Limit,
    };
    
    let order_id = adapter.place_order(order).await
        .expect("Should place order");
    assert!(order_id.starts_with("MOCK"));
    
    // 7. 断开
    adapter.disconnect().await.expect("Should disconnect");
}
```

**运行集成测试**:
```bash
cd backend
cargo test --test ctp_integration_test
```

---

## 已知限制

### 12.1 Mock模式限制

| 限制 | 影响 | 缓解措施 |
|------|------|---------|
| **静态行情** | 价格不波动 | 使用SimNow或Real模式 |
| **无滑点** | 按委托价成交 | 添加随机滑点模拟 |
| **立即成交** | 不测试挂单逻辑 | Real模式测试 |
| **无拒单** | 不测试错误处理 | 手动测试错误场景 |

### 12.2 Real模式限制

| 限制 | 影响 | 缓解措施 |
|------|------|---------|
| **未实现** | 无法连接真实CTP | 等待ctp2rs集成 |
| **回调复杂** | 异步事件处理 | 使用tokio channel |
| **连接断开** | 需要重连机制 | 实现心跳+重连 |
| **流控** | 请求频率限制 | 实现请求队列 |

### 12.3 功能限制

**未实现的功能**:
- ❌ 撤单 (CancelOrder)
- ❌ 改单 (ModifyOrder)
- ❌ 成交查询 (QueryTrade)
- ❌ 历史数据 (Historical Data)
- ❌ 条件单 (Conditional Order)
- ❌ 止损止盈 (Stop Loss/Take Profit)

---

## 路线图

### 13.1 短期目标 (1-2周)

- [ ] **Real模式实现**
  - [ ] 集成ctp2rs库
  - [ ] 实现CTP连接和登录
  - [ ] 处理行情/交易回调
  
- [ ] **撤单功能**
  - [ ] 实现 `cancel_order(order_id)`
  - [ ] 添加撤单Mock逻辑
  
- [ ] **改单功能**
  - [ ] 实现 `modify_order(order_id, new_price, new_volume)`
  
- [ ] **成交查询**
  - [ ] 实现 `query_trades(start_date, end_date)`

### 13.2 中期目标 (1-2月)

- [ ] **更多合约类型**
  - [ ] 商品期货 (rb, au, cu等)
  - [ ] 期权合约
  
- [ ] **高级订单类型**
  - [ ] 条件单 (触发价)
  - [ ] 冰山单 (分批下单)
  - [ ] 止损止盈单
  
- [ ] **历史数据**
  - [ ] 分钟K线查询
  - [ ] 日K线查询
  
- [ ] **性能优化**
  - [ ] 连接池
  - [ ] 行情订阅优化
  - [ ] 缓存策略调优

### 13.3 长期目标 (3-6月)

- [ ] **多账户支持**
  - [ ] 同时管理多个CTP账户
  - [ ] 跨账户资金调度
  
- [ ] **高可用**
  - [ ] 自动重连
  - [ ] 故障转移
  - [ ] 健康检查
  
- [ ] **监控告警**
  - [ ] Prometheus metrics
  - [ ] 连接状态告警
  - [ ] 订单异常告警
  
- [ ] **回测模式**
  - [ ] 历史数据回放
  - [ ] 策略回测支持

---

## 附录

### A.1 CTP术语表

| 术语 | 英文 | 说明 |
|------|------|------|
| **合约** | Instrument | 期货交易标的,如IF2501 |
| **做多** | Long/Buy | 买入开仓,预期价格上涨 |
| **做空** | Short/Sell | 卖出开仓,预期价格下跌 |
| **开仓** | Open | 建立新的持仓 |
| **平仓** | Close | 关闭现有持仓 |
| **平今** | Close Today | 平掉今天开的仓位 |
| **平昨** | Close Yesterday | 平掉昨天及之前的仓位 |
| **保证金** | Margin | 持仓所需的资金占用 |
| **持仓量** | Open Interest | 市场总持仓手数 |
| **成交量** | Volume | 今日累计成交手数 |
| **滑点** | Slippage | 委托价与成交价的差异 |

### A.2 合约命名规则

**格式**: `品种代码 + 年份(2位) + 月份(2位)`

**示例**:
- **IF2501**: 沪深300股指期货,2025年1月到期
- **IC2503**: 中证500股指期货,2025年3月到期
- **IH2506**: 上证50股指期货,2025年6月到期

**交割月份**:
- 股指期货: 1, 3, 5, 7, 9, 11月(双月)
- 商品期货: 各有不同,如螺纹钢全年12个月

### A.3 保证金计算

**公式**:
```
保证金 = 持仓量 × 合约乘数 × 最新价 × 保证金率
```

**示例**:
```
IF2501:
  持仓量: 1手
  合约乘数: 300 (每点300元)
  最新价: 3500.0
  保证金率: 15% (交易所+期货公司)
  
保证金 = 1 × 300 × 3500 × 0.15 = 157,500元
```

### A.4 参考资源

**官方文档**:
- CTP API文档: http://www.sfit.com.cn/
- SimNow注册: http://www.simnow.com.cn/

**Rust资源**:
- async-trait: https://docs.rs/async-trait/
- tokio: https://tokio.rs/
- anyhow: https://docs.rs/anyhow/

**项目链接**:
- nof0 GitHub: https://github.com/yourusername/nof0
- Risk Management文档: [RISK_MANAGEMENT.md](./RISK_MANAGEMENT.md)

---

**📝 文档版本**: 1.0.0  
**✍️ 最后更新**: 2025-01-18  
**👤 维护者**: nof0 Development Team

**🎯 下一步**: 阅读 [CTP_ADAPTER_QUICKSTART.md](./CTP_ADAPTER_QUICKSTART.md) 快速开始使用!
