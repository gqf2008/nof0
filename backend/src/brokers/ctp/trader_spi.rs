// CTP 交易SPI回调实现
// Trader SPI Callbacks Implementation

#[cfg(feature = "ctp-real")]
use ctp2rs::v1alpha1::{
    CThostFtdcInputOrderField, CThostFtdcInvestorPositionField, CThostFtdcOrderField,
    CThostFtdcRspAuthenticateField, CThostFtdcRspInfoField, CThostFtdcRspUserLoginField,
    CThostFtdcTradeField, CThostFtdcTradingAccountField, TraderSpi,
};

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::types::{CtpAccount, CtpOrderResponse, CtpOrderStatus, CtpPosition};

/// 交易SPI回调处理器
///
/// 此结构体实现了CTP的TraderSpi trait,用于接收CTP交易API的各种回调通知。
/// 通过channel机制将C++回调事件传递到Rust异步代码中。
#[cfg(feature = "ctp-real")]
pub struct CtpTraderSpi {
    /// 连接状态通道 - 用于通知前置连接成功/断开
    connected_tx: mpsc::Sender<bool>,

    /// 认证结果通道 - 用于通知认证成功/失败
    auth_tx: mpsc::Sender<Result<(), String>>,

    /// 登录结果通道 - 用于通知登录成功/失败
    login_tx: mpsc::Sender<Result<(), String>>,

    /// 订单回报通道 - 用于推送订单状态变化
    order_tx: mpsc::UnboundedSender<CtpOrderResponse>,

    /// 成交回报通道 - 用于推送成交信息
    trade_tx: mpsc::UnboundedSender<String>,

    /// 账户查询响应通道
    account_tx: mpsc::Sender<Result<CtpAccount, String>>,

    /// 持仓查询响应通道
    position_tx: mpsc::Sender<Result<Vec<CtpPosition>, String>>,
}

#[cfg(feature = "ctp-real")]
impl CtpTraderSpi {
    /// 创建新的交易SPI处理器
    pub fn new(
        connected_tx: mpsc::Sender<bool>,
        auth_tx: mpsc::Sender<Result<(), String>>,
        login_tx: mpsc::Sender<Result<(), String>>,
        order_tx: mpsc::UnboundedSender<CtpOrderResponse>,
        trade_tx: mpsc::UnboundedSender<String>,
        account_tx: mpsc::Sender<Result<CtpAccount, String>>,
        position_tx: mpsc::Sender<Result<Vec<CtpPosition>, String>>,
    ) -> Self {
        Self {
            connected_tx,
            auth_tx,
            login_tx,
            order_tx,
            trade_tx,
            account_tx,
            position_tx,
        }
    }

    /// 检查并解析响应信息
    fn check_rsp_info(rsp_info: Option<&CThostFtdcRspInfoField>) -> Result<(), String> {
        if let Some(info) = rsp_info {
            if info.ErrorID != 0 {
                let error_msg = Self::convert_gb2312_to_utf8(&info.ErrorMsg);
                return Err(format!("CTP Error {}: {}", info.ErrorID, error_msg));
            }
        }
        Ok(())
    }

    /// 转换GB2312编码到UTF-8
    fn convert_gb2312_to_utf8(data: &[i8]) -> String {
        // 将i8转换为u8
        let bytes: Vec<u8> = data.iter().map(|&b| b as u8).collect();
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let slice = &bytes[..end];
        String::from_utf8_lossy(slice).to_string()
    }

    /// 转换CTP账户数据到内部格式
    fn convert_account(ctp_account: &CThostFtdcTradingAccountField) -> CtpAccount {
        let account_id = Self::convert_gb2312_to_utf8(&ctp_account.AccountID);

        CtpAccount {
            account_id,
            available: ctp_account.Available,
            margin: ctp_account.CurrMargin,
            frozen_margin: ctp_account.FrozenMargin,
            close_profit: ctp_account.CloseProfit,
            position_profit: ctp_account.PositionProfit,
            commission: ctp_account.Commission,
            pre_balance: ctp_account.PreBalance,
            balance: ctp_account.Balance,
        }
    }

