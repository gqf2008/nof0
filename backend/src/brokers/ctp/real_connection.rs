// CTP Real Mode Connection Implementation
// 真实CTP连接实现(需要启用 ctp-real feature)

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};

use super::error_codes;
use super::types::{
    CtpAccount, CtpConfig, CtpMarketData, CtpOrderRequest, CtpOrderResponse, CtpPosition,
};

#[cfg(feature = "ctp-real")]
use ctp2rs::v1alpha1::{
    CThostFtdcInputOrderField, CThostFtdcQryInvestorPositionField,
    CThostFtdcQryTradingAccountField, CThostFtdcReqAuthenticateField, CThostFtdcReqUserLoginField,
    MdApi, TraderApi,
};

#[cfg(feature = "ctp-real")]
use super::md_spi::CtpMdSpi;

#[cfg(feature = "ctp-real")]
use super::trader_spi::CtpTraderSpi;

/// CTP真实连接
///
/// 此结构体管理与CTP前置服务器的真实连接。
/// 需要启用 `ctp-real` feature 并确保CTP SDK动态库在系统路径中。
///
/// # 依赖要求
///
/// ## Windows
/// - thostmduserapi_se.dll
/// - thosttraderapi_se.dll
///
/// ## Linux
/// - libthostmduserapi_se.so
/// - libthosttraderapi_se.so
///
/// # 启用方式
///
/// ```bash
/// cargo build --features ctp-real
/// ```
pub struct RealCtpConnection {
    config: CtpConfig,

    // 行情API
    #[cfg(feature = "ctp-real")]
    md_api: Option<Arc<MdApi>>,

    // 交易API
    #[cfg(feature = "ctp-real")]
    td_api: Option<Arc<TraderApi>>,

    // 数据缓存
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
    market_data_rx: Option<mpsc::UnboundedReceiver<CtpMarketData>>,

    // 行情SPI通道 (用于SPI回调)
    #[cfg(feature = "ctp-real")]
    md_connected_tx: Option<mpsc::Sender<bool>>,
    #[cfg(feature = "ctp-real")]
    md_connected_rx: Option<mpsc::Receiver<bool>>,
    #[cfg(feature = "ctp-real")]
    md_login_tx: Option<mpsc::Sender<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    md_login_rx: Option<mpsc::Receiver<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    md_subscribe_tx: Option<mpsc::Sender<Result<String, String>>>,
    #[cfg(feature = "ctp-real")]
    md_subscribe_rx: Option<mpsc::Receiver<Result<String, String>>>,

    // 交易SPI通道 (用于TraderSpi回调)
    #[cfg(feature = "ctp-real")]
    td_connected_tx: Option<mpsc::Sender<bool>>,
    #[cfg(feature = "ctp-real")]
    td_connected_rx: Option<mpsc::Receiver<bool>>,
    #[cfg(feature = "ctp-real")]
    td_auth_tx: Option<mpsc::Sender<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    td_auth_rx: Option<mpsc::Receiver<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    td_login_tx: Option<mpsc::Sender<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    td_login_rx: Option<mpsc::Receiver<Result<(), String>>>,
    #[cfg(feature = "ctp-real")]
    order_tx: Option<mpsc::UnboundedSender<CtpOrderResponse>>,
    #[cfg(feature = "ctp-real")]
    order_rx: Option<mpsc::UnboundedReceiver<CtpOrderResponse>>,
    #[cfg(feature = "ctp-real")]
    trade_tx: Option<mpsc::UnboundedSender<String>>,
    #[cfg(feature = "ctp-real")]
    trade_rx: Option<mpsc::UnboundedReceiver<String>>,
    #[cfg(feature = "ctp-real")]
    account_query_tx: Option<mpsc::Sender<Result<CtpAccount, String>>>,
    #[cfg(feature = "ctp-real")]
    account_query_rx: Option<mpsc::Receiver<Result<CtpAccount, String>>>,
    #[cfg(feature = "ctp-real")]
    position_query_tx: Option<mpsc::Sender<Result<Vec<CtpPosition>, String>>>,
    #[cfg(feature = "ctp-real")]
    position_query_rx: Option<mpsc::Receiver<Result<Vec<CtpPosition>, String>>>,

    // 请求ID计数器
    #[cfg(feature = "ctp-real")]
    request_id: Arc<AtomicI32>,

    // 查询流控 (最后查询时间)
    #[cfg(feature = "ctp-real")]
    last_query_time: Arc<Mutex<Option<Instant>>>,

    // 重连机制相关字段
    md_reconnect_attempts: Arc<AtomicI32>,
    td_reconnect_attempts: Arc<AtomicI32>,
    max_reconnect_attempts: i32,
    is_md_reconnecting: Arc<RwLock<bool>>,
    is_td_reconnecting: Arc<RwLock<bool>>,

    // 订阅状态保存 (用于重连后恢复)
    subscribed_instruments: Arc<RwLock<Vec<String>>>,
}

