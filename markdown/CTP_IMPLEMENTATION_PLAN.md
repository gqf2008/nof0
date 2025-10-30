# CTP Real Mode 实现计划

**创建日期**: 2025-10-29  
**当前进度**: 40% → 目标 100%

---

## 📋 实现策略

基于ctp2rs库的API,我们需要:
1. 实现 `MdSpi` trait 用于行情回调
2. 实现 `TraderSpi` trait 用于交易回调
3. 使用 Rust async/channel 机制桥接CTP的C++回调和Rust异步代码
4. 实现状态管理和错误处理

---

## 🎯 Phase 1: 行情API实现 (高优先级)

### 1.1 行情SPI回调结构

```rust
// 创建独立的行情SPI处理器
struct CtpMdSpi {
    // 连接状态通道
    connected_tx: mpsc::Sender<bool>,
    login_tx: mpsc::Sender<Result<()>>,
    
    // 行情数据通道
    market_data_tx: mpsc::UnboundedSender<CtpMarketData>,
    
    // 请求ID计数器
    request_id: Arc<AtomicI32>,
}

impl MdSpi for CtpMdSpi {
    fn on_front_connected(&mut self) {
        // 通知连接成功
    }
    
    fn on_front_disconnected(&mut self, reason: i32) {
        // 通知连接断开
    }
    
    fn on_rsp_user_login(...) {
        // 登录响应处理
    }
    
    fn on_rtn_depth_market_data(&mut self, data: Option<&CThostFtdcDepthMarketDataField>) {
        // 行情推送处理
    }
}
```

### 1.2 行情连接和登录

**实现步骤**:
- [ ] 创建 CtpMdSpi 结构体
- [ ] 实现 on_front_connected 回调
- [ ] 实现 on_front_disconnected 回调
- [ ] 实现 req_user_login 登录请求
- [ ] 实现 on_rsp_user_login 登录响应
- [ ] 添加连接状态管理
- [ ] 添加超时和错误处理

**代码框架**:
```rust
#[cfg(feature = "ctp-real")]
async fn md_connect(&mut self) -> Result<()> {
    // 1. 创建行情SPI
    let spi = Box::new(CtpMdSpi::new(...));
    
    // 2. 创建MdApi
    let mdapi = MdApi::new("md_flow/", false, false);
    mdapi.register_spi(spi);
    mdapi.register_front(&self.config.md_address);
    mdapi.init();
    
    // 3. 等待连接
    wait_for_connect().await?;
    
    // 4. 登录
    let mut req = CThostFtdcReqUserLoginField::default();
    req.BrokerID.set_str(&self.config.broker_id);
    req.UserID.set_str(&self.config.investor_id);
    req.Password.set_str(&self.config.password);
    mdapi.req_user_login(&mut req, request_id);
    
    // 5. 等待登录响应
    wait_for_login().await?;
    
    Ok(())
}
```

### 1.3 行情订阅

**实现步骤**:
- [ ] 实现 subscribe_market_data 方法
- [ ] 实现 on_rsp_sub_market_data 回调
- [ ] 实现 on_rtn_depth_market_data 回调
- [ ] 转换CTP行情数据为CtpMarketData
- [ ] 更新行情缓存
- [ ] 通过channel发送行情更新

**数据转换**:
```rust
fn convert_market_data(ctp_data: &CThostFtdcDepthMarketDataField) -> CtpMarketData {
    CtpMarketData {
        instrument_id: ctp_data.InstrumentID.to_string(),
        last_price: ctp_data.LastPrice,
        bid_price: ctp_data.BidPrice1,
        bid_volume: ctp_data.BidVolume1,
        ask_price: ctp_data.AskPrice1,
        ask_volume: ctp_data.AskVolume1,
        volume: ctp_data.Volume as i64,
        open_interest: ctp_data.OpenInterest,
        update_time: format!("{}:{}:{}", 
            ctp_data.UpdateTime.hour(), 
            ctp_data.UpdateTime.minute(), 
            ctp_data.UpdateTime.second()
        ),
        trading_day: ctp_data.TradingDay.to_string(),
    }
}
```

---

## 🎯 Phase 2: 交易API实现 (高优先级)

### 2.1 交易SPI回调结构