    /// 转换CTP持仓数据到内部格式
    fn convert_position(ctp_position: &CThostFtdcInvestorPositionField) -> CtpPosition {
        let instrument_id = Self::convert_gb2312_to_utf8(&ctp_position.InstrumentID);

        CtpPosition {
            instrument_id,
            direction: if ctp_position.PosiDirection == 2 {
                '2' // 多头
            } else {
                '3' // 空头
            },
            position: ctp_position.Position,
            today_position: ctp_position.TodayPosition,
            available: ctp_position.Position - ctp_position.TodayPosition, // 简化:总持仓 - 今仓
            open_cost: ctp_position.OpenCost,
            position_profit: ctp_position.PositionProfit,
        }
    }

    /// 转换CTP订单数据到内部格式
    fn convert_order(ctp_order: &CThostFtdcOrderField) -> CtpOrderResponse {
        let order_ref = Self::convert_gb2312_to_utf8(&ctp_order.OrderRef);
        let order_sys_id = Self::convert_gb2312_to_utf8(&ctp_order.OrderSysID);
        let instrument_id = Self::convert_gb2312_to_utf8(&ctp_order.InstrumentID);
        let status_msg = Self::convert_gb2312_to_utf8(&ctp_order.StatusMsg);

        // 将CTP订单状态转换为内部枚举
        let order_status = match ctp_order.OrderStatus as u8 {
            b'0' => CtpOrderStatus::AllTraded,  // 全部成交
            b'1' => CtpOrderStatus::PartTraded, // 部分成交还在队列中
            b'3' => CtpOrderStatus::NoTraded,   // 未成交还在队列中
            b'5' => CtpOrderStatus::Canceled,   // 撤单
            _ => CtpOrderStatus::Unknown,
        };

        CtpOrderResponse {
            order_sys_id,
            order_ref,
            instrument_id,
            order_status,
            status_msg,
        }
    }
}

#[cfg(feature = "ctp-real")]
impl TraderSpi for CtpTraderSpi {
    /// 当客户端与交易后台建立起通信连接时,该方法被调用
    fn on_front_connected(&mut self) {
        info!("📡 CTP TD: Front connected");

        if let Err(e) = self.connected_tx.try_send(true) {
            error!("Failed to send TD connection status: {}", e);
        }
    }

    /// 当客户端与交易后台通信连接断开时,该方法被调用
    fn on_front_disconnected(&mut self, n_reason: i32) {
        warn!(
            "⚠️  CTP TD: Front disconnected, reason code: 0x{:X}",
            n_reason
        );

        let reason_str = match n_reason {
            0x1001 => "网络读失败",
            0x1002 => "网络写失败",
            0x2001 => "接收心跳超时",
            0x2002 => "发送心跳失败",
            0x2003 => "收到错误报文",
            _ => "未知原因",
        };

        warn!("   Reason: {}", reason_str);

        if let Err(e) = self.connected_tx.try_send(false) {
            error!("Failed to send TD disconnection status: {}", e);
        }
    }

    /// 心跳超时警告
    fn on_heart_beat_warning(&mut self, n_time_lapse: i32) {
        warn!(
            "💓 CTP TD: Heartbeat warning, time lapse: {} seconds",
            n_time_lapse
        );
    }

    /// 客户端认证响应
    fn on_rsp_authenticate(
        &mut self,
        _p_rsp_authenticate_field: Option<&CThostFtdcRspAuthenticateField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        debug!("📥 CTP TD: Authenticate response received");

        match Self::check_rsp_info(p_rsp_info) {
            Ok(_) => {
                info!("✅ CTP TD: Authentication successful");
                if let Err(e) = self.auth_tx.try_send(Ok(())) {
                    error!("Failed to send auth success: {}", e);
                }
            }
            Err(e) => {
                error!("❌ CTP TD: Authentication failed: {}", e);
                if let Err(send_err) = self.auth_tx.try_send(Err(e)) {
                    error!("Failed to send auth error: {}", send_err);
                }
            }
        }
    }