impl RealCtpConnection {
    /// 创建新的CTP真实连接
    pub fn new(config: CtpConfig) -> Self {
        let (market_data_tx, market_data_rx) = mpsc::unbounded_channel();

        #[cfg(feature = "ctp-real")]
        {
            // 创建SPI通信通道
            let (md_connected_tx, md_connected_rx) = mpsc::channel(10);
            let (md_login_tx, md_login_rx) = mpsc::channel(10);
            let (md_subscribe_tx, md_subscribe_rx) = mpsc::channel(100);

            // 创建交易SPI通道
            let (td_connected_tx, td_connected_rx) = mpsc::channel(10);
            let (td_auth_tx, td_auth_rx) = mpsc::channel(10);
            let (td_login_tx, td_login_rx) = mpsc::channel(10);
            let (order_tx, order_rx) = mpsc::unbounded_channel();
            let (trade_tx, trade_rx) = mpsc::unbounded_channel();
            let (account_query_tx, account_query_rx) = mpsc::channel(10);
            let (position_query_tx, position_query_rx) = mpsc::channel(10);

            Self {
                config,
                md_api: None,
                td_api: None,
                market_data: Arc::new(RwLock::new(HashMap::new())),
                positions: Arc::new(RwLock::new(HashMap::new())),
                account: Arc::new(RwLock::new(None)),
                md_connected: Arc::new(RwLock::new(false)),
                td_connected: Arc::new(RwLock::new(false)),
                md_logged_in: Arc::new(RwLock::new(false)),
                td_logged_in: Arc::new(RwLock::new(false)),
                market_data_tx: market_data_tx.clone(),
                market_data_rx: Some(market_data_rx),
                md_connected_tx: Some(md_connected_tx),
                md_connected_rx: Some(md_connected_rx),
                md_login_tx: Some(md_login_tx),
                md_login_rx: Some(md_login_rx),
                md_subscribe_tx: Some(md_subscribe_tx),
                md_subscribe_rx: Some(md_subscribe_rx),
                td_connected_tx: Some(td_connected_tx),
                td_connected_rx: Some(td_connected_rx),
                td_auth_tx: Some(td_auth_tx),
                td_auth_rx: Some(td_auth_rx),
                td_login_tx: Some(td_login_tx),
                td_login_rx: Some(td_login_rx),
                order_tx: Some(order_tx),
                order_rx: Some(order_rx),
                trade_tx: Some(trade_tx),
                trade_rx: Some(trade_rx),
                account_query_tx: Some(account_query_tx),
                account_query_rx: Some(account_query_rx),
                position_query_tx: Some(position_query_tx),
                position_query_rx: Some(position_query_rx),
                request_id: Arc::new(AtomicI32::new(1)),
                last_query_time: Arc::new(Mutex::new(None)),
                // 重连机制字段
                md_reconnect_attempts: Arc::new(AtomicI32::new(0)),
                td_reconnect_attempts: Arc::new(AtomicI32::new(0)),
                max_reconnect_attempts: 5, // 默认最多重连5次
                is_md_reconnecting: Arc::new(RwLock::new(false)),
                is_td_reconnecting: Arc::new(RwLock::new(false)),
                subscribed_instruments: Arc::new(RwLock::new(Vec::new())),
            }
        }

        #[cfg(not(feature = "ctp-real"))]
        {
            Self {
                config,
                market_data: Arc::new(RwLock::new(HashMap::new())),
                positions: Arc::new(RwLock::new(HashMap::new())),
                account: Arc::new(RwLock::new(None)),
                md_connected: Arc::new(RwLock::new(false)),
                td_connected: Arc::new(RwLock::new(false)),
                md_logged_in: Arc::new(RwLock::new(false)),
                td_logged_in: Arc::new(RwLock::new(false)),
                market_data_tx,
                market_data_rx: Some(market_data_rx),
                // 重连机制字段
                md_reconnect_attempts: Arc::new(AtomicI32::new(0)),
                td_reconnect_attempts: Arc::new(AtomicI32::new(0)),
                max_reconnect_attempts: 5,
                is_md_reconnecting: Arc::new(RwLock::new(false)),
                is_td_reconnecting: Arc::new(RwLock::new(false)),
                subscribed_instruments: Arc::new(RwLock::new(Vec::new())),
            }
        }
    }

    #[cfg(feature = "ctp-real")]
    fn get_next_request_id(&self) -> i32 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 复制字符串到i8数组 (用于CTP API字段)
    #[cfg(feature = "ctp-real")]
    fn copy_str_to_i8_array(dest: &mut [i8], src: &str) {
        let bytes = src.as_bytes();
        let len = bytes.len().min(dest.len() - 1); // 保留空间给终止符

        // 复制字节并转换为i8
        for i in 0..len {
            dest[i] = bytes[i] as i8;
        }

        // 添加终止符
        if len < dest.len() {
            dest[len] = 0;
        }
    }

    /// 连接到CTP服务器
    #[cfg(feature = "ctp-real")]
    pub async fn connect(&mut self) -> Result<()> {
        use tokio::time::{timeout, Duration};
        use tracing::{info, warn};

        info!("🚀 Connecting to CTP server (Real Mode)...");
        info!("   MD Address: {}", self.config.md_address);
        info!("   TD Address: {}", self.config.td_address);

        // 1. 连接行情服务器
        self.connect_md().await?;

        // 2. 连接交易服务器
        self.connect_td().await?;

        info!("✅ CTP Real Mode: Connected successfully");
        Ok(())
    }