```rust
struct CtpTraderSpi {
    // 连接和登录通道
    connected_tx: mpsc::Sender<bool>,
    login_tx: mpsc::Sender<Result<()>>,
    
    // 订单回报通道
    order_tx: mpsc::UnboundedSender<CtpOrderResponse>,
    trade_tx: mpsc::UnboundedSender<CtpTrade>,
    
    // 查询响应通道
    account_tx: mpsc::Sender<CtpAccount>,
    position_tx: mpsc::Sender<Vec<CtpPosition>>,
    
    // 状态管理
    request_id: Arc<AtomicI32>,
}

impl TraderSpi for CtpTraderSpi {
    fn on_front_connected(&mut self) { }
    fn on_front_disconnected(&mut self, reason: i32) { }
    fn on_rsp_authenticate(...) { }  // 认证响应
    fn on_rsp_user_login(...) { }
    fn on_rtn_order(...) { }  // 订单回报
    fn on_rtn_trade(...) { }  // 成交回报
    fn on_rsp_qry_trading_account(...) { }  // 账户查询响应
    fn on_rsp_qry_investor_position(...) { }  // 持仓查询响应
}
```

### 2.2 交易连接和认证

**实现步骤**:
- [ ] 创建 CtpTraderSpi 结构体
- [ ] 实现 on_front_connected 回调
- [ ] 实现认证流程 (req_authenticate)
- [ ] 实现 on_rsp_authenticate 回调
- [ ] 实现登录流程 (req_user_login)
- [ ] 实现 on_rsp_user_login 回调
- [ ] 添加认证失败重试

**认证和登录流程**:
```rust
async fn td_connect(&mut self) -> Result<()> {
    // 1. 创建交易SPI和API
    let spi = Box::new(CtpTraderSpi::new(...));
    let tdapi = TraderApi::new("td_flow/", false, false);
    tdapi.register_spi(spi);
    tdapi.register_front(&self.config.td_address);
    tdapi.init();
    
    // 2. 等待连接
    wait_for_connect().await?;
    
    // 3. 认证(如果需要)
    if !self.config.app_id.is_empty() {
        let mut auth_req = CThostFtdcReqAuthenticateField::default();
        auth_req.BrokerID.set_str(&self.config.broker_id);
        auth_req.UserID.set_str(&self.config.investor_id);
        auth_req.AppID.set_str(&self.config.app_id);
        auth_req.AuthCode.set_str(&self.config.auth_code);
        tdapi.req_authenticate(&mut auth_req, request_id);
        wait_for_auth().await?;
    }
    
    // 4. 登录
    let mut login_req = CThostFtdcReqUserLoginField::default();
    login_req.BrokerID.set_str(&self.config.broker_id);
    login_req.UserID.set_str(&self.config.investor_id);
    login_req.Password.set_str(&self.config.password);
    tdapi.req_user_login(&mut login_req, request_id);
    wait_for_login().await?;
    
    Ok(())
}
```

### 2.3 订单提交和回报

**实现步骤**:
- [ ] 实现 place_order 方法
- [ ] 构造 CThostFtdcInputOrderField
- [ ] 调用 req_order_insert
- [ ] 实现 on_rtn_order 回调
- [ ] 实现 on_rtn_trade 回报
- [ ] 实现 on_err_rtn_order_insert 错误回调
- [ ] 订单状态管理

**下单流程**:
```rust
async fn place_order_real(&self, req: CtpOrderRequest) -> Result<CtpOrderResponse> {
    let tdapi = self.td_api.as_ref().unwrap();
    
    // 1. 构造CTP订单请求
    let mut ctp_req = CThostFtdcInputOrderField::default();
    ctp_req.BrokerID.set_str(&self.config.broker_id);
    ctp_req.InvestorID.set_str(&self.config.investor_id);
    ctp_req.InstrumentID.set_str(&req.instrument_id);
    ctp_req.Direction = req.direction as i8;  // '0'买/'1'卖
    ctp_req.CombOffsetFlag[0] = req.offset_flag as i8;  // '0'开/'1'平
    ctp_req.LimitPrice = req.price;
    ctp_req.VolumeTotalOriginal = req.volume;
    ctp_req.OrderPriceType = req.price_type as i8;  // '2'限价
    ctp_req.CombHedgeFlag[0] = req.hedge_flag as i8;  // '1'投机
    ctp_req.TimeCondition = b'3' as i8;  // 当日有效
    ctp_req.VolumeCondition = b'1' as i8;  // 任意数量
    ctp_req.MinVolume = 1;
    ctp_req.ContingentCondition = b'1' as i8;  // 立即
    ctp_req.ForceCloseReason = b'0' as i8;  // 非强平
    
    // 2. 生成OrderRef
    let order_ref = self.get_next_order_ref();
    ctp_req.OrderRef.set_str(&order_ref);
    
    // 3. 发送订单
    let request_id = self.get_next_request_id();
    tdapi.req_order_insert(&mut ctp_req, request_id)?;
    
    // 4. 等待订单回报
    let response = self.wait_for_order_response(&order_ref).await?;
    
    Ok(response)
}
```

