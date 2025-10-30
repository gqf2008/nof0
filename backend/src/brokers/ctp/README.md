# CTP (中国期货市场) 模块

本模块提供完整的 CTP (Comprehensive Transaction Platform) 接口实现,支持中国期货市场的实时行情订阅和交易功能。

## ✨ 特性

- ✅ **完整的 CTP API 封装** - 支持行情和交易全部功能
- ✅ **异步架构** - 基于 tokio 的高性能异步实现
- ✅ **自动重连** - 断线自动重连,支持指数退避策略
- ✅ **流控保护** - 自动处理 CTP 查询频率限制
- ✅ **状态恢复** - 重连后自动恢复订阅状态
- ✅ **错误映射** - 70+ CTP 错误码中文映射
- ✅ **类型安全** - Rust 强类型系统保证内存安全
- ✅ **详细日志** - 使用 tracing 提供完整的操作日志

## 📁 模块结构

```
ctp/
├── adapter.rs           # CTP 适配器 (统一接口)
├── error_codes.rs       # CTP 错误码映射
├── md_spi.rs           # 行情 SPI 回调实现
├── trader_spi.rs       # 交易 SPI 回调实现
├── real_connection.rs  # 真实连接实现
├── types.rs            # 数据类型定义
└── mod.rs              # 模块导出
```

## 🚀 快速开始

### 1. 启用 CTP Real Mode

在 `Cargo.toml` 中:

```toml
[features]
default = []
ctp-real = ["ctp2rs"]

[dependencies]
ctp2rs = { version = "0.1.8", optional = true }
```

### 2. 编译

```bash
cargo build --features ctp-real
```

### 3. 最小示例

```rust
use nof0_backend::markets::ctp::{CtpConfig, RealCtpConnection};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建配置
    let config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "your_id".to_string(),
        password: "your_password".to_string(),
        md_address: "tcp://180.168.146.187:10211".to_string(),
        td_address: "tcp://180.168.146.187:10201".to_string(),
        app_id: "".to_string(),
        auth_code: "".to_string(),
        user_product_info: "nof0".to_string(),
        mock_mode: false,
    };

    // 连接
    let mut conn = RealCtpConnection::new(config);
    conn.connect().await?;

    // 订阅行情
    conn.subscribe_market_data(vec!["IF2501".to_string()]).await?;

    // 查询账户
    let account = conn.query_account().await?;
    println!("账户余额: {:.2}", account.balance);

    Ok(())
}
```

## 📚 文档

- [使用指南](../docs/CTP_USER_GUIDE.md) - 详细的使用文档
- [实现总结](CTP_IMPLEMENTATION_SUMMARY.md) - 技术实现细节
- [重连机制](TASK_7_RECONNECTION_SUMMARY.md) - 重连功能文档
- [示例代码](../examples/ctp_example.rs) - 完整示例

## 🔧 主要功能

### 连接管理

```rust
// 创建连接
let mut conn = RealCtpConnection::new(config);

// 设置重连参数
conn.set_max_reconnect_attempts(10);

// 连接到服务器
conn.connect().await?;

// 检查连接状态
if conn.is_connected().await {
    println!("连接正常");
}
```

### 行情订阅

```rust
// 订阅单个合约
conn.subscribe_market_data(vec!["IF2501".to_string()]).await?;

// 批量订阅
let instruments = vec![
    "IF2501".to_string(),
    "IC2501".to_string(),
    "IH2501".to_string(),
];
conn.subscribe_market_data(instruments).await?;

// 获取行情
let market_data = conn.get_market_data("IF2501").await?;
println!("最新价: {:.2}", market_data.last_price);
```

### 账户查询

```rust
// 查询账户
let account = conn.query_account().await?;
println!("余额: {:.2}", account.balance);
println!("可用: {:.2}", account.available);
println!("权益: {:.2}", account.equity());

// 查询持仓
let positions = conn.query_position().await?;
for pos in positions {
    println!("{}: {} {}", pos.instrument_id, pos.position, pos.profit);
}
```

### 订单交易

```rust
use nof0_backend::markets::ctp::CtpOrderRequest;

// 买入开仓
let order = CtpOrderRequest {
    instrument_id: "IF2501".to_string(),
    direction: '0',        // 买入
    offset_flag: '0',      // 开仓
    price: 5000.0,
    volume: 1,
    price_type: '2',       // 限价
    hedge_flag: '1',       // 投机
};

let response = conn.place_order(order).await?;
println!("订单引用: {}", response.order_ref);
```

### 重连机制

```rust
// 查询重连状态
let (md_reconnecting, td_reconnecting) = conn.get_reconnect_status().await;
let (md_attempts, td_attempts) = conn.get_reconnect_attempts();

if md_reconnecting {
    println!("行情重连中,已尝试 {} 次", md_attempts);
}
```