    /// 连接行情服务器
    #[cfg(feature = "ctp-real")]
    async fn connect_md(&mut self) -> Result<()> {
        use tokio::time::{timeout, Duration};
        use tracing::info;

        info!("📡 Connecting to MD server...");

        // 检查CTP SDK库文件
        #[cfg(target_os = "windows")]
        let dynlib_path = "thostmduserapi_se.dll";
        #[cfg(target_os = "linux")]
        let dynlib_path = "libthostmduserapi_se.so";
        #[cfg(target_os = "macos")]
        let dynlib_path = "libthostmduserapi_se.dylib";

        // 1. 创建SPI通道
        let md_connected_tx = self
            .md_connected_tx
            .take()
            .ok_or_else(|| anyhow!("MD connected channel already taken"))?;
        let md_login_tx = self
            .md_login_tx
            .take()
            .ok_or_else(|| anyhow!("MD login channel already taken"))?;
        let md_subscribe_tx = self
            .md_subscribe_tx
            .take()
            .ok_or_else(|| anyhow!("MD subscribe channel already taken"))?;

        let mut md_connected_rx = self
            .md_connected_rx
            .take()
            .ok_or_else(|| anyhow!("MD connected rx channel already taken"))?;
        let mut md_login_rx = self
            .md_login_rx
            .take()
            .ok_or_else(|| anyhow!("MD login rx channel already taken"))?;

        // 取出market_data_rx用于后台处理任务
        let md_data_rx = self
            .market_data_rx
            .take()
            .ok_or_else(|| anyhow!("Market data rx channel already taken"))?;

        // 2. 创建MdSpi
        let spi = Box::new(CtpMdSpi::new(
            md_connected_tx,
            md_login_tx,
            self.market_data_tx.clone(),
            md_subscribe_tx,
        ));

        // 3. 创建MdApi (尝试从系统库加载)
        let md_api = match MdApi::create_api(dynlib_path, "md_flow/", false, false) {
            api => api,
        };

        // 4. 注册SPI
        let spi_ptr = Box::into_raw(spi) as *mut dyn ctp2rs::v1alpha1::MdSpi;
        md_api.register_spi(spi_ptr);

        // 5. 注册前置地址
        md_api.register_front(&self.config.md_address);

        // 6. 初始化连接
        md_api.init();
        info!("   MdApi initialized, waiting for connection...");

        // 7. 等待前置连接成功 (超时30秒)
        match timeout(Duration::from_secs(30), md_connected_rx.recv()).await {
            Ok(Some(true)) => {
                *self.md_connected.write().await = true;
                info!("   ✅ MD front connected");
            }
            Ok(Some(false)) => {
                return Err(anyhow!("MD front disconnected unexpectedly"));
            }
            Ok(None) => {
                return Err(anyhow!("MD connected channel closed"));
            }
            Err(_) => {
                return Err(anyhow!("MD connection timeout (30s)"));
            }
        }

        // 8. 发送登录请求
        info!("   Logging in to MD server...");
        let mut login_req = CThostFtdcReqUserLoginField::default();
        Self::copy_str_to_i8_array(&mut login_req.BrokerID, &self.config.broker_id);
        Self::copy_str_to_i8_array(&mut login_req.UserID, &self.config.investor_id);
        Self::copy_str_to_i8_array(&mut login_req.Password, &self.config.password);

        let request_id = self.get_next_request_id();
        md_api.req_user_login(&mut login_req, request_id);

        // 9. 等待登录响应 (超时10秒)
        match timeout(Duration::from_secs(10), md_login_rx.recv()).await {
            Ok(Some(Ok(()))) => {
                *self.md_logged_in.write().await = true;
                info!("   ✅ MD login successful");
            }
            Ok(Some(Err(e))) => {
                return Err(anyhow!("MD login failed: {}", e));
            }
            Ok(None) => {
                return Err(anyhow!("MD login channel closed"));
            }
            Err(_) => {
                return Err(anyhow!("MD login timeout (10s)"));
            }
        }

        // 10. 保存API实例
        self.md_api = Some(Arc::new(md_api));

        // 11. 启动行情数据处理任务 (将rx所有权转移到后台任务)
        self.start_market_data_processor(md_data_rx);

        Ok(())
    }

    /// 启动行情数据处理任务
    #[cfg(feature = "ctp-real")]
    fn start_market_data_processor(&self, mut rx: mpsc::UnboundedReceiver<CtpMarketData>) {
        let market_data = self.market_data.clone();

        tokio::spawn(async move {
            use tracing::debug;

            while let Some(data) = rx.recv().await {
                debug!(
                    "📊 Processing market data: {} @ {}",
                    data.instrument_id, data.last_price
                );

                // 更新缓存
                market_data
                    .write()
                    .await
                    .insert(data.instrument_id.clone(), data);
            }
        });
    }

