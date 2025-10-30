# CTP Real Mode 集成状态报告

**更新日期**: 2025-01-18  
**状态**: ⚠️ 框架已集成,核心功能待实现

---

## ✅ 已完成

### 1. 依赖集成

**Cargo.toml**:
```toml
# CTP期货交易接口 (https://github.com/pseudocodes/ctp2rs)
ctp2rs = { version = "0.1.8", optional = true }

[features]
default = []
ctp-real = ["ctp2rs"]  # 可选feature
```

**说明**:
- ✅ 已添加 `ctp2rs` 依赖 (v0.1.8)
- ✅ 设为可选依赖(optional = true)
- ✅ 通过feature gate控制(ctp-real)
- ✅ 默认不启用,避免需要CTP SDK动态库

### 2. Real模式框架

**文件**: `backend/src/markets/ctp/real_connection.rs` (~280行)

**已实现结构**:
```rust
pub struct RealCtpConnection {
    config: CtpConfig,
    
    #[cfg(feature = "ctp-real")]
    md_api: Option<Arc<MdApi>>,        // 行情API
    
    #[cfg(feature = "ctp-real")]
    td_api: Option<Arc<TraderApi>>,    // 交易API
    
    market_data: Arc<RwLock<HashMap<String, CtpMarketData>>>,
    positions: Arc<RwLock<HashMap<String, CtpPosition>>>,
    account: Arc<RwLock<Option<CtpAccount>>>,
    
    // 连接状态
    md_connected: Arc<RwLock<bool>>,
    td_connected: Arc<RwLock<bool>>,
    md_logged_in: Arc<RwLock<bool>>,
    td_logged_in: Arc<RwLock<bool>>,
    
    // 事件通道
    market_data_tx: mpsc::UnboundedSender<CtpMarketData>,
    market_data_rx: Arc<RwLock<mpsc::UnboundedReceiver<CtpMarketData>>>,
}
```

**已实现方法**:
- ✅ `new()` - 创建连接实例
- ✅ `connect()` - 连接框架(带feature gate)
- ✅ `disconnect()` - 断开连接
- ✅ `is_connected()` - 连接状态检查
- ✅ `subscribe_market_data()` - 订阅框架
- ✅ `place_order()` - 下单框架
- ✅ `query_account()` - 账户查询
- ✅ `query_position()` - 持仓查询
- ✅ `get_market_data()` - 行情数据获取

### 3. Feature Gate保护

**编译时条件编译**:
```rust
#[cfg(feature = "ctp-real")]
pub async fn connect(&mut self) -> Result<()> {
    // 真实CTP连接代码
}

#[cfg(not(feature = "ctp-real"))]
pub async fn connect(&mut self) -> Result<()> {
    Err(anyhow!(
        "CTP Real Mode is not enabled. \
         Please compile with --features ctp-real"
    ))
}
```

**优势**:
- ✅ 不启用feature时,不依赖CTP SDK
- ✅ 编译器会优化掉未使用的代码
- ✅ 友好的错误提示信息

### 4. 文档

**已创建**:
- ✅ `README_REAL_MODE.md` - 完整配置指南
- ✅ 前置要求说明
- ✅ 编译和运行步骤
- ✅ 常见问题解答
- ✅ 安全注意事项

---

## ⏳ 待实现功能

### 高优先级

#### 1. CTP连接回调处理

**需要实现**:
```rust
// OnFrontConnected回调
fn on_front_connected(&self) {
    *self.md_connected.write().await = true;
}

// OnFrontDisconnected回调
fn on_front_disconnected(&self, reason: i32) {
    *self.md_connected.write().await = false;
    // 触发重连逻辑
}

// OnHeartBeatWarning回调
fn on_heartbeat_warning(&self, time_lapse: i32) {
    warn!("Heartbeat warning: {} seconds", time_lapse);
}
```

**挑战**:
- CTP API使用回调模式
- 需要处理线程安全
- 需要状态同步机制

#### 2. 登录流程

**需要实现**:
```rust
async fn login(&self) -> Result<()> {
    // 1. 构造登录请求
    let req = ReqUserLogin {
        broker_id: self.config.broker_id.clone(),
        user_id: self.config.investor_id.clone(),
        password: self.config.password.clone(),
        // ...
    };
    
    // 2. 发送登录请求
    self.td_api.req_user_login(&req, request_id)?;
    
    // 3. 等待登录响应
    self.wait_for_login().await?;
    
    Ok(())
}

// 登录响应回调
fn on_rsp_user_login(&self, login_field: &RspUserLogin, error: &RspInfo) {
    if error.error_id == 0 {
        *self.td_logged_in.write().await = true;
    } else {
        error!("Login failed: {}", error.error_msg);
    }
}
```

#### 3. 行情订阅和推送