### 错误处理

```rust
use nof0_backend::markets::ctp::error_codes;

// 格式化错误
let msg = error_codes::format_ctp_error(error_code, Some("详细信息"));

// 判断错误类型
if error_codes::is_network_error(error_code) {
    println!("网络错误");
}

if error_codes::should_reconnect(error_code) {
    // 触发重连
}
```

## 🏗️ 架构设计

### 异步桥接

```
CTP C++ API (同步回调)
        ↓
    SPI Trait 实现
        ↓
   tokio::mpsc Channel
        ↓
   Async Rust 方法
```

### 通道架构

**行情通道** (4组):
- `md_connected` - 连接状态
- `md_login` - 登录结果
- `md_subscribe` - 订阅结果
- `market_data` - 行情数据流

**交易通道** (7组):
- `td_connected` - 连接状态
- `td_auth` - 认证结果
- `td_login` - 登录结果
- `order` - 订单回报流
- `trade` - 成交通知流
- `account_query` - 账户查询
- `position_query` - 持仓查询

### 缓存管理

```rust
Arc<RwLock<HashMap<String, CtpMarketData>>>   // 行情缓存
Arc<RwLock<HashMap<String, CtpPosition>>>     // 持仓缓存
Arc<RwLock<Option<CtpAccount>>>               // 账户缓存
```

## ⚙️ 配置

### SimNow 测试环境

```rust
let config = CtpConfig {
    broker_id: "9999".to_string(),
    investor_id: "your_simnow_id".to_string(),
    password: "your_simnow_password".to_string(),
    md_address: "tcp://180.168.146.187:10211".to_string(),
    td_address: "tcp://180.168.146.187:10201".to_string(),
    app_id: "".to_string(),
    auth_code: "".to_string(),
    user_product_info: "nof0".to_string(),
    mock_mode: false,
};
```

获取 SimNow 账号: http://www.simnow.com.cn/

### 生产环境

```rust
let config = CtpConfig {
    broker_id: "your_broker".to_string(),
    investor_id: "your_account".to_string(),
    password: "your_password".to_string(),
    md_address: "tcp://your_md_server:port".to_string(),
    td_address: "tcp://your_td_server:port".to_string(),
    app_id: "your_app_id".to_string(),      // 穿透式监管
    auth_code: "your_auth_code".to_string(),
    user_product_info: "your_product".to_string(),
    mock_mode: false,
};
```

## 📊 统计信息

| 项目 | 数量 |
|------|------|
| 代码总行数 | ~2600+ |
| 核心文件数 | 6 |
| 支持的功能 | 全部 CTP 功能 |
| 错误码映射 | 70+ |
| 测试覆盖 | 基础单元测试 |

## ⚠️ 注意事项

### 系统要求

1. **动态库依赖**:
   - Windows: `thostmduserapi_se.dll`, `thosttraderapi_se.dll`
   - Linux: `libthostmduserapi_se.so`, `libthosttraderapi_se.so`
   - macOS: `libthostmduserapi_se.dylib`, `libthosttraderapi_se.dylib`

2. **CTP 账号**:
   - SimNow 模拟账号 (测试)
   - 期货公司实盘账号 (生产)

### 限制

- **查询频率**: CTP 限制 1 秒 1 次查询 (系统自动处理)
- **订单撤单**: 需要等待报单回报后才能撤单
- **平今/平昨**: 上期所区分平今平昨,其他交易所不区分

### 安全提示

⚠️ **生产环境使用时**:
- 妥善保管账号密码
- 不要在代码中硬编码密钥
- 使用环境变量或配置文件
- 启用日志审计
- 实施风控策略

## 🧪 测试

### 运行示例

```bash
cargo run --example ctp_example --features ctp-real
```

### 单元测试

```bash
cargo test --features ctp-real
```

## 📈 进度

| 任务 | 状态 | 完成度 |
|------|------|--------|
| 行情 SPI | ✅ 完成 | 100% |
| 交易 SPI | ✅ 完成 | 100% |
| 连接管理 | ✅ 完成 | 100% |
| 订单交易 | ✅ 完成 | 100% |
| 账户查询 | ✅ 完成 | 100% |
| 错误处理 | ✅ 完成 | 100% |
| 重连机制 | ✅ 完成 | 100% |
| 文档示例 | ✅ 完成 | 100% |

**总进度**: 8/8 任务完成 (100%) ✅

## 🤝 贡献

欢迎提交 Issue 和 Pull Request!

## 📄 许可

本项目遵循 MIT 许可证。

---

**最后更新**: 2025年10月29日  
**版本**: 1.0.0  
**作者**: GitHub Copilot