### 2.4 查询功能

**实现步骤**:
- [ ] 实现账户查询 (req_qry_trading_account)
- [ ] 实现 on_rsp_qry_trading_account 回调
- [ ] 实现持仓查询 (req_qry_investor_position)
- [ ] 实现 on_rsp_qry_investor_position 回调
- [ ] 添加查询流控 (间隔1秒)
- [ ] 数据转换和缓存更新

**账户查询**:
```rust
async fn query_account_real(&self) -> Result<CtpAccount> {
    let tdapi = self.td_api.as_ref().unwrap();
    
    // 1. 构造查询请求
    let mut req = CThostFtdcQryTradingAccountField::default();
    req.BrokerID.set_str(&self.config.broker_id);
    req.InvestorID.set_str(&self.config.investor_id);
    
    // 2. 流控检查(CTP限制查询频率)
    self.throttle.wait_if_needed().await;
    
    // 3. 发送查询请求
    let request_id = self.get_next_request_id();
    tdapi.req_qry_trading_account(&mut req, request_id)?;
    
    // 4. 等待查询响应
    let account = self.wait_for_account_response().await?;
    
    Ok(account)
}
```

---

## 🎯 Phase 3: 错误处理和稳定性

### 3.1 错误处理

**实现步骤**:
- [ ] CTP错误码映射
- [ ] RspInfo 错误解析
- [ ] 友好错误消息
- [ ] 错误日志记录

**错误处理工具**:
```rust
fn check_rsp_info(rsp_info: Option<&CThostFtdcRspInfoField>) -> Result<()> {
    if let Some(info) = rsp_info {
        if info.ErrorID != 0 {
            let error_msg = info.ErrorMsg.to_string();
            return Err(anyhow!(
                "CTP Error {}: {}",
                info.ErrorID,
                error_msg
            ));
        }
    }
    Ok(())
}
```

### 3.2 自动重连

**实现步骤**:
- [ ] 监听 on_front_disconnected
- [ ] 实现重连逻辑
- [ ] 指数退避策略
- [ ] 重新登录和恢复订阅
- [ ] 状态恢复

**重连逻辑**:
```rust
async fn handle_disconnection(&mut self) {
    let mut retry_count = 0;
    let max_retries = 5;
    
    while retry_count < max_retries {
        let backoff = Duration::from_secs(2u64.pow(retry_count));
        tokio::time::sleep(backoff).await;
        
        match self.reconnect().await {
            Ok(_) => {
                info!("Reconnected successfully");
                // 恢复订阅
                self.resubscribe_all().await?;
                break;
            }
            Err(e) => {
                error!("Reconnection attempt {} failed: {}", retry_count + 1, e);
                retry_count += 1;
            }
        }
    }
}
```

### 3.3 流控和限制

**实现步骤**:
- [ ] 查询请求流控(1秒间隔)
- [ ] 订单提交流控
- [ ] 请求队列管理
- [ ] 并发限制

**流控实现**:
```rust
struct RequestThrottle {
    last_request_time: RwLock<Instant>,
    min_interval: Duration,
}

impl RequestThrottle {
    async fn wait_if_needed(&self) {
        let last_time = *self.last_request_time.read().await;
        let elapsed = last_time.elapsed();
        
        if elapsed < self.min_interval {
            let sleep_time = self.min_interval - elapsed;
            tokio::time::sleep(sleep_time).await;
        }
        
        *self.last_request_time.write().await = Instant::now();
    }
}
```

---

## 🎯 Phase 4: 测试和文档

### 4.1 单元测试

