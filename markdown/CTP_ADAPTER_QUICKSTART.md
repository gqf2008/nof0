# CTP Market Adapter - 快速入门指南

**⏱️ 预计时间**: 10分钟  
**🎯 目标**: 快速上手CTP期货市场适配器

---

## 🚀 快速开始

### Step 1: 检查配置文件

配置文件位于 `backend/etc/ctp_config.yaml`:

```yaml
broker_id: "9999"              # SimNow测试环境
investor_id: "000000"          # 测试账号
password: "password"           # 测试密码
md_address: "tcp://180.168.146.187:10131"
td_address: "tcp://180.168.146.187:10130"
mock_mode: true                # ← 启用Mock模式(推荐初次使用)
```

**✅ Mock模式**: 无需真实CTP连接,适合测试  
**⚠️ Real模式**: 需要SimNow账号(未实现)

### Step 2: 运行演示程序

```bash
cd backend
cargo run --example ctp_market_demo
```

**预期输出**:

```text
🚀 CTP Market Adapter Demo

============================================================

📋 Scenario 1: Connect to CTP
------------------------------------------------------------
✅ Connected successfully!
   Market: CTP (Mock)

📋 Scenario 2: Subscribe Market Data
------------------------------------------------------------
✅ Subscription successful

📋 Scenario 3: Query Account Info
------------------------------------------------------------
📊 Account Information:
   Balance: ¥1000000.00
   Available: ¥1000000.00
   Margin: ¥0.00
   Equity: ¥1000000.00
   Risk Ratio: 0.00%

📋 Scenario 4: Query Positions
------------------------------------------------------------
📭 No positions currently held

📋 Scenario 5: Get Market Prices
------------------------------------------------------------
📊 IF2501
   Last: ¥3500.00
   Time: 10:03:33

📋 Scenario 6: Place Orders
------------------------------------------------------------
📤 Order #1: Buy IF2501
   ✅ Order ID: MOCK00000001

✅ Disconnected
```

### Step 3: 在你的代码中使用

创建新文件 `my_trading_bot.rs`:

```rust
use nof0_backend::markets::{
    CtpConfig, CtpMarketAdapter, MarketAdapter,
    Order, OrderSide, OrderType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置并连接
    let config = CtpConfig::from_file("etc/ctp_config.yaml")?;
    let adapter = CtpMarketAdapter::new(config);
    adapter.connect().await?;
    
    // 2. 订阅行情
    adapter.subscribe_market_data(vec![
        "IF2501".to_string(),
        "IC2501".to_string(),
    ]).await?;
    
    // 3. 获取价格
    let price = adapter.get_price("IF2501").await?;
    println!("IF2501 Price: ¥{:.2}", price.price);
    
    // 4. 查询账户
    let account = adapter.query_account().await?;
    println!("Available: ¥{:.2}", account.available);
    
    // 5. 下单
    let order = Order {
        symbol: "IF2501".to_string(),
        side: OrderSide::Buy,
        quantity: 1.0,
        price: Some(3500.0),
        order_type: OrderType::Limit,
    };
    
    let order_id = adapter.place_order(order).await?;
    println!("Order placed: {}", order_id);
    
    // 6. 断开连接
    adapter.disconnect().await?;
    
    Ok(())
}
```

**运行**:

```bash
cargo run --bin my_trading_bot
```

---

## 📚 核心概念

### 1. 合约代码

中国期货合约命名格式: `品种 + 年份(后2位) + 月份(2位)`

| 代码 | 品种 | 到期日 |
|------|------|--------|
| **IF2501** | 沪深300股指 | 2025年1月 |
| **IC2503** | 中证500股指 | 2025年3月 |
| **IH2506** | 上证50股指 | 2025年6月 |

### 2. 买卖方向