    /// 连接交易服务器
    #[cfg(feature = "ctp-real")]
    async fn connect_td(&mut self) -> Result<()> {
        use tokio::time::{timeout, Duration};
        use tracing::info;

        info!("📡 Connecting to TD server...");

        // 检查CTP SDK库文件
        #[cfg(target_os = "windows")]
        let dynlib_path = "thosttraderapi_se.dll";
        #[cfg(target_os = "linux")]
        let dynlib_path = "libthosttraderapi_se.so";
        #[cfg(target_os = "macos")]
        let dynlib_path = "libthosttraderapi_se.dylib";

        // 1. 创建SPI通道
        let td_connected_tx = self
            .td_connected_tx
            .take()
            .ok_or_else(|| anyhow!("TD connected channel already taken"))?;
        let td_auth_tx = self
            .td_auth_tx
            .take()
            .ok_or_else(|| anyhow!("TD auth channel already taken"))?;
        let td_login_tx = self
            .td_login_tx
            .take()
            .ok_or_else(|| anyhow!("TD login channel already taken"))?;
        let order_tx = self
            .order_tx
            .take()
            .ok_or_else(|| anyhow!("Order channel already taken"))?;
        let trade_tx = self
            .trade_tx
            .take()
            .ok_or_else(|| anyhow!("Trade channel already taken"))?;
        let account_query_tx = self
            .account_query_tx
            .take()
            .ok_or_else(|| anyhow!("Account query channel already taken"))?;
        let position_query_tx = self
            .position_query_tx
            .take()
            .ok_or_else(|| anyhow!("Position query channel already taken"))?;

        let mut td_connected_rx = self
            .td_connected_rx
            .take()
            .ok_or_else(|| anyhow!("TD connected rx channel already taken"))?;
        let mut td_auth_rx = self
            .td_auth_rx
            .take()
            .ok_or_else(|| anyhow!("TD auth rx channel already taken"))?;
        let mut td_login_rx = self
            .td_login_rx
            .take()
            .ok_or_else(|| anyhow!("TD login rx channel already taken"))?;

        // 取出 order_rx 和 trade_rx 用于后台处理任务
        let order_rx = self
            .order_rx
            .take()
            .ok_or_else(|| anyhow!("Order rx channel already taken"))?;
        let trade_rx = self
            .trade_rx
            .take()
            .ok_or_else(|| anyhow!("Trade rx channel already taken"))?;
        let account_query_rx = self
            .account_query_rx
            .take()
            .ok_or_else(|| anyhow!("Account query rx channel already taken"))?;
        let position_query_rx = self
            .position_query_rx
            .take()
            .ok_or_else(|| anyhow!("Position query rx channel already taken"))?;

        // 2. 创建TraderSpi
        let spi = Box::new(CtpTraderSpi::new(
            td_connected_tx,
            td_auth_tx,
            td_login_tx,
            order_tx,
            trade_tx,
            account_query_tx,
            position_query_tx,
        ));

        // 3. 创建TraderApi
        info!("   Loading CTP Trader API from: {}", dynlib_path);
        let mut td_api = TraderApi::create_api(dynlib_path, "td_flow/");

        // 4. 注册SPI
        let spi_ptr = Box::into_raw(spi) as *mut dyn ctp2rs::v1alpha1::TraderSpi;
        td_api.register_spi(spi_ptr);

        // 5. 注册前置地址
        info!("   Registering TD front: {}", self.config.td_address);
        td_api.register_front(&self.config.td_address);

        // 6. 初始化并等待连接
        info!("   Initializing TD connection...");
        td_api.init();

        match timeout(Duration::from_secs(30), td_connected_rx.recv()).await {
            Ok(Some(true)) => {
                *self.td_connected.write().await = true;
                info!("   ✅ TD connected");
            }
            Ok(Some(false)) => {
                return Err(anyhow!("TD connection lost"));
            }
            Ok(None) => {
                return Err(anyhow!("TD connection channel closed"));
            }
            Err(_) => {
                return Err(anyhow!("TD connection timeout (30s)"));
            }
        }

        // 7. 如果配置了认证信息,进行认证
        if !self.config.app_id.is_empty() && !self.config.auth_code.is_empty() {
            info!("   Authenticating...");
            let mut auth_req = CThostFtdcReqAuthenticateField::default();
            Self::copy_str_to_i8_array(&mut auth_req.BrokerID, &self.config.broker_id);
            Self::copy_str_to_i8_array(&mut auth_req.UserID, &self.config.investor_id);
            Self::copy_str_to_i8_array(&mut auth_req.AppID, &self.config.app_id);
            Self::copy_str_to_i8_array(&mut auth_req.AuthCode, &self.config.auth_code);
            Self::copy_str_to_i8_array(
                &mut auth_req.UserProductInfo,
                &self.config.user_product_info,
            );

            let request_id = self.get_next_request_id();
            td_api.req_authenticate(&mut auth_req, request_id);

            // 等待认证响应 (超时10秒)
            match timeout(Duration::from_secs(10), td_auth_rx.recv()).await {
                Ok(Some(Ok(()))) => {
                    info!("   ✅ TD authentication successful");
                }
                Ok(Some(Err(e))) => {
                    return Err(anyhow!("TD authentication failed: {}", e));
                }
                Ok(None) => {
                    return Err(anyhow!("TD auth channel closed"));
                }
                Err(_) => {
                    return Err(anyhow!("TD authentication timeout (10s)"));
                }
            }
        } else {
            info!("   Skipping authentication (no app_id/auth_code configured)");
        }

        // 8. 发送登录请求
        info!("   Logging in to TD server...");
        let mut login_req = CThostFtdcReqUserLoginField::default();
        Self::copy_str_to_i8_array(&mut login_req.BrokerID, &self.config.broker_id);
        Self::copy_str_to_i8_array(&mut login_req.UserID, &self.config.investor_id);
        Self::copy_str_to_i8_array(&mut login_req.Password, &self.config.password);
        Self::copy_str_to_i8_array(
            &mut login_req.UserProductInfo,
            &self.config.user_product_info,
        );

        let request_id = self.get_next_request_id();
        td_api.req_user_login(&mut login_req, request_id);

        // 9. 等待登录响应 (超时10秒)
        match timeout(Duration::from_secs(10), td_login_rx.recv()).await {
            Ok(Some(Ok(()))) => {
                *self.td_logged_in.write().await = true;
                info!("   ✅ TD login successful");
            }
            Ok(Some(Err(e))) => {
                return Err(anyhow!("TD login failed: {}", e));
            }
            Ok(None) => {
                return Err(anyhow!("TD login channel closed"));
            }
            Err(_) => {
                return Err(anyhow!("TD login timeout (10s)"));
            }
        }

        // 10. 保存API实例
        self.td_api = Some(Arc::new(td_api));

        // 11. 启动所有后台处理任务
        self.start_order_processor(order_rx);
        self.start_trade_processor(trade_rx);
        self.start_account_query_processor(account_query_rx);
        self.start_position_query_processor(position_query_rx);

        Ok(())
    }

    /// 启动订单回报处理任务
    #[cfg(feature = "ctp-real")]
    fn start_order_processor(&self, mut rx: mpsc::UnboundedReceiver<CtpOrderResponse>) {
        use tracing::{debug, info};

        info!("📋 Order processor started");

        tokio::spawn(async move {
            while let Some(order) = rx.recv().await {
                debug!(
                    "📋 Order update: {} - {} ({})",
                    order.order_ref, order.order_status, order.status_msg
                );

                // TODO: 可以在这里添加订单状态的持久化、通知等逻辑
                // 例如: 更新订单缓存、发送WebSocket通知等
            }
            debug!("Order processor terminated");
        });
    }

    /// 启动成交回报处理任务
    #[cfg(feature = "ctp-real")]
    fn start_trade_processor(&self, mut rx: mpsc::UnboundedReceiver<String>) {
        use tracing::{debug, info};

        info!("💰 Trade processor started");

        tokio::spawn(async move {
            while let Some(trade_id) = rx.recv().await {
                debug!("💰 Trade notification: {}", trade_id);

                // TODO: 可以在这里添加成交的处理逻辑
                // 例如: 更新持仓、计算盈亏、发送通知等
            }
            debug!("Trade processor terminated");
        });
    }