- [ ] 连接测试
- [ ] 登录测试
- [ ] 行情订阅测试
- [ ] 订单提交测试
- [ ] 查询测试
- [ ] 错误处理测试
- [ ] 重连测试

### 4.2 集成测试

- [ ] SimNow环境测试
- [ ] 完整交易流程测试
- [ ] 压力测试
- [ ] 长时间运行测试

### 4.3 文档更新

- [ ] API文档
- [ ] 使用示例
- [ ] 配置说明
- [ ] 故障排查
- [ ] 最佳实践

---

## 📊 实现优先级

### 第1周: 基础连接和行情
- [x] 搭建CTP框架结构 (已完成 40%)
- [ ] 实现MdSpi回调
- [ ] 实现行情连接和登录
- [ ] 实现行情订阅
- [ ] 行情数据转换和缓存

### 第2周: 交易功能
- [ ] 实现TraderSpi回调
- [ ] 实现交易连接和认证
- [ ] 实现订单提交
- [ ] 实现订单回报处理
- [ ] 实现成交回报处理

### 第3周: 查询和稳定性
- [ ] 实现账户查询
- [ ] 实现持仓查询
- [ ] 实现错误处理
- [ ] 实现自动重连
- [ ] 实现流控机制

### 第4周: 测试和文档
- [ ] SimNow测试
- [ ] 单元测试覆盖
- [ ] 集成测试
- [ ] 文档完善
- [ ] 示例代码

---

## 🔧 关键技术挑战

### 1. C++回调 → Rust异步 桥接

**挑战**: CTP使用C++同步回调,Rust使用async/await
**解决方案**: 使用 tokio::sync::mpsc channel桥接

```rust
// SPI回调中发送事件
fn on_rtn_order(&mut self, order: Option<&CThostFtdcOrderField>) {
    self.order_tx.send(convert_order(order)).ok();
}

// 异步代码中接收事件
async fn wait_for_order(&mut self) -> Result<Order> {
    match self.order_rx.recv().await {
        Some(order) => Ok(order),
        None => Err(anyhow!("Channel closed")),
    }
}
```

### 2. 线程安全和状态管理

**挑战**: CTP回调可能在不同线程执行
**解决方案**: 使用 Arc<RwLock<T>> 和 Arc<AtomicXXX>

```rust
struct SharedState {
    md_connected: Arc<RwLock<bool>>,
    td_connected: Arc<RwLock<bool>>,
    request_id: Arc<AtomicI32>,
}
```

### 3. 请求-响应匹配

**挑战**: 需要匹配异步请求和回调响应
**解决方案**: 使用 request_id 和 HashMap 跟踪

```rust
struct PendingRequests {
    map: Arc<RwLock<HashMap<i32, oneshot::Sender<Response>>>>,
}

impl PendingRequests {
    async fn add(&self, request_id: i32) -> oneshot::Receiver<Response> {
        let (tx, rx) = oneshot::channel();
        self.map.write().await.insert(request_id, tx);
        rx
    }
    
    async fn complete(&self, request_id: i32, response: Response) {
        if let Some(tx) = self.map.write().await.remove(&request_id) {
            tx.send(response).ok();
        }
    }
}
```

---

## 📝 代码组织

```
src/markets/ctp/
├── adapter.rs          # Mock模式适配器
├── types.rs            # 数据类型定义
├── real_connection.rs  # Real模式主入口
├── md_spi.rs          # 行情SPI实现 (新增)
├── trader_spi.rs      # 交易SPI实现 (新增)
├── converter.rs       # 数据转换工具 (新增)
├── error.rs           # 错误处理 (新增)
└── mod.rs             # 模块导出
```

---

## ✅ 验收标准

### 功能完整性
- [ ] 可以连接到SimNow
- [ ] 可以登录(行情和交易)
- [ ] 可以订阅行情并接收推送
- [ ] 可以提交订单并接收回报
- [ ] 可以查询账户和持仓
- [ ] 断线自动重连

### 稳定性
- [ ] 可以连续运行24小时
- [ ] 错误处理完善
- [ ] 内存无泄漏
- [ ] 线程安全

### 代码质量
- [ ] 通过 cargo clippy
- [ ] 通过 cargo test
- [ ] 文档完整
- [ ] 示例可运行

---

**下一步行动**: 从Phase 1开始,实现行情API的基础功能
