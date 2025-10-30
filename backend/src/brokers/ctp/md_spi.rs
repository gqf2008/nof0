// CTP 行情SPI回调实现
// Market Data SPI Callbacks Implementation

#[cfg(feature = "ctp-real")]
use ctp2rs::v1alpha1::{
    CThostFtdcDepthMarketDataField, CThostFtdcRspInfoField, CThostFtdcRspUserLoginField,
    CThostFtdcSpecificInstrumentField, MdSpi,
};

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::types::CtpMarketData;

/// 行情SPI回调处理器
///
/// 此结构体实现了CTP的MdSpi trait,用于接收CTP行情API的各种回调通知。
/// 通过channel机制将C++回调事件传递到Rust异步代码中。
#[cfg(feature = "ctp-real")]
pub struct CtpMdSpi {
    /// 连接状态通道 - 用于通知前置连接成功/断开
    connected_tx: mpsc::Sender<bool>,

    /// 登录结果通道 - 用于通知登录成功/失败
    login_tx: mpsc::Sender<Result<(), String>>,

    /// 行情数据通道 - 用于推送实时行情数据
    market_data_tx: mpsc::UnboundedSender<CtpMarketData>,

    /// 订阅结果通道 - 用于通知订阅成功/失败
    subscribe_tx: mpsc::Sender<Result<String, String>>,
}

#[cfg(feature = "ctp-real")]
impl CtpMdSpi {
    /// 创建新的行情SPI处理器
    pub fn new(
        connected_tx: mpsc::Sender<bool>,
        login_tx: mpsc::Sender<Result<(), String>>,
        market_data_tx: mpsc::UnboundedSender<CtpMarketData>,
        subscribe_tx: mpsc::Sender<Result<String, String>>,
    ) -> Self {
        Self {
            connected_tx,
            login_tx,
            market_data_tx,
            subscribe_tx,
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
        // 查找第一个零字节
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let slice = &bytes[..end];

        // 尝试转换为UTF-8
        String::from_utf8_lossy(slice).to_string()
    }

    /// 转换CTP行情数据到内部格式
    fn convert_market_data(
        ctp_data: &CThostFtdcDepthMarketDataField,
    ) -> Result<CtpMarketData, String> {
        let instrument_id = Self::convert_gb2312_to_utf8(&ctp_data.InstrumentID);
        let update_time = Self::convert_gb2312_to_utf8(&ctp_data.UpdateTime);
        let trading_day = Self::convert_gb2312_to_utf8(&ctp_data.TradingDay);

        Ok(CtpMarketData {
            instrument_id,
            last_price: ctp_data.LastPrice,
            bid_price: ctp_data.BidPrice1,
            bid_volume: ctp_data.BidVolume1,
            ask_price: ctp_data.AskPrice1,
            ask_volume: ctp_data.AskVolume1,
            volume: ctp_data.Volume,
            open_interest: ctp_data.OpenInterest as i32,
            highest_price: ctp_data.HighestPrice,
            lowest_price: ctp_data.LowestPrice,
            update_time,
        })
    }
}

#[cfg(feature = "ctp-real")]
impl MdSpi for CtpMdSpi {
    /// 当客户端与交易后台建立起通信连接时,该方法被调用
    fn on_front_connected(&mut self) {
        info!("📡 CTP MD: Front connected");

        // 通知连接成功
        if let Err(e) = self.connected_tx.try_send(true) {
            error!("Failed to send connection status: {}", e);
        }
    }