    /// 启动账户查询响应处理任务
    #[cfg(feature = "ctp-real")]
    fn start_account_query_processor(&self, mut rx: mpsc::Receiver<Result<CtpAccount, String>>) {
        use tracing::{debug, error, info};

        let account = self.account.clone();

        info!("💰 Account query processor started");

        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(acc) => {
                        debug!(
                            "💰 Account update: Balance={:.2}, Available={:.2}",
                            acc.balance, acc.available
                        );
                        *account.write().await = Some(acc);
                    }
                    Err(e) => {
                        error!("❌ Account query error: {}", e);
                    }
                }
            }
            debug!("Account query processor terminated");
        });
    }

    /// 启动持仓查询响应处理任务
    #[cfg(feature = "ctp-real")]
    fn start_position_query_processor(
        &self,
        mut rx: mpsc::Receiver<Result<Vec<CtpPosition>, String>>,
    ) {
        use tracing::{debug, error, info};

        let positions = self.positions.clone();

        info!("📊 Position query processor started");

        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                match result {
                    Ok(pos_list) => {
                        debug!("📊 Position update: {} positions", pos_list.len());

                        // 更新持仓缓存
                        let mut pos_map = positions.write().await;
                        pos_map.clear();
                        for pos in pos_list {
                            pos_map.insert(pos.instrument_id.clone(), pos);
                        }
                    }
                    Err(e) => {
                        error!("❌ Position query error: {}", e);
                    }
                }
            }
            debug!("Position query processor terminated");
        });
    }

    /// 连接到CTP服务器 (无ctp-real feature时的fallback)
    #[cfg(not(feature = "ctp-real"))]
    pub async fn connect(&mut self) -> Result<()> {
        Err(anyhow!(
            "CTP Real Mode is not enabled. Please compile with --features ctp-real\n\
             \n\
             Also ensure CTP SDK libraries are in your system PATH:\n\
             Windows: thostmduserapi_se.dll, thosttraderapi_se.dll\n\
             Linux: libthostmduserapi_se.so, libthosttraderapi_se.so"
        ))
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "ctp-real")]
        {
            if let Some(md_api) = &self.md_api {
                // md_api.release();
            }
            if let Some(td_api) = &self.td_api {
                // td_api.release();
            }

            *self.md_connected.write().await = false;
            *self.td_connected.write().await = false;
            *self.md_logged_in.write().await = false;
            *self.td_logged_in.write().await = false;
        }

        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        #[cfg(feature = "ctp-real")]
        {
            let md_conn = *self.md_connected.read().await;
            let td_conn = *self.td_connected.read().await;
            let md_login = *self.md_logged_in.read().await;
            let td_login = *self.td_logged_in.read().await;

            md_conn && td_conn && md_login && td_login
        }

        #[cfg(not(feature = "ctp-real"))]
        {
            false
        }
    }

    /// 订阅行情
    #[cfg(feature = "ctp-real")]
    pub async fn subscribe_market_data(&self, instruments: Vec<String>) -> Result<()> {
        use tracing::info;

        let md_api = self
            .md_api
            .as_ref()
            .ok_or_else(|| anyhow!("MD API not initialized"))?;

        info!("📊 Subscribing to {} instruments", instruments.len());

        // 保存订阅列表 (用于重连后恢复)
        {
            let mut subscribed = self.subscribed_instruments.write().await;
            for instrument in &instruments {
                if !subscribed.contains(instrument) {
                    subscribed.push(instrument.clone());
                }
            }
        }

        // 转换为C字符串数组
        let instrument_ids: Vec<String> = instruments.iter().map(|s| s.to_string()).collect();

        // 调用CTP订阅接口
        let ret = md_api.subscribe_market_data(&instrument_ids);

        if ret != 0 {
            return Err(anyhow!(
                "Failed to subscribe market data, error code: {}",
                ret
            ));
        }

        info!("   ✅ Subscription request sent for: {:?}", instrument_ids);
        Ok(())
    }

    /// 订阅行情 (无ctp-real feature)
    #[cfg(not(feature = "ctp-real"))]
    pub async fn subscribe_market_data(&self, _instruments: Vec<String>) -> Result<()> {
        Err(anyhow!("CTP Real Mode is not enabled"))
    }

    /// 下单
    #[cfg(feature = "ctp-real")]
    pub async fn place_order(&self, request: CtpOrderRequest) -> Result<CtpOrderResponse> {
        use tracing::info;

        let td_api = self
            .td_api
            .as_ref()
            .ok_or_else(|| anyhow!("TD API not initialized"))?;

        info!(
            "📝 Placing order: {} {} x{} @ {}",
            request.instrument_id,
            if request.direction == '0' {
                "Buy"
            } else {
                "Sell"
            },
            request.volume,
            request.price
        );

        // 1. 构造报单请求
        let mut order_field = CThostFtdcInputOrderField::default();

        // 基本信息
        Self::copy_str_to_i8_array(&mut order_field.BrokerID, &self.config.broker_id);
        Self::copy_str_to_i8_array(&mut order_field.InvestorID, &self.config.investor_id);
        Self::copy_str_to_i8_array(&mut order_field.InstrumentID, &request.instrument_id);

        // 生成报单引用
        let order_ref = self.get_next_request_id().to_string();
        Self::copy_str_to_i8_array(&mut order_field.OrderRef, &order_ref);

        // 订单参数
        order_field.Direction = request.direction as i8;
        order_field.CombOffsetFlag[0] = request.offset_flag as i8;
        order_field.CombHedgeFlag[0] = request.hedge_flag as i8;
        order_field.LimitPrice = request.price;
        order_field.VolumeTotalOriginal = request.volume;

        // 订单价格类型
        order_field.OrderPriceType = request.price_type as i8;

        // 有效期类型: 当日有效
        order_field.TimeCondition = b'3' as i8; // THOST_FTDC_TC_GFD (当日有效)

        // 成交量类型: 任何数量
        order_field.VolumeCondition = b'1' as i8; // THOST_FTDC_VC_AV (任何数量)

        // 最小成交量
        order_field.MinVolume = 1;

        // 触发条件: 立即
        order_field.ContingentCondition = b'1' as i8; // THOST_FTDC_CC_Immediately

        // 强平原因: 非强平
        order_field.ForceCloseReason = b'0' as i8; // THOST_FTDC_FCC_NotForceClose

        // 自动挂起标志: 否
        order_field.IsAutoSuspend = 0;

        // 用户强评标志: 否
        order_field.UserForceClose = 0;

        // 2. 发送报单请求
        let request_id = self.get_next_request_id();
        let ret = td_api.req_order_insert(&mut order_field, request_id);

        if ret != 0 {
            return Err(anyhow!("Failed to submit order, error code: {}", ret));
        }

        info!("✅ Order submitted successfully, OrderRef: {}", order_ref);

        // 3. 返回订单响应 (简化版本,实际订单状态通过回调获得)
        Ok(CtpOrderResponse {
            order_sys_id: String::new(), // 系统编号在回报中获得
            order_ref: order_ref.clone(),
            instrument_id: request.instrument_id,
            order_status: super::types::CtpOrderStatus::Unknown,
            status_msg: "Order submitted".to_string(),
        })
    }

    /// 下单 (无ctp-real feature)
    #[cfg(not(feature = "ctp-real"))]
    pub async fn place_order(&self, _request: CtpOrderRequest) -> Result<CtpOrderResponse> {
        Err(anyhow!("CTP Real Mode is not enabled"))
    }

    /// 查询账户
    #[cfg(feature = "ctp-real")]
    pub async fn query_account(&self) -> Result<CtpAccount> {
        use tokio::time::sleep;
        use tracing::info;

        let td_api = self
            .td_api
            .as_ref()
            .ok_or_else(|| anyhow!("TD API not initialized"))?;

        // 1. 检查流控 (CTP限制每秒1次查询)
        {
            let mut last_time = self.last_query_time.lock().await;
            if let Some(last) = *last_time {
                let elapsed = last.elapsed();
                if elapsed < Duration::from_secs(1) {
                    let wait_time = Duration::from_secs(1) - elapsed;
                    info!("⏱️  Query throttle: waiting {:?}", wait_time);
                    sleep(wait_time).await;
                }
            }
            *last_time = Some(Instant::now());
        }

        // 2. 构造查询请求
        let mut qry_req = CThostFtdcQryTradingAccountField::default();
        Self::copy_str_to_i8_array(&mut qry_req.BrokerID, &self.config.broker_id);
        Self::copy_str_to_i8_array(&mut qry_req.InvestorID, &self.config.investor_id);

        // 3. 发送查询请求
        let request_id = self.get_next_request_id();
        let ret = td_api.req_qry_trading_account(&mut qry_req, request_id);

        if ret != 0 {
            return Err(anyhow!("Failed to query account, error code: {}", ret));
        }

        info!("💰 Querying account...");

        // 4. 等待响应 (通过SPI回调会自动更新到 self.account)
        // 简单等待一段时间让响应返回
        sleep(Duration::from_millis(500)).await;

        // 5. 从缓存读取
        let account = self.account.read().await;
        account
            .clone()
            .ok_or_else(|| anyhow!("Account not available after query"))
    }

    /// 查询账户 (无ctp-real feature)
    #[cfg(not(feature = "ctp-real"))]
    pub async fn query_account(&self) -> Result<CtpAccount> {
        let account = self.account.read().await;
        account
            .clone()
            .ok_or_else(|| anyhow!("Account not available"))
    }

    /// 查询持仓
    #[cfg(feature = "ctp-real")]
    pub async fn query_position(&self) -> Result<Vec<CtpPosition>> {
        use tokio::time::sleep;
        use tracing::info;

        let td_api = self
            .td_api
            .as_ref()
            .ok_or_else(|| anyhow!("TD API not initialized"))?;

        // 1. 检查流控 (CTP限制每秒1次查询)
        {
            let mut last_time = self.last_query_time.lock().await;
            if let Some(last) = *last_time {
                let elapsed = last.elapsed();
                if elapsed < Duration::from_secs(1) {
                    let wait_time = Duration::from_secs(1) - elapsed;
                    info!("⏱️  Query throttle: waiting {:?}", wait_time);
                    sleep(wait_time).await;
                }
            }
            *last_time = Some(Instant::now());
        }

        // 2. 构造查询请求
        let mut qry_req = CThostFtdcQryInvestorPositionField::default();
        Self::copy_str_to_i8_array(&mut qry_req.BrokerID, &self.config.broker_id);
        Self::copy_str_to_i8_array(&mut qry_req.InvestorID, &self.config.investor_id);
        // InstrumentID留空表示查询所有持仓

        // 3. 发送查询请求
        let request_id = self.get_next_request_id();
        let ret = td_api.req_qry_investor_position(&mut qry_req, request_id);

        if ret != 0 {
            return Err(anyhow!("Failed to query position, error code: {}", ret));
        }

        info!("📊 Querying positions...");

        // 4. 等待响应 (通过SPI回调会自动更新到 self.positions)
        // 简单等待一段时间让响应返回
        sleep(Duration::from_millis(500)).await;

        // 5. 从缓存读取
        let positions = self.positions.read().await;
        let result: Vec<CtpPosition> = positions.values().cloned().collect();

        info!("✅ Position query result: {} positions", result.len());
        Ok(result)
    }

    /// 查询持仓 (无ctp-real feature)
    #[cfg(not(feature = "ctp-real"))]
    pub async fn query_position(&self) -> Result<Vec<CtpPosition>> {
        let positions = self.positions.read().await;
        Ok(positions.values().cloned().collect())
    }

    /// 获取市场数据
    pub async fn get_market_data(&self, instrument_id: &str) -> Result<CtpMarketData> {
        let data = self.market_data.read().await;
        data.get(instrument_id)
            .cloned()
            .ok_or_else(|| anyhow!("Market data not found for {}", instrument_id))
    }
}

