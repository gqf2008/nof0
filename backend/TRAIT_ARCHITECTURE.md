# Broker 特性架构重构完成

## 概述
已成功将原有的单一 Broker trait 重构为基于特性组合的模块化架构。

## 架构设计

### 核心特性 (Traits)

#### 1. MarketData 特性 (行情接口)
负责市场数据相关功能:
- `get_prices()` - 获取实时价格
- `get_orderbook(symbol)` - 获取市场深度
- `get_klines(symbol, interval, limit)` - 获取K线数据
- `get_ticker_24h(symbol)` - 获取24小时ticker

#### 2. Trading 特性 (交易接口)
负责交易执行相关功能:
- `place_order(order)` - 下单
- `cancel_order(order_id)` - 撤单
- `get_order(order_id)` - 查询订单
- `get_orders(symbol)` - 查询所有订单
- `get_trades()` - 查询成交记录

#### 3. AccountManagement 特性 (账户管理接口)
负责账户和持仓管理:
- `get_account_totals(last_marker)` - 获取账户历史数据（图表用）
- `get_model_accounts()` - 获取模型账户摘要（卡片展示用）
- `get_positions(limit)` - 获取持仓数据
- `get_balance()` - 获取账户余额
- `get_broker_account()` - 获取经纪商账户信息

#### 4. Analytics 特性 (分析与统计接口)
负责策略分析和统计:
- `get_analytics()` - 获取分析数据
- `get_leaderboard()` - 获取排行榜
- `get_since_inception_values()` - 获取初始值数据
- `get_conversations()` - 获取 AI 对话记录
- `get_models_list()` - 获取模型列表

#### 5. Broker 特性 (组合特性)
聚合所有功能接口:
```rust
pub trait Broker: MarketData + Trading + AccountManagement + Analytics + Send + Sync {
    fn broker_id(&self) -> &str;
    fn broker_name(&self) -> &str;
}
```

## 文件结构

```
backend/src/
├── brokers/
│   ├── mod.rs                # Broker trait 定义 + BrokerRegistry
│   ├── mock_broker.rs        # MockBroker 实现
│   └── ctp/
│       ├── mod.rs            # CTP 模块
│       ├── broker.rs         # CtpBroker trait 实现
│       ├── adapter.rs        # CTP 市场适配器（旧架构）
│       ├── types.rs          # CTP 类型定义
│       └── ...               # 其他 CTP 相关文件
├── config/
│   └── brokers.rs            # BrokerConfig 和 BrokersConfig
└── main.rs                   # 模块注册
```

## MockBroker 实现

### 结构定义
```rust
pub struct MockBroker {
    id: String,
    name: String,
    broker_type: BrokerType,
}

pub enum BrokerType {
    Crypto,    // 加密货币交易所
    Futures,   // 期货交易所
    Stock,     // 股票交易所
}
```

### 特性实现分离
所有实现都按照特性类别分组:
- `impl MarketData for MockBroker { ... }`
- `impl Trading for MockBroker { ... }`
- `impl AccountManagement for MockBroker { ... }`
- `impl Analytics for MockBroker { ... }`
- `impl Broker for MockBroker { ... }`

### 数据生成特点

#### 价格数据 (MarketData)
- 基于预定义的基准价格
- 支持多种时间间隔的K线数据
- 模拟真实的市场深度

#### 账户数据 (AccountManagement)
- 使用随机游走算法生成真实的净值曲线
- 根据风险等级差异化数据:
  - VERY_LOW: 低波动 (0.05%), 高胜率 (55-70%)
  - LOW: 较低波动 (0.1%), 较高胜率 (52-65%)
  - MEDIUM: 中等波动 (0.2%), 中等胜率 (48-62%)
  - HIGH: 较高波动 (0.3%), 较低胜率 (45-60%)
  - VERY_HIGH: 高波动 (0.5%), 低胜率 (42-58%)

#### 持仓数据 (AccountManagement)
- 根据交易所类型设置不同特征:
  - Crypto: 2-4个持仓, 5-20倍杠杆, 激进风格
  - CTP: 3-5个持仓, 1-10倍杠杆, 保守风格
  - Binance: 3-6个持仓, 10-25倍杠杆, 非常激进
  - Bybit: 2-5个持仓, 5-15倍杠杆, 中等风格
  - Kraken: 1-3个持仓, 1-5倍杠杆, 非常保守

## BrokerRegistry

注册表模式用于管理多个 broker 实例:

```rust
pub struct BrokerRegistry {
    brokers: HashMap<String, Box<dyn Broker>>,
}

impl BrokerRegistry {
    pub fn new() -> Self
    pub fn register(&mut self, broker: Box<dyn Broker>)
    pub fn get(&self, broker_id: &str) -> Option<&Box<dyn Broker>>
    pub fn list_ids(&self) -> Vec<String>
}
```

## 使用示例

### 创建 MockBroker
```rust
use brokers::{MockBroker, BrokerType};

let broker = MockBroker::new(
    "crypto".to_string(),
    "Crypto Broker".to_string(),
    BrokerType::Crypto
);
```

### 注册到 BrokerRegistry
```rust
use broker_api::BrokerRegistry;

let mut registry = BrokerRegistry::new();
registry.register(Box::new(broker));
```

### 调用接口
```rust
// 通过特性调用
let prices = broker.get_prices().await?;
let positions = broker.get_positions(Some(10)).await?;

// 通过注册表调用
if let Some(broker) = registry.get("crypto") {
    let accounts = broker.get_model_accounts().await?;
}
```

## 优势

### 1. 关注点分离
每个特性只负责一类功能,代码职责清晰。

### 2. 模块化
- 新的 broker 实现可以选择性实现某些特性
- 易于测试和 mock 单个特性

### 3. 可扩展性
- 添加新特性不影响现有实现
- 新 broker 类型只需实现对应特性

### 4. 类型安全
- 编译时检查特性实现完整性
- Rust 的 trait 系统提供强类型保证

### 5. 组合优于继承
- 通过 trait bounds 实现特性组合
- 避免了继承层次带来的复杂性

## 下一步工作

1. **集成到 server.rs**
   - 在 server 启动时创建 BrokerRegistry
   - 初始化各个 broker 实例
   - 将路由请求分发到对应的 broker

2. **实现真实 broker**
   - CtpBroker (CTP 期货)
   - BinanceBroker (币安)
   - BybitBroker (Bybit)
   等等...

3. **完善 Mock 实现**
   - 添加更多真实的模拟数据
   - 实现 Analytics 特性的详细逻辑
   - 优化数据生成算法

4. **测试覆盖**
   - 为每个特性编写单元测试
   - 集成测试验证多 broker 场景

5. **文档完善**
   - API 文档
   - 使用指南
   - 架构决策记录