    /// 当客户端与交易后台通信连接断开时,该方法被调用
    ///
    /// # 错误原因
    /// - 0x1001: 网络读失败
    /// - 0x1002: 网络写失败
    /// - 0x2001: 接收心跳超时
    /// - 0x2002: 发送心跳失败
    /// - 0x2003: 收到错误报文
    fn on_front_disconnected(&mut self, n_reason: i32) {
        warn!(
            "⚠️  CTP MD: Front disconnected, reason code: 0x{:X}",
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

        // 通知连接断开
        if let Err(e) = self.connected_tx.try_send(false) {
            error!("Failed to send disconnection status: {}", e);
        }
    }

    /// 心跳超时警告
    ///
    /// 当长时间未收到报文时,该方法被调用
    fn on_heart_beat_warning(&mut self, n_time_lapse: i32) {
        warn!(
            "💓 CTP MD: Heartbeat warning, time lapse: {} seconds",
            n_time_lapse
        );
    }

    /// 登录请求响应
    fn on_rsp_user_login(
        &mut self,
        p_rsp_user_login: Option<&CThostFtdcRspUserLoginField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        debug!("📥 CTP MD: Login response received");

        // 检查错误
        match Self::check_rsp_info(p_rsp_info) {
            Ok(_) => {
                if let Some(login_info) = p_rsp_user_login {
                    let trading_day = Self::convert_gb2312_to_utf8(&login_info.TradingDay);
                    let login_time = Self::convert_gb2312_to_utf8(&login_info.LoginTime);

                    info!("✅ CTP MD: Login successful");
                    info!("   Trading Day: {}", trading_day);
                    info!("   Login Time: {}", login_time);
                    info!(
                        "   Front ID: {}, Session ID: {}",
                        login_info.FrontID, login_info.SessionID
                    );
                }

                // 通知登录成功
                if let Err(e) = self.login_tx.try_send(Ok(())) {
                    error!("Failed to send login success: {}", e);
                }
            }
            Err(e) => {
                error!("❌ CTP MD: Login failed: {}", e);

                // 通知登录失败
                if let Err(send_err) = self.login_tx.try_send(Err(e)) {
                    error!("Failed to send login error: {}", send_err);
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
            error!("CTP MD: Logout failed: {}", e);
        } else {
            info!("CTP MD: Logout successful");
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
            error!("❌ CTP MD Error {}: {}", info.ErrorID, error_msg);
        }
    }

    /// 订阅行情应答
    fn on_rsp_sub_market_data(
        &mut self,
        p_specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        let instrument_id = p_specific_instrument
            .map(|inst| Self::convert_gb2312_to_utf8(&inst.InstrumentID))
            .unwrap_or_else(|| "Unknown".to_string());

        match Self::check_rsp_info(p_rsp_info) {
            Ok(_) => {
                info!("✅ CTP MD: Subscribed to {}", instrument_id);

                // 通知订阅成功
                if let Err(e) = self.subscribe_tx.try_send(Ok(instrument_id)) {
                    error!("Failed to send subscribe success: {}", e);
                }
            }
            Err(e) => {
                error!("❌ CTP MD: Subscribe to {} failed: {}", instrument_id, e);

                // 通知订阅失败
                if let Err(send_err) = self
                    .subscribe_tx
                    .try_send(Err(format!("{}: {}", instrument_id, e)))
                {
                    error!("Failed to send subscribe error: {}", send_err);
                }
            }
        }
    }

    /// 取消订阅行情应答
    fn on_rsp_unsub_market_data(
        &mut self,
        p_specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        p_rsp_info: Option<&CThostFtdcRspInfoField>,
        _n_request_id: i32,
        _b_is_last: bool,
    ) {
        let instrument_id = p_specific_instrument
            .map(|inst| Self::convert_gb2312_to_utf8(&inst.InstrumentID))
            .unwrap_or_else(|| "Unknown".to_string());

        if let Err(e) = Self::check_rsp_info(p_rsp_info) {
            error!("CTP MD: Unsubscribe {} failed: {}", instrument_id, e);
        } else {
            info!("CTP MD: Unsubscribed from {}", instrument_id);
        }
    }

    /// 深度行情通知
    ///
    /// 这是最重要的回调,接收实时行情数据推送
    fn on_rtn_depth_market_data(
        &mut self,
        p_depth_market_data: Option<&CThostFtdcDepthMarketDataField>,
    ) {
        if let Some(ctp_data) = p_depth_market_data {
            match Self::convert_market_data(ctp_data) {
                Ok(market_data) => {
                    debug!(
                        "📊 CTP MD: Market data - {} @ {} ({})",
                        market_data.instrument_id, market_data.last_price, market_data.update_time
                    );

                    // 发送行情数据到通道
                    if let Err(e) = self.market_data_tx.send(market_data) {
                        error!("Failed to send market data: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to convert market data: {}", e);
                }
            }
        }
    }
}

// 当不启用ctp-real feature时,提供空实现
#[cfg(not(feature = "ctp-real"))]
pub struct CtpMdSpi;

#[cfg(not(feature = "ctp-real"))]
impl CtpMdSpi {
    pub fn new() -> Self {
        Self
    }
}