```rust
use nof0_backend::markets::OrderSide;

// 做多(买入开仓)
let buy_order = Order {
    side: OrderSide::Buy,
    // ...
};

// 做空(卖出开仓)
let sell_order = Order {
    side: OrderSide::Sell,
    // ...
};
```

### 3. 订单类型

```rust
use nof0_backend::markets::OrderType;

// 限价单(指定价格)
let limit_order = Order {
    order_type: OrderType::Limit,
    price: Some(3500.0),  // 必须指定价格
    // ...
};

// 市价单(对手价成交)
let market_order = Order {
    order_type: OrderType::Market,
    price: None,          // 无需指定价格
    // ...
};
```

### 4. 账户字段说明

```rust
let account = adapter.query_account().await?;

// 核心字段
account.balance        // 账户余额
account.available      // 可用资金
account.margin         // 已用保证金
account.position_profit // 持仓盈亏

// 计算字段
account.equity()       // 动态权益 = 余额 + 持仓盈亏
account.risk_ratio()   // 风险度 = 保证金 / 动态权益
```

**风险警示**:

- 🟢 风险度 < 50%: 安全
- 🟡 50% ≤ 风险度 < 80%: 警告
- 🔴 风险度 ≥ 80%: 危险!接近强平

---

## 🔧 常见任务

### 获取实时行情

```rust
// 方法1: 使用 get_price (简化版)
let price = adapter.get_price("IF2501").await?;
println!("Last: ¥{:.2}", price.price);

// 方法2: 使用 get_market_data (详细版)
let data = adapter.get_market_data("IF2501").await?;
println!("Bid: ¥{:.2}, Ask: ¥{:.2}", data.bid_price, data.ask_price);
println!("Volume: {}, OI: {}", data.volume, data.open_interest);
```

### 批量获取价格

```rust
let symbols = vec!["IF2501", "IC2501", "IH2501"];

for symbol in symbols {
    let price = adapter.get_price(symbol).await?;
    println!("{}: ¥{:.2}", symbol, price.price);
}
```

### 条件下单

```rust
// 当价格突破3550时买入
let current_price = adapter.get_price("IF2501").await?.price;

if current_price > 3550.0 {
    let order = Order {
        symbol: "IF2501".to_string(),
        side: OrderSide::Buy,
        quantity: 1.0,
        price: None,  // 市价单,快速成交
        order_type: OrderType::Market,
    };
    
    let order_id = adapter.place_order(order).await?;
    println!("Breakout buy: {}", order_id);
}
```

### 监控持仓

```rust
let positions = adapter.query_position().await?;

for pos in positions {
    let direction = if pos.direction == '2' { "Long" } else { "Short" };
    let pnl_percent = (pos.position_profit / pos.open_cost) * 100.0;
    
    println!("{} {} x{}: {:.2}% ({:.2})",
        pos.instrument_id,
        direction,
        pos.position,
        pnl_percent,
        pos.position_profit
    );
    
    // 止损逻辑
    if pnl_percent < -5.0 {
        println!("⚠️ Stop loss triggered!");
        // 平仓逻辑...
    }
}
```

### 风控检查

```rust
use nof0_backend::risk::{RiskManager, RiskConfig};

// 加载风控配置
let risk_config = RiskConfig::from_file("etc/risk_config.yaml")?;
let risk_manager = RiskManager::new(risk_config);

// 下单前检查
let order = Order {
    symbol: "IF2501".to_string(),
    side: OrderSide::Buy,
    quantity: 10.0,  // 大单
    price: Some(3500.0),
    order_type: OrderType::Limit,
};

let account = adapter.query_account().await?;
let balance = account.available;

// 执行5个风控规则
match risk_manager.check_order(&order, balance).await {
    Ok(_) => {
        // 风控通过,下单
        let order_id = adapter.place_order(order).await?;
        println!("✅ Order passed: {}", order_id);
    }
    Err(e) => {
        // 风控拒绝
        eprintln!("❌ Risk check failed: {}", e);
    }
}
```

