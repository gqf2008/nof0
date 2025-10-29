# CTP 集成参考 - 基于 ctp2rs

## 项目概述

[ctp2rs](https://github.com/pseudocodes/ctp2rs) 是上海期货信息技术公司 CTP 接口的 Rust 原生封装。

### 核心特性

- ✅ 支持多种 CTP 版本: 生产、测评、CTP-Mini、CTP-Sopt 股票期权、OpenCTP、LocalCTP
- ✅ 跨平台: Windows、Linux、MacOS
- ✅ 动态库加载: 使用 libloading，可轻松切换不同柜台环境
- ✅ 保持与原生 CTP 接口一致的调用方式
- ✅ 灵活的 codegen 脚本

## 架构设计学习

### 1. 双 API 设计模式

CTP 提供两个核心 API：

#### MdApi - 行情 API
```rust
pub struct MdApi {
    api_ptr: *mut CThostFtdcMdApi,
}

impl MdApi {
    // 创建 API
    pub fn create_api(dynlib_path: &str, flow_path: &str, 
                      is_udp: bool, is_multicast: bool) -> Self;
    
    // 初始化
    pub fn init(&self);
    
    // 等待线程结束
    pub fn join(&self) -> i32;
    
    // 获取交易日
    pub fn get_trading_day(&self) -> String;
    
    // 注册前置机
    pub fn register_front(&self, front_address: &str);
    
    // 订阅行情
    pub fn subscribe_market_data(&self, instruments: &Vec<String>) -> i32;
    
    // 退订行情
    pub fn unsubscribe_market_data(&self, instruments: &Vec<String>) -> i32;
    
    // 用户登录
    pub fn req_user_login(&self, req: &mut CThostFtdcReqUserLoginField, 
                          request_id: i32) -> i32;
}
```

#### TraderApi - 交易 API
```rust
pub struct TraderApi {
    api_ptr: *mut CThostFtdcTraderApi,
}

impl TraderApi {
    // 创建 API
    pub fn create_api(dynlib_path: &str, flow_path: &str) -> Self;
    
    // 初始化
    pub fn init(&self);
    
    // 注册前置机
    pub fn register_front(&self, front_address: &str);
    
    // 用户登录
    pub fn req_user_login(&self, req: &mut CThostFtdcReqUserLoginField, 
                          request_id: i32) -> i32;
    
    // 报单录入
    pub fn req_order_insert(&self, req: &mut CThostFtdcInputOrderField,
                           request_id: i32) -> i32;
    
    // 报单操作 (撤单)
    pub fn req_order_action(&self, req: &mut CThostFtdcInputOrderActionField,
                           request_id: i32) -> i32;
    
    // 查询报单
    pub fn req_qry_order(&self, req: &mut CThostFtdcQryOrderField,
                        request_id: i32) -> i32;
    
    // 查询持仓
    pub fn req_qry_investor_position(&self, req: &mut CThostFtdcQryInvestorPositionField,
                                    request_id: i32) -> i32;
    
    // 查询资金
    pub fn req_qry_trading_account(&self, req: &mut CThostFtdcQryTradingAccountField,
                                   request_id: i32) -> i32;
}
```

### 2. 回调 Trait 设计

#### MdSpi - 行情回调
```rust
pub trait MdSpi: Send {
    // 连接建立
    fn on_front_connected(&mut self) {}
    
    // 连接断开
    fn on_front_disconnected(&mut self, reason: i32) {}
    
    // 登录响应
    fn on_rsp_user_login(&mut self, 
        login_field: Option<&CThostFtdcRspUserLoginField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 订阅行情响应
    fn on_rsp_sub_market_data(&mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 深度行情通知
    fn on_rtn_depth_market_data(&mut self,
        depth_market_data: Option<&CThostFtdcDepthMarketDataField>) {}
    
    // 错误应答
    fn on_rsp_error(&mut self,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
}
```

#### TraderSpi - 交易回调
```rust
pub trait TraderSpi: Send {
    // 连接建立
    fn on_front_connected(&mut self) {}
    
    // 连接断开
    fn on_front_disconnected(&mut self, reason: i32) {}
    
    // 登录响应
    fn on_rsp_user_login(&mut self,
        login_field: Option<&CThostFtdcRspUserLoginField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 报单录入响应
    fn on_rsp_order_insert(&mut self,
        input_order: Option<&CThostFtdcInputOrderField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 报单通知
    fn on_rtn_order(&mut self,
        order: Option<&CThostFtdcOrderField>) {}
    
    // 成交通知
    fn on_rtn_trade(&mut self,
        trade: Option<&CThostFtdcTradeField>) {}
    
    // 查询报单响应
    fn on_rsp_qry_order(&mut self,
        order: Option<&CThostFtdcOrderField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 查询持仓响应
    fn on_rsp_qry_investor_position(&mut self,
        position: Option<&CThostFtdcInvestorPositionField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
    
    // 查询资金响应
    fn on_rsp_qry_trading_account(&mut self,
        account: Option<&CThostFtdcTradingAccountField>,
        rsp_info: Option<&CThostFtdcRspInfoField>,
        request_id: i32,
        is_last: bool) {}
}
```

### 3. Channel 模式处理异步事件

关键学习：使用 channel 将回调事件传递到主线程

```rust
use tokio::sync::mpsc;

// 定义事件类型
#[derive(Debug)]
pub enum MdSpiEvent {
    OnFrontConnected,
    OnFrontDisconnected(i32),
    OnRspUserLogin(RspUserLoginEvent),
    OnRtnDepthMarketData(DepthMarketDataEvent),
    OnRspSubMarketData(RspSubMarketDataEvent),
}

// 实现 MdSpi
struct ChannelMdSpi {
    tx: mpsc::UnboundedSender<MdSpiEvent>,
}

impl MdSpi for ChannelMdSpi {
    fn on_front_connected(&mut self) {
        self.tx.send(MdSpiEvent::OnFrontConnected).unwrap();
    }
    
    fn on_rtn_depth_market_data(&mut self, data: Option<&CThostFtdcDepthMarketDataField>) {
        self.tx.send(MdSpiEvent::OnRtnDepthMarketData(
            DepthMarketDataEvent {
                data: data.cloned(),
            }
        )).unwrap();
    }
}

// 主线程处理
async fn process_events(mut rx: mpsc::UnboundedReceiver<MdSpiEvent>) {
    while let Some(event) = rx.recv().await {
        match event {
            MdSpiEvent::OnFrontConnected => {
                println!("行情前置已连接");
            }
            MdSpiEvent::OnRtnDepthMarketData(event) => {
                if let Some(data) = event.data {
                    println!("收到行情: {} 价格: {}", 
                        data.InstrumentID.to_string(), 
                        data.LastPrice);
                }
            }
            _ => {}
        }
    }
}
```

## 为 nof0 设计 CTP Adapter

### MarketAdapter 实现

```rust
use ctp2rs::v1alpha1::{MdApi, MdSpi, CThostFtdcDepthMarketDataField};
use tokio::sync::mpsc;

pub struct CtpMarketAdapter {
    md_api: MdApi,
    event_rx: mpsc::UnboundedReceiver<MdSpiEvent>,
    instruments: Vec<String>,
}

impl CtpMarketAdapter {
    pub fn new(
        ctp_dll_path: &str,
        front_address: &str,
        broker_id: &str,
        user_id: &str,
        password: &str,
    ) -> Self {
        // 创建 channel
        let (tx, rx) = mpsc::unbounded_channel();
        
        // 创建 API
        let md_api = MdApi::create_api(ctp_dll_path, "./md_", false, false);
        
        // 注册 Spi
        let spi = Box::new(ChannelMdSpi { tx });
        md_api.register_spi(spi);
        
        // 注册前置机
        md_api.register_front(front_address);
        
        // 初始化
        md_api.init();
        
        Self {
            md_api,
            event_rx: rx,
            instruments: Vec::new(),
        }
    }
    
    pub async fn subscribe(&mut self, instruments: Vec<String>) {
        self.instruments.extend(instruments.clone());
        self.md_api.subscribe_market_data(&instruments);
    }
    
    pub async fn process_events(&mut self) {
        while let Some(event) = self.event_rx.recv().await {
            match event {
                MdSpiEvent::OnRtnDepthMarketData(data) => {
                    self.handle_market_data(data).await;
                }
                _ => {}
            }
        }
    }
    
    async fn handle_market_data(&self, event: DepthMarketDataEvent) {
        // 处理行情数据
        // 转换为统一的 Price 结构
    }
}

#[async_trait]
impl MarketAdapter for CtpMarketAdapter {
    async fn get_price(&self, symbol: &str) -> Result<Price, anyhow::Error> {
        // 从缓存获取最新价格
        Ok(Price {
            symbol: symbol.to_string(),
            price: 0.0,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn place_order(&self, order: Order) -> Result<String, anyhow::Error> {
        // CTP 不在 MdApi 下单，需要 TraderApi
        Err(anyhow::anyhow!("Use CtpTraderAdapter for placing orders"))
    }
    
    async fn get_balance(&self, account_id: &str) -> Result<Vec<Balance>, anyhow::Error> {
        // 需要 TraderApi 查询
        Err(anyhow::anyhow!("Use CtpTraderAdapter for balance queries"))
    }
    
    fn market_name(&self) -> &str {
        "CTP"
    }
}
```

### TraderAdapter 实现

```rust
pub struct CtpTraderAdapter {
    trader_api: TraderApi,
    event_rx: mpsc::UnboundedReceiver<TraderSpiEvent>,
    broker_id: String,
    user_id: String,
    password: String,
    request_id: AtomicI32,
}

impl CtpTraderAdapter {
    pub fn new(
        ctp_dll_path: &str,
        front_address: &str,
        broker_id: &str,
        user_id: &str,
        password: &str,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let trader_api = TraderApi::create_api(ctp_dll_path, "./trader_");
        let spi = Box::new(ChannelTraderSpi { tx });
        trader_api.register_spi(spi);
        trader_api.register_front(front_address);
        trader_api.init();
        
        Self {
            trader_api,
            event_rx: rx,
            broker_id: broker_id.to_string(),
            user_id: user_id.to_string(),
            password: password.to_string(),
            request_id: AtomicI32::new(1),
        }
    }
    
    fn next_request_id(&self) -> i32 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
    
    pub async fn login(&mut self) -> Result<(), anyhow::Error> {
        let mut req = CThostFtdcReqUserLoginField::default();
        req.BrokerID = self.broker_id.clone().into();
        req.UserID = self.user_id.clone().into();
        req.Password = self.password.clone().into();
        
        self.trader_api.req_user_login(&mut req, self.next_request_id());
        
        // 等待登录响应
        // ...
        
        Ok(())
    }
}

#[async_trait]
impl MarketAdapter for CtpTraderAdapter {
    async fn place_order(&self, order: Order) -> Result<String, anyhow::Error> {
        let mut req = CThostFtdcInputOrderField::default();
        req.BrokerID = self.broker_id.clone().into();
        req.InvestorID = self.user_id.clone().into();
        req.InstrumentID = order.symbol.into();
        req.OrderPriceType = THOST_FTDC_OPT_LimitPrice; // 限价单
        req.Direction = if order.side == "buy" { 
            THOST_FTDC_D_Buy 
        } else { 
            THOST_FTDC_D_Sell 
        };
        req.LimitPrice = order.price;
        req.VolumeTotalOriginal = order.quantity as i32;
        req.TimeCondition = THOST_FTDC_TC_GFD; // 当日有效
        req.VolumeCondition = THOST_FTDC_VC_AV; // 任意数量
        req.MinVolume = 1;
        req.ContingentCondition = THOST_FTDC_CC_Immediately; // 立即
        req.ForceCloseReason = THOST_FTDC_FCC_NotForceClose; // 非强平
        
        let request_id = self.next_request_id();
        self.trader_api.req_order_insert(&mut req, request_id);
        
        // 等待报单响应
        // ...
        
        Ok(request_id.to_string())
    }
    
    async fn get_balance(&self, account_id: &str) -> Result<Vec<Balance>, anyhow::Error> {
        let mut req = CThostFtdcQryTradingAccountField::default();
        req.BrokerID = self.broker_id.clone().into();
        req.InvestorID = self.user_id.clone().into();
        
        self.trader_api.req_qry_trading_account(&mut req, self.next_request_id());
        
        // 等待查询响应
        // ...
        
        Ok(vec![])
    }
    
    fn market_name(&self) -> &str {
        "CTP"
    }
}
```

## 关键学习点

### 1. 动态库加载
```rust
// 不同平台加载不同动态库
#[cfg(target_os = "windows")]
let dll_path = "./thostmduserapi_se.dll";

#[cfg(target_os = "linux")]
let dll_path = "./thostmduserapi_se.so";

#[cfg(target_os = "macos")]
let dll_path = "./thostmduserapi_se.framework/thostmduserapi_se";
```

### 2. Request ID 管理
```rust
use std::sync::atomic::{AtomicI32, Ordering};

struct RequestIdManager {
    counter: AtomicI32,
}

impl RequestIdManager {
    fn new() -> Self {
        Self { counter: AtomicI32::new(1) }
    }
    
    fn next(&self) -> i32 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}
```

### 3. 查询流控
```rust
// CTP 有查询频率限制，通常为 1秒/次
use tokio::time::{sleep, Duration};

async fn query_with_flow_control<F, R>(&self, query_fn: F) -> Result<R, anyhow::Error>
where
    F: FnOnce() -> Result<R, anyhow::Error>,
{
    sleep(Duration::from_secs(1)).await;
    query_fn()
}
```

### 4. 错误处理
```rust
fn check_rsp_info(rsp_info: Option<&CThostFtdcRspInfoField>) -> Result<(), anyhow::Error> {
    if let Some(info) = rsp_info {
        if info.ErrorID != 0 {
            return Err(anyhow::anyhow!(
                "CTP Error {}: {}", 
                info.ErrorID, 
                info.ErrorMsg.to_string()
            ));
        }
    }
    Ok(())
}
```

## 配置示例

```yaml
# nof0.yaml
ctp:
  # 动态库路径
  dll_path:
    windows: "./ctp/thostmduserapi_se.dll"
    linux: "./ctp/thostmduserapi_se.so"
    macos: "./ctp/thostmduserapi_se.framework/thostmduserapi_se"
  
  # 行情前置
  md_front: "tcp://180.168.146.187:10131"
  
  # 交易前置
  trader_front: "tcp://180.168.146.187:10130"
  
  # 认证信息
  broker_id: "9999"
  user_id: "your_user_id"
  password: "your_password"
  app_id: "simnow_client_test"
  auth_code: "0000000000000000"
  
  # 订阅合约
  instruments:
    - "IF2503"  # 沪深300指数期货
    - "IC2503"  # 中证500指数期货
    - "IH2503"  # 上证50指数期货
    - "rb2505"  # 螺纹钢
    - "au2506"  # 黄金
```

## 测试环境

### SimNow 仿真环境
- 行情前置: `tcp://180.168.146.187:10131`
- 交易前置: `tcp://180.168.146.187:10130`
- 用户注册: https://www.simnow.com.cn/

### OpenCTP
- 开源 CTP 兼容实现
- 支持更多交易所
- https://github.com/openctp/openctp

## 实现计划

### Phase 1: 行情接入
1. ✅ 学习 ctp2rs 架构
2. [ ] 创建 CtpMarketAdapter
3. [ ] 实现行情订阅
4. [ ] 行情数据缓存
5. [ ] Channel 事件处理

### Phase 2: 交易接入
1. [ ] 创建 CtpTraderAdapter
2. [ ] 实现登录认证
3. [ ] 实现报单录入
4. [ ] 实现撤单
5. [ ] 实现查询接口

### Phase 3: 集成到 TradingEngine
1. [ ] 注册 CTP Adapter
2. [ ] LLM 调用 CTP 工具
3. [ ] 风险控制
4. [ ] 订单管理
5. [ ] 持仓管理

### Phase 4: 监控和日志
1. [ ] 连接状态监控
2. [ ] 订单状态追踪
3. [ ] 异常处理和重连
4. [ ] 审计日志

## 优势和挑战

### 优势
- ✅ 直接访问中国期货市场
- ✅ 低延迟
- ✅ 完整的交易功能
- ✅ Rust 类型安全

### 挑战
- ⚠️ CTP API 学习曲线陡峭
- ⚠️ 回调模式需要特殊处理
- ⚠️ 查询有流控限制
- ⚠️ 需要处理重连逻辑
- ⚠️ 不同版本 API 可能不兼容

## 参考资源

- [ctp2rs GitHub](https://github.com/pseudocodes/ctp2rs)
- [CTP 官方文档](http://www.sfit.com.cn/DocumentDown/api_3/index.htm)
- [SimNow 仿真平台](https://www.simnow.com.cn/)
- [OpenCTP](https://github.com/openctp/openctp)
- [go2ctp (Go 语言版本)](https://github.com/pseudocodes/go2ctp)

## 下一步

建议按以下顺序实现：
1. 先实现 CtpMarketAdapter (只读，风险低)
2. 用 SimNow 环境测试
3. 实现 CtpTraderAdapter (写操作，需谨慎)
4. 集成到 TradingEngine
5. 添加完善的风控和监控

**重要提示**: CTP 连接真实账户前，务必在 SimNow 环境充分测试！