    /// 登录请求响应
    fn on_rsp_user_login(
        &mut self,
        p_rsp_user_login: Option<&CThostFtdcRspUserLoginField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        debug!("📥 CTP TD: Login response received");

        match Self::check_rsp_info(p_rsp_info) {
            Ok(_) => {
                if let Some(login_info) = p_rsp_user_login {
                    let trading_day = Self::convert_gb2312_to_utf8(&login_info.TradingDay);
                    let login_time = Self::convert_gb2312_to_utf8(&login_info.LoginTime);

                    info!("✅ CTP TD: Login successful");
                    info!("   Trading Day: {}", trading_day);
                    info!("   Login Time: {}", login_time);
                    info!(
                        "   Front ID: {}, Session ID: {}",
                        login_info.FrontID, login_info.SessionID
                    );

                    let max_order_ref = Self::convert_gb2312_to_utf8(&login_info.MaxOrderRef);
                    info!("   Max Order Ref: {}", max_order_ref);
                }

                if let Err(e) = self.login_tx.try_send(Ok(())) {
                    error!("Failed to send TD login success: {}", e);
                }
            }
            Err(e) => {
                error!("❌ CTP TD: Login failed: {}", e);
                if let Err(send_err) = self.login_tx.try_send(Err(e)) {
                    error!("Failed to send TD login error: {}", send_err);
                }
            }
        }
    }

    /// 登出请求响应
    fn on_rsp_user_logout(
        &mut self,
        _p_user_logout: Option<&ctp2rs::v1alpha1::CThostFtdcUserLogoutField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        if let Err(e) = Self::check_rsp_info(p_rsp_info) {
            error!("CTP TD: Logout failed: {}", e);
        } else {
            info!("CTP TD: Logout successful");
        }
    }

    /// 报单录入请求响应
    fn on_rsp_order_insert(
        &mut self,
        p_input_order: Option<&CThostFtdcInputOrderField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        if let Some(order) = p_input_order {
            let order_ref = Self::convert_gb2312_to_utf8(&order.OrderRef);
            let instrument_id = Self::convert_gb2312_to_utf8(&order.InstrumentID);

            match Self::check_rsp_info(p_rsp_info) {
                Ok(_) => {
                    debug!("✅ CTP TD: Order insert accepted - {}", order_ref);
                }
                Err(e) => {
                    error!(
                        "❌ CTP TD: Order insert rejected - {} ({}): {}",
                        order_ref, instrument_id, e
                    );

                    // 发送错误订单响应
                    let error_response = CtpOrderResponse {
                        order_sys_id: String::new(),
                        order_ref,
                        instrument_id,
                        order_status: CtpOrderStatus::Error,
                        status_msg: e,
                    };

                    if let Err(send_err) = self.order_tx.send(error_response) {
                        error!("Failed to send error order response: {}", send_err);
                    }
                }
            }
        }
    }

    /// 报单通知 (订单状态变化)
    fn on_rtn_order(&mut self, p_order: Option<&CThostFtdcOrderField>) {
        if let Some(order) = p_order {
            let order_response = Self::convert_order(order);
            let order_ref = order_response.order_ref.clone();
            let status = &order_response.order_status;

            debug!("📋 CTP TD: Order update - {} status: {}", order_ref, status);

            // 发送订单更新
            if let Err(e) = self.order_tx.send(order_response) {
                error!("Failed to send order update: {}", e);
            }
        }
    }