// 实现Drop trait以确保资源释放
impl Drop for RealCtpConnection {
    fn drop(&mut self) {
        // 清理资源
        #[cfg(feature = "ctp-real")]
        {
            // CTP API的release会在Arc drop时自动调用
        }
    }
}

// ==================== 重连机制相关方法 ====================
impl RealCtpConnection {
    /// 计算指数退避延迟时间
    ///
    /// # 参数
    /// * `attempt` - 当前重连尝试次数 (从0开始)
    ///
    /// # 返回
    /// 延迟时间 (秒),使用指数退避: 2^attempt 秒,最大60秒
    fn calculate_backoff_delay(attempt: i32) -> u64 {
        let delay = 2u64.pow(attempt as u32);
        delay.min(60) // 最大60秒
    }

    /// 处理行情连接断线
    ///
    /// 此方法会在行情连接断开时被调用,启动自动重连流程
    #[cfg(feature = "ctp-real")]
    async fn handle_md_disconnection(&mut self) {
        tracing::warn!("📡 行情连接断开,启动重连流程...");

        // 设置重连状态
        *self.is_md_reconnecting.write().await = true;
        *self.md_connected.write().await = false;
        *self.md_logged_in.write().await = false;

        // 重置重连计数
        self.md_reconnect_attempts.store(0, Ordering::SeqCst);

        // 启动重连任务
        let connection = self.clone_for_reconnect();
        tokio::spawn(async move {
            connection.reconnect_md_loop().await;
        });
    }