**需要实现**:
```rust
pub async fn subscribe_market_data(&self, instruments: Vec<String>) -> Result<()> {
    let md_api = self.md_api.as_ref().unwrap();
    
    // 转换为C字符串数组
    let instrument_refs: Vec<&str> = instruments.iter()
        .map(|s| s.as_str())
        .collect();
    
    // 调用CTP API
    md_api.subscribe_market_data(&instrument_refs)?;
    
    Ok(())
}

// 行情推送回调
fn on_rtn_depth_market_data(&self, data: &DepthMarketData) {
    // 1. 转换为CtpMarketData
    let market_data = CtpMarketData {
        instrument_id: data.instrument_id.clone(),
        last_price: data.last_price,
        bid_price: data.bid_price_1,
        ask_price: data.ask_price_1,
        // ...
    };
    
    // 2. 更新缓存
    self.market_data.write().await
        .insert(data.instrument_id.clone(), market_data.clone());
    
    // 3. 发送到事件通道
    self.market_data_tx.send(market_data).ok();
}
```

#### 4. 订单提交和回报

**需要实现**:
```rust
pub async fn place_order(&self, request: CtpOrderRequest) -> Result<CtpOrderResponse> {
    let td_api = self.td_api.as_ref().unwrap();
    
    // 1. 构造CTP报单请求
    let req = InputOrder {
        broker_id: self.config.broker_id.clone(),
        investor_id: self.config.investor_id.clone(),
        instrument_id: request.instrument_id.clone(),
        direction: request.direction,
        offset_flag: request.offset_flag,
        price: request.price,
        volume: request.volume,
        // ...
    };
    
    // 2. 提交订单
    let request_id = self.get_next_request_id();
    td_api.req_order_insert(&req, request_id)?;
    
    // 3. 等待回报
    let response = self.wait_for_order_response(request_id).await?;
    
    Ok(response)
}

// 订单回报回调
fn on_rtn_order(&self, order: &Order) {
    // 更新订单状态
    // 如果全部成交,更新持仓
}

// 成交回报回调
fn on_rtn_trade(&self, trade: &Trade) {
    // 更新持仓
    // 更新账户资金
}
```

### 中优先级

#### 5. 查询功能

**账户查询**:
```rust
pub async fn query_account_real(&self) -> Result<CtpAccount> {
    let td_api = self.td_api.as_ref().unwrap();
    
    let req = QryTradingAccount {
        broker_id: self.config.broker_id.clone(),
        investor_id: self.config.investor_id.clone(),
        // ...
    };
    
    td_api.req_qry_trading_account(&req, request_id)?;
    
    // 等待查询响应
    self.wait_for_account_response().await
}

fn on_rsp_qry_trading_account(&self, account: &TradingAccount) {
    let ctp_account = CtpAccount {
        account_id: self.config.investor_id.clone(),
        balance: account.balance,
        available: account.available,
        margin: account.curr_margin,
        // ...
    };
    
    *self.account.write().await = Some(ctp_account);
}
```

**持仓查询**:
```rust
pub async fn query_position_real(&self) -> Result<Vec<CtpPosition>> {
    // 类似账户查询
    // ReqQryInvestorPosition
    // OnRspQryInvestorPosition
}
```

#### 6. 错误处理和重连

**自动重连**:
```rust
async fn handle_disconnection(&self) {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        match self.reconnect().await {
            Ok(_) => {
                info!("Reconnected successfully");
                break;
            }
            Err(e) => {
                error!("Reconnection failed: {}", e);
            }
        }
    }
}
```

**流控处理**:
```rust
struct RequestThrottle {
    last_request_time: Instant,
    min_interval: Duration,
}

impl RequestThrottle {
    async fn wait_if_needed(&mut self) {
        let elapsed = self.last_request_time.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        self.last_request_time = Instant::now();
    }
}
```

### 低优先级

#### 7. 高级功能

- ⏳ 条件单
- ⏳ 止损止盈
- ⏳ 算法订单
- ⏳ 套利订单

---

## 🔧 如何启用Real模式

### 编译

```bash
# 不启用feature (默认,只有Mock模式)
cargo build

# 启用ctp-real feature
cargo build --features ctp-real

# 运行示例
cargo run --example ctp_market_demo --features ctp-real
```

### 依赖要求