---

## ⚠️ 常见问题

### Q1: "Config file not found"

**问题**: 找不到配置文件

**解决**:

```bash
# 检查文件是否存在
ls backend/etc/ctp_config.yaml

# 如果不存在,从模板创建
cp backend/etc/ctp_config.example.yaml backend/etc/ctp_config.yaml
```

### Q2: "Not connected"

**问题**: 调用API前忘记连接

**解决**:

```rust
// ❌ 错误: 未连接就下单
let adapter = CtpMarketAdapter::new(config);
adapter.place_order(order).await?;  // 失败!

// ✅ 正确: 先连接再操作
let adapter = CtpMarketAdapter::new(config);
adapter.connect().await?;           // 先连接
adapter.place_order(order).await?;  // 成功
```

### Q3: "Instrument not found"

**问题**: 获取未订阅的合约行情

**解决**:

```rust
// ❌ 错误: 未订阅就查询
let price = adapter.get_price("IF2501").await?;  // 失败!

// ✅ 正确: 先订阅再查询
adapter.subscribe_market_data(vec!["IF2501".to_string()]).await?;
let price = adapter.get_price("IF2501").await?;  // 成功
```

### Q4: Mock模式 vs Real模式?

**Mock模式** (推荐新手):

- ✅ 无需注册SimNow
- ✅ 立即可用
- ✅ 数据可预测
- ❌ 不反映真实市场

**Real模式** (生产环境):

- ✅ 真实行情数据
- ✅ 真实交易流程
- ❌ 需要SimNow账号
- ❌ 未实现(开发中)

**切换方式**:

```yaml
# etc/ctp_config.yaml
mock_mode: true   # Mock模式
mock_mode: false  # Real模式(需要真实账号)
```

### Q5: 如何调试?

**启用日志**:

```rust
// 在 main() 开头添加
env_logger::init();

// 运行时设置日志级别
RUST_LOG=debug cargo run --example ctp_market_demo
```

**打印调试信息**:

```rust
let adapter = CtpMarketAdapter::new(config);
adapter.connect().await?;

// 检查连接状态
println!("Connected: {}", adapter.is_connected().await);
println!("Market: {}", adapter.market_name());

// 打印账户详情
let account = adapter.query_account().await?;
println!("Account: {:#?}", account);
```

---

## 🎯 下一步

### 学习更多

- 📖 **完整文档**: [CTP_ADAPTER.md](./CTP_ADAPTER.md)
- 🔒 **风控系统**: [RISK_MANAGEMENT.md](./RISK_MANAGEMENT.md)
- 🏗️ **架构设计**: [docs/data-architecture.md](../go/docs/data-architecture.md)

### 实践项目

1. **行情监控Bot**
   - 监控多个合约价格
   - 价格突破时发送通知

2. **网格交易Bot**
   - 在价格区间内高抛低吸
   - 结合风控系统防止亏损

3. **套利Bot**
   - 监控不同合约的价差
   - 跨期套利、跨品种套利

### 示例代码

查看 `backend/examples/` 目录:

```bash
# CTP适配器演示
cargo run --example ctp_market_demo

# 风控系统演示
cargo run --example risk_demo

# 集成示例(即将发布)
cargo run --example trading_bot
```

---

## 📞 获取帮助

**遇到问题?**

1. **查看文档**: [CTP_ADAPTER.md](./CTP_ADAPTER.md)
2. **查看示例**: `backend/examples/ctp_market_demo.rs`
3. **提交Issue**: [GitHub Issues](https://github.com/yourusername/nof0/issues)

**贡献代码?**

1. Fork项目
2. 创建feature分支
3. 提交Pull Request

---

**🎉 恭喜!** 你已经学会了CTP Market Adapter的基础用法。

**下一步**: 阅读 [CTP_ADAPTER.md](./CTP_ADAPTER.md) 了解高级功能! 🚀