    /// 成交通知
    fn on_rtn_trade(&mut self, p_trade: Option<&CThostFtdcTradeField>) {
        if let Some(trade) = p_trade {
            let trade_id = Self::convert_gb2312_to_utf8(&trade.TradeID);
            let order_ref = Self::convert_gb2312_to_utf8(&trade.OrderRef);
            let instrument_id = Self::convert_gb2312_to_utf8(&trade.InstrumentID);

            info!(
                "💰 CTP TD: Trade - {} {} {} @ {} x{}",
                trade_id,
                instrument_id,
                if trade.Direction as u8 == b'0' {
                    "Buy"
                } else {
                    "Sell"
                },
                trade.Price,
                trade.Volume
            );

            // 发送成交通知
            if let Err(e) = self.trade_tx.send(trade_id) {
                error!("Failed to send trade notification: {}", e);
            }
        }
    }

    /// 错误应答
    fn on_rsp_error(
        &mut self,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        if let Some(info) = p_rsp_info {
            let error_msg = Self::convert_gb2312_to_utf8(&info.ErrorMsg);
            error!("❌ CTP TD Error {}: {}", info.ErrorID, error_msg);
        }
    }

    /// 请求查询资金账户响应
    fn on_rsp_qry_trading_account(
        &mut self,
        p_trading_account: Option<&CThostFtdcTradingAccountField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        b_is_last: bool,
    ) {
        if !b_is_last {
            return; // 等待最后一条
        }

        match Self::check_rsp_info(p_rsp_info) {
            Ok(_) => {
                if let Some(account) = p_trading_account {
                    let ctp_account = Self::convert_account(account);
                    debug!(
                        "💰 CTP TD: Account query - Balance: {:.2}, Available: {:.2}",
                        ctp_account.balance, ctp_account.available
                    );

                    if let Err(e) = self.account_tx.try_send(Ok(ctp_account)) {
                        error!("Failed to send account data: {}", e);
                    }
                } else {
                    if let Err(e) = self
                        .account_tx
                        .try_send(Err("No account data returned".to_string()))
                    {
                        error!("Failed to send account error: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ CTP TD: Account query failed: {}", e);
                if let Err(send_err) = self.account_tx.try_send(Err(e)) {
                    error!("Failed to send account query error: {}", send_err);
                }
            }
        }
    }

    /// 请求查询投资者持仓响应
    fn on_rsp_qry_investor_position(
        &mut self,
        p_investor_position: Option<&CThostFtdcInvestorPositionField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        b_is_last: bool,
    ) {
        // 注意: CTP可能返回多条持仓记录,需要累积
        static mut POSITIONS: Option<Vec<CtpPosition>> = None;

        unsafe {
            if POSITIONS.is_none() {
                POSITIONS = Some(Vec::new());
            }

            if let Some(position) = p_investor_position {
                let ctp_position = Self::convert_position(position);
                debug!(
                    "📊 CTP TD: Position - {} {} position: {}",
                    ctp_position.instrument_id,
                    if ctp_position.direction == '2' {
                        "Long"
                    } else {
                        "Short"
                    },
                    ctp_position.position
                );

                if let Some(ref mut positions) = POSITIONS {
                    positions.push(ctp_position);
                }
            }

            if b_is_last {
                match Self::check_rsp_info(p_rsp_info) {
                    Ok(_) => {
                        let positions = POSITIONS.take().unwrap_or_default();
                        info!(
                            "📊 CTP TD: Position query completed - {} records",
                            positions.len()
                        );

                        if let Err(e) = self.position_tx.try_send(Ok(positions)) {
                            error!("Failed to send position data: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("❌ CTP TD: Position query failed: {}", e);
                        POSITIONS = None; // 清空累积的数据

                        if let Err(send_err) = self.position_tx.try_send(Err(e)) {
                            error!("Failed to send position query error: {}", send_err);
                        }
                    }
                }
            }
        }
    }
}

// 当不启用ctp-real feature时,提供空实现
#[cfg(not(feature = "ctp-real"))]
pub struct CtpTraderSpi;

#[cfg(not(feature = "ctp-real"))]
impl CtpTraderSpi {
    pub fn new() -> Self {
        Self
    }
}
