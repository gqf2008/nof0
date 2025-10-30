// CTP错误码映射表
// 根据CTP API文档整理的常见错误码及其中文描述

use std::collections::HashMap;
use std::sync::OnceLock;

/// CTP错误码映射表 (全局单例)
static ERROR_CODE_MAP: OnceLock<HashMap<i32, &'static str>> = OnceLock::new();

/// 获取错误码映射表
fn get_error_code_map() -> &'static HashMap<i32, &'static str> {
    ERROR_CODE_MAP.get_or_init(|| {
        let mut map = HashMap::new();

        // 连接相关错误
        map.insert(0, "正常");
        map.insert(-1, "CTP_未连接");
        map.insert(-2, "CTP_尚未完成初始化");
        map.insert(-3, "CTP_输入队列已满");
        map.insert(-4, "CTP_超过流控限制");

        // 登录认证相关
        map.insert(3, "会员不存在");
        map.insert(4, "会员编号不存在");
        map.insert(6, "投资者不存在");
        map.insert(7, "交易编码不存在");
        map.insert(8, "密码错误");
        map.insert(16, "客户端认证失败");
        map.insert(17, "客户端认证超时");
        map.insert(18, "客户端认证信息不正确");
        map.insert(19, "客户端未绑定认证");
        map.insert(26, "未登录");
        map.insert(63, "没有可用的交易席位");
        map.insert(90, "重复的登录请求");

        // 报单相关错误
        map.insert(22, "报单不存在");
        map.insert(31, "报单已提交");
        map.insert(36, "撤单已提交");
        map.insert(37, "已撤单");
        map.insert(40, "超过最大报单数");
        map.insert(41, "重复的报单引用");
        map.insert(42, "报单价格不合理");
        map.insert(43, "报单数量不合理");
        map.insert(44, "报单价格超出涨跌停板");
        map.insert(51, "资金不足");
        map.insert(52, "仓位不足");
        map.insert(53, "超出持仓限制");
        map.insert(54, "非法的报单类型");
        map.insert(55, "非法的组合单报单");
        map.insert(56, "非法的投机套保标志");
        map.insert(57, "非法的开平标志");
        map.insert(58, "非法的买卖方向");

        // 账户相关
        map.insert(61, "投资者账户已锁定");
        map.insert(62, "投资者账户已销户");
        map.insert(64, "账户保证金不足");
        map.insert(65, "账户权益为负");

        // 合约相关
        map.insert(70, "合约不存在");
        map.insert(71, "合约停牌");
        map.insert(72, "合约已到期");
        map.insert(73, "非交易时间");
        map.insert(74, "不支持该合约交易");

        // 交易所相关
        map.insert(80, "交易所系统异常");
        map.insert(81, "交易所网络异常");
        map.insert(82, "交易所席位异常");

        // 查询相关
        map.insert(91, "查询频率过快");
        map.insert(92, "查询结果不存在");

        // 网络通信相关
        map.insert(100, "网络连接失败");
        map.insert(101, "网络读写失败");
        map.insert(102, "接收心跳超时");
        map.insert(103, "发送心跳超时");

        map
    })
}

/// 将CTP错误码转换为人类可读的错误消息
///
/// # 参数
/// * `error_code` - CTP API返回的错误码
/// * `error_msg` - CTP API返回的原始错误消息 (可选)
///
/// # 返回
/// 格式化的错误消息字符串
pub fn format_ctp_error(error_code: i32, error_msg: Option<&str>) -> String {
    let map = get_error_code_map();

    let code_desc = map.get(&error_code).copied().unwrap_or("未知错误");

    if let Some(msg) = error_msg {
        if msg.is_empty() {
            format!("CTP错误 [{}]: {}", error_code, code_desc)
        } else {
            format!("CTP错误 [{}]: {} - {}", error_code, code_desc, msg)
        }
    } else {
        format!("CTP错误 [{}]: {}", error_code, code_desc)
    }
}

/// 判断错误码是否为网络连接相关错误
pub fn is_network_error(error_code: i32) -> bool {
    matches!(error_code, -1 | 100 | 101 | 102 | 103)
}

/// 判断错误码是否为认证相关错误
pub fn is_auth_error(error_code: i32) -> bool {
    matches!(error_code, 3 | 4 | 6 | 7 | 8 | 16 | 17 | 18 | 19)
}

/// 判断错误码是否为流控相关错误
pub fn is_flow_control_error(error_code: i32) -> bool {
    matches!(error_code, -3 | -4 | 91)
}

/// 判断错误码是否需要重连
pub fn should_reconnect(error_code: i32) -> bool {
    is_network_error(error_code) || matches!(error_code, 80 | 81 | 82)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error() {
        let msg = format_ctp_error(0, None);
        assert!(msg.contains("正常"));

        let msg = format_ctp_error(8, Some("密码验证失败"));
        assert!(msg.contains("密码错误"));
        assert!(msg.contains("密码验证失败"));
    }

    #[test]
    fn test_error_classification() {
        assert!(is_network_error(-1));
        assert!(is_network_error(100));
        assert!(!is_network_error(8));

        assert!(is_auth_error(8));
        assert!(is_auth_error(16));
        assert!(!is_auth_error(100));

        assert!(is_flow_control_error(-4));
        assert!(is_flow_control_error(91));
        assert!(!is_flow_control_error(8));

        assert!(should_reconnect(-1));
        assert!(should_reconnect(100));
        assert!(should_reconnect(80));
        assert!(!should_reconnect(8));
    }
}