#### Windows
1. 下载CTP SDK (http://www.sfit.com.cn/)
2. 复制DLL到系统目录或项目目录:
   - `thostmduserapi_se.dll`
   - `thosttraderapi_se.dll`

#### Linux
1. 下载CTP SDK
2. 复制SO到系统库目录:
   - `libthostmduserapi_se.so`
   - `libthosttraderapi_se.so`
3. 或添加到LD_LIBRARY_PATH

### 配置

```yaml
# etc/ctp_config.yaml
mock_mode: false  # ← 改为false启用Real模式
broker_id: "9999"
investor_id: "YOUR_ACCOUNT"
password: "${CTP_PASSWORD}"
md_address: "tcp://180.168.146.187:10131"
td_address: "tcp://180.168.146.187:10130"
```

---

## 📊 实现进度

### 总体进度: 40%

| 模块 | 进度 | 说明 |
|------|------|------|
| **依赖集成** | 100% | ✅ ctp-futures已添加 |
| **框架结构** | 100% | ✅ RealCtpConnection创建 |
| **连接回调** | 0% | ⏳ 待实现 |
| **登录流程** | 0% | ⏳ 待实现 |
| **行情订阅** | 20% | ⏳ 框架已搭建,回调待实现 |
| **订单提交** | 20% | ⏳ 框架已搭建,回报待实现 |
| **账户查询** | 10% | ⏳ 接口已定义,实现待完成 |
| **持仓查询** | 10% | ⏳ 接口已定义,实现待完成 |
| **错误处理** | 0% | ⏳ 待实现 |
| **自动重连** | 0% | ⏳ 待实现 |

### 代码统计

```
src/markets/ctp/
├── adapter.rs          (280行) - Mock模式适配器 ✅
├── types.rs            (300行) - 数据类型定义 ✅
├── real_connection.rs  (280行) - Real模式框架 ⚠️ 40%
├── mod.rs              (9行)   - 模块导出 ✅
└── README_REAL_MODE.md (450行) - 配置文档 ✅

Total: ~1,319 lines
```

---

## 🎯 下一步行动

### Phase 1: 基础连接 (1-2周)

1. **实现连接回调**
   - [ ] OnFrontConnected
   - [ ] OnFrontDisconnected
   - [ ] 状态同步机制

2. **实现登录流程**
   - [ ] ReqUserLogin
   - [ ] OnRspUserLogin
   - [ ] 错误处理

3. **测试SimNow连接**
   - [ ] 注册SimNow账号
   - [ ] 测试连接和登录
   - [ ] 验证状态管理

### Phase 2: 行情和交易 (2-3周)

1. **实现行情订阅**
   - [ ] SubscribeMarketData
   - [ ] OnRtnDepthMarketData
   - [ ] 行情缓存更新
   - [ ] 事件通道分发

2. **实现订单提交**
   - [ ] ReqOrderInsert
   - [ ] OnRtnOrder
   - [ ] OnRtnTrade
   - [ ] 订单状态管理

3. **实现持仓更新**
   - [ ] 成交回报处理
   - [ ] 持仓自动计算
   - [ ] 资金自动更新

### Phase 3: 查询和稳定性 (1-2周)

1. **实现查询功能**
   - [ ] ReqQryTradingAccount
   - [ ] ReqQryInvestorPosition
   - [ ] 查询节流控制

2. **错误处理**
   - [ ] 错误码映射
   - [ ] 友好错误信息
   - [ ] 日志记录

3. **自动重连**
   - [ ] 断线检测
   - [ ] 重连逻辑
   - [ ] 状态恢复

### Phase 4: 完善和优化 (1-2周)

1. **性能优化**
   - [ ] 内存池
   - [ ] 事件批处理
   - [ ] 缓存优化

2. **文档完善**
   - [ ] API文档
   - [ ] 示例代码
   - [ ] 最佳实践

3. **测试覆盖**
   - [ ] 单元测试
   - [ ] 集成测试
   - [ ] 压力测试

---

## 🤝 贡献指南

### 如何贡献

1. **选择一个TODO任务**
2. **Fork项目并创建分支**
3. **实现功能并添加测试**
4. **提交PR**

### 开发环境

```bash
# 克隆项目
git clone https://github.com/yourusername/nof0.git
cd nof0/backend

# 安装CTP SDK (下载并放置DLL/SO)

# 编译
cargo build --features ctp-real

# 运行测试
cargo test --features ctp-real

# 运行示例
cargo run --example ctp_market_demo --features ctp-real
```

### 代码规范

- 使用rustfmt格式化代码
- 使用clippy检查代码质量
- 添加文档注释
- 编写单元测试

---

## 📚 参考资源

### CTP官方

- **开发者网站**: http://www.sfit.com.cn/
- **SimNow**: http://www.simnow.com.cn/
- **API文档**: CTP SDK中的PDF文档

### Rust库

- **ctp2rs**: https://github.com/pseudocodes/ctp2rs
- **crates.io**: https://crates.io/crates/ctp2rs
- **ctp2rs文档**: https://docs.rs/ctp2rs/

### 项目文档

- [CTP_ADAPTER.md](../../../markdown/CTP_ADAPTER.md)
- [CTP_ADAPTER_QUICKSTART.md](../../../markdown/CTP_ADAPTER_QUICKSTART.md)
- [README_REAL_MODE.md](./README_REAL_MODE.md)

---

## ⚠️ 注意事项

### 当前状态

**Real模式是框架代码,核心功能尚未实现!**

- ✅ 依赖已集成
- ✅ 结构已搭建
- ⏳ 连接、行情、交易功能待实现

**建议**:
- 开发测试请使用Mock模式
- Real模式正在积极开发中
- 欢迎贡献代码!

### 风险提示

**实盘交易前必须**:
1. ✅ 在SimNow充分测试
2. ✅ 启用风控系统
3. ✅ 小资金试跑
4. ✅ 实时监控

---

**更新日期**: 2025-01-18  
**维护者**: nof0 Development Team  
**状态**: ⚠️ 开发中

**欢迎贡献!** 🚀