    /// 处理交易连接断线
    ///
    /// 此方法会在交易连接断开时被调用,启动自动重连流程
    #[cfg(feature = "ctp-real")]
    async fn handle_td_disconnection(&mut self) {
        tracing::warn!("📡 交易连接断开,启动重连流程...");

        // 设置重连状态
        *self.is_td_reconnecting.write().await = true;
        *self.td_connected.write().await = false;
        *self.td_logged_in.write().await = false;

        // 重置重连计数
        self.td_reconnect_attempts.store(0, Ordering::SeqCst);

        // 启动重连任务
        let connection = self.clone_for_reconnect();
        tokio::spawn(async move {
            connection.reconnect_td_loop().await;
        });
    }

    /// 行情重连循环
    ///
    /// 使用指数退避策略不断尝试重连,直到成功或达到最大重连次数
    #[cfg(feature = "ctp-real")]
    async fn reconnect_md_loop(mut self) {
        loop {
            let attempt = self.md_reconnect_attempts.load(Ordering::SeqCst);

            if attempt >= self.max_reconnect_attempts {
                tracing::error!(
                    "❌ 行情重连失败: 已达到最大重连次数 {}",
                    self.max_reconnect_attempts
                );
                *self.is_md_reconnecting.write().await = false;
                break;
            }

            // 计算退避延迟
            let delay = Self::calculate_backoff_delay(attempt);
            tracing::info!(
                "🔄 行情重连尝试 {}/{}, 等待 {} 秒...",
                attempt + 1,
                self.max_reconnect_attempts,
                delay
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;

            // 尝试重连
            match self.connect_md().await {
                Ok(_) => {
                    tracing::info!("✅ 行情重连成功!");

                    // 恢复订阅
                    let instruments = self.subscribed_instruments.read().await.clone();
                    if !instruments.is_empty() {
                        tracing::info!("📊 恢复订阅 {} 个合约...", instruments.len());
                        if let Err(e) = self.subscribe_market_data(instruments).await {
                            tracing::error!("⚠️ 恢复订阅失败: {}", e);
                        }
                    }

                    // 重置状态
                    self.md_reconnect_attempts.store(0, Ordering::SeqCst);
                    *self.is_md_reconnecting.write().await = false;
                    break;
                }
                Err(e) => {
                    tracing::error!("❌ 行情重连失败: {}", e);
                    self.md_reconnect_attempts.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// 交易重连循环
    ///
    /// 使用指数退避策略不断尝试重连,直到成功或达到最大重连次数
    #[cfg(feature = "ctp-real")]
    async fn reconnect_td_loop(mut self) {
        loop {
            let attempt = self.td_reconnect_attempts.load(Ordering::SeqCst);

            if attempt >= self.max_reconnect_attempts {
                tracing::error!(
                    "❌ 交易重连失败: 已达到最大重连次数 {}",
                    self.max_reconnect_attempts
                );
                *self.is_td_reconnecting.write().await = false;
                break;
            }

            // 计算退避延迟
            let delay = Self::calculate_backoff_delay(attempt);
            tracing::info!(
                "🔄 交易重连尝试 {}/{}, 等待 {} 秒...",
                attempt + 1,
                self.max_reconnect_attempts,
                delay
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;

            // 尝试重连
            match self.connect_td().await {
                Ok(_) => {
                    tracing::info!("✅ 交易重连成功!");

                    // 重置状态
                    self.td_reconnect_attempts.store(0, Ordering::SeqCst);
                    *self.is_td_reconnecting.write().await = false;
                    break;
                }
                Err(e) => {
                    tracing::error!("❌ 交易重连失败: {}", e);
                    self.td_reconnect_attempts.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// 克隆必要字段用于重连
    ///
    /// 由于重连需要在后台任务中运行,我们需要克隆一份连接对象
    #[cfg(feature = "ctp-real")]
    fn clone_for_reconnect(&self) -> Self {
        Self {
            config: self.config.clone(),
            md_api: self.md_api.clone(),
            td_api: self.td_api.clone(),
            market_data: self.market_data.clone(),
            positions: self.positions.clone(),
            account: self.account.clone(),
            md_connected: self.md_connected.clone(),
            td_connected: self.td_connected.clone(),
            md_logged_in: self.md_logged_in.clone(),
            td_logged_in: self.td_logged_in.clone(),
            market_data_tx: self.market_data_tx.clone(),
            market_data_rx: None, // 不克隆 rx
            md_connected_tx: self.md_connected_tx.clone(),
            md_connected_rx: None,
            md_login_tx: self.md_login_tx.clone(),
            md_login_rx: None,
            md_subscribe_tx: self.md_subscribe_tx.clone(),
            md_subscribe_rx: None,
            td_connected_tx: self.td_connected_tx.clone(),
            td_connected_rx: None,
            td_auth_tx: self.td_auth_tx.clone(),
            td_auth_rx: None,
            td_login_tx: self.td_login_tx.clone(),
            td_login_rx: None,
            order_tx: self.order_tx.clone(),
            order_rx: None,
            trade_tx: self.trade_tx.clone(),
            trade_rx: None,
            account_query_tx: self.account_query_tx.clone(),
            account_query_rx: None,
            position_query_tx: self.position_query_tx.clone(),
            position_query_rx: None,
            request_id: self.request_id.clone(),
            last_query_time: self.last_query_time.clone(),
            md_reconnect_attempts: self.md_reconnect_attempts.clone(),
            td_reconnect_attempts: self.td_reconnect_attempts.clone(),
            max_reconnect_attempts: self.max_reconnect_attempts,
            is_md_reconnecting: self.is_md_reconnecting.clone(),
            is_td_reconnecting: self.is_td_reconnecting.clone(),
            subscribed_instruments: self.subscribed_instruments.clone(),
        }
    }

    /// 设置最大重连次数
    ///
    /// # 参数
    /// * `max_attempts` - 最大重连次数,默认为5
    pub fn set_max_reconnect_attempts(&mut self, max_attempts: i32) {
        self.max_reconnect_attempts = max_attempts;
        tracing::info!("🔧 设置最大重连次数: {}", max_attempts);
    }

    /// 获取当前重连状态
    ///
    /// # 返回
    /// (行情是否在重连, 交易是否在重连)
    pub async fn get_reconnect_status(&self) -> (bool, bool) {
        let md_reconnecting = *self.is_md_reconnecting.read().await;
        let td_reconnecting = *self.is_td_reconnecting.read().await;
        (md_reconnecting, td_reconnecting)
    }

    /// 获取重连尝试次数
    ///
    /// # 返回
    /// (行情重连次数, 交易重连次数)
    pub fn get_reconnect_attempts(&self) -> (i32, i32) {
        let md_attempts = self.md_reconnect_attempts.load(Ordering::SeqCst);
        let td_attempts = self.td_reconnect_attempts.load(Ordering::SeqCst);
        (md_attempts, td_attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> CtpConfig {
        CtpConfig {
            broker_id: "9999".to_string(),
            investor_id: "test".to_string(),
            password: "test".to_string(),
            md_address: "tcp://180.168.146.187:10131".to_string(),
            td_address: "tcp://180.168.146.187:10130".to_string(),
            app_id: String::new(),
            auth_code: String::new(),
            user_product_info: "nof0".to_string(),
            mock_mode: false,
        }
    }

    #[tokio::test]
    async fn test_real_connection_without_feature() {
        let config = create_test_config();
        let mut conn = RealCtpConnection::new(config);

        // 不启用feature时应该返回错误
        #[cfg(not(feature = "ctp-real"))]
        {
            let result = conn.connect().await;
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("not enabled"));
            }
        }

        // 启用feature时测试连接(需要CTP SDK)
        #[cfg(feature = "ctp-real")]
        {
            // 注意: 这个测试需要真实的CTP SDK库文件
            // 如果没有库文件会失败,这是预期行为
            let result = conn.connect().await;
            // 可能成功或失败(取决于是否有CTP SDK)
            // 这里只测试不会panic
            match result {
                Ok(_) => println!("CTP connected successfully"),
                Err(e) => println!("CTP connection failed (expected without SDK): {}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_without_feature() {
        let config = create_test_config();
        let conn = RealCtpConnection::new(config);

        #[cfg(not(feature = "ctp-real"))]
        {
            let result = conn.subscribe_market_data(vec!["IF2501".to_string()]).await;
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("not enabled"));
            }
        }
    }

    #[tokio::test]
    async fn test_place_order_without_feature() {
        let config = create_test_config();
        let conn = RealCtpConnection::new(config);

        let order_request = CtpOrderRequest {
            instrument_id: "IF2501".to_string(),
            direction: '0',   // 买
            offset_flag: '0', // 开仓
            price: 3500.0,
            volume: 1,
            price_type: '2', // 限价
            hedge_flag: '1', // 投机
        };

        #[cfg(not(feature = "ctp-real"))]
        {
            let result = conn.place_order(order_request).await;
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("not enabled"));
            }
        }
    }

    #[tokio::test]
    async fn test_query_operations() {
        let config = create_test_config();
        let conn = RealCtpConnection::new(config);

        // 测试查询账户(未初始化时应该返回错误)
        let account_result = conn.query_account().await;
        assert!(account_result.is_err());

        // 测试查询持仓(空持仓)
        let positions_result = conn.query_position().await;
        assert!(positions_result.is_ok());
        assert!(positions_result.unwrap().is_empty());

        // 测试获取市场数据(未订阅时应该返回错误)
        let market_data_result = conn.get_market_data("IF2501").await;
        assert!(market_data_result.is_err());
    }

    #[tokio::test]
    async fn test_is_connected() {
        let config = create_test_config();
        let conn = RealCtpConnection::new(config);

        // 未连接时应该返回false
        let connected = conn.is_connected().await;
        assert!(!connected);
    }
}
