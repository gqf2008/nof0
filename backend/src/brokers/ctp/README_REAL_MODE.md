# CTP Real Mode - 真实连接配置指南

本文档说明如何启用CTP真实模式,连接到实际的CTP服务器(SimNow或期货公司前置)。

---

## 📋 前置要求

### 1. CTP SDK动态库

CTP Real模式需要上海期货信息技术有限公司提供的CTP SDK动态库。

#### Windows系统

需要以下DLL文件:
- `thostmduserapi_se.dll` (行情API)
- `thosttraderapi_se.dll` (交易API)

放置位置:
```
方式1: 放在系统PATH中
  C:\Windows\System32\

方式2: 放在项目目录
  nof0\backend\
```

#### Linux系统

需要以下SO文件:
- `libthostmduserapi_se.so` (行情API)
- `libthosttraderapi_se.so` (交易API)

放置位置:
```bash
方式1: 系统库目录
  /usr/lib/
  /usr/local/lib/

方式2: 添加到LD_LIBRARY_PATH
  export LD_LIBRARY_PATH=/path/to/ctp/lib:$LD_LIBRARY_PATH
```

### 2. 获取CTP SDK

**官方下载**:
- CTP官网: http://www.sfit.com.cn/
- SimNow下载: http://www.simnow.com.cn/

**版本要求**:
- 推荐: v6.6.9 或更新版本
- 支持平台: Windows x64, Linux x64

### 3. SimNow账号 (测试环境)

如果使用SimNow测试环境,需要先注册账号。

**注册地址**: http://www.simnow.com.cn/

**SimNow环境信息**:

| 环境 | 行情地址 | 交易地址 |
|------|---------|---------|
| 电信 | tcp://180.168.146.187:10131 | tcp://180.168.146.187:10130 |
| 移动 | tcp://218.202.237.33:10131 | tcp://218.202.237.33:10130 |

**Broker ID**: 9999

---

## 🔧 启用Real模式

### Step 1: 编译时启用feature

```bash
# 编译时启用 ctp-real feature
cd backend
cargo build --features ctp-real

# 或者运行示例
cargo run --example ctp_market_demo --features ctp-real
```

### Step 2: 配置文件设置

编辑 `backend/etc/ctp_config.yaml`:

```yaml
# 期货公司代码
broker_id: "9999"  # SimNow使用9999

# 你的账号
investor_id: "YOUR_SIMNOW_ACCOUNT"

# 密码 (建议使用环境变量)
password: "${CTP_PASSWORD}"

# 前置地址
md_address: "tcp://180.168.146.187:10131"
td_address: "tcp://180.168.146.187:10130"

# ⚠️ 关闭Mock模式,启用真实连接
mock_mode: false  # ← 改为 false
```

### Step 3: 设置环境变量 (推荐)

**Windows (PowerShell)**:
```powershell
# 设置密码
$env:CTP_PASSWORD = "your_password"

# 验证
echo $env:CTP_PASSWORD
```

**Linux/Mac (Bash)**:
```bash
# 设置密码
export CTP_PASSWORD="your_password"

# 验证
echo $CTP_PASSWORD
```

### Step 4: 运行程序

```bash
# 运行演示程序 (启用Real模式)
cargo run --example ctp_market_demo --features ctp-real

# 或者编译你的交易程序
cargo build --release --features ctp-real
```

---

## 📝 代码使用

### 基础用法

```rust
use nof0_backend::markets::ctp::{CtpConfig, RealCtpConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = CtpConfig::from_file("etc/ctp_config.yaml")?;
    
    // 确保mock_mode = false
    assert!(!config.mock_mode, "Must disable mock_mode for real connection");
    
    // 2. 创建真实连接
    let mut conn = RealCtpConnection::new(config);
    
    // 3. 连接到CTP
    conn.connect().await?;
    
    // 4. 订阅行情
    conn.subscribe_market_data(vec![
        "IF2501".to_string(),
        "IC2501".to_string(),
    ]).await?;
    
    // 5. 获取行情
    let data = conn.get_market_data("IF2501").await?;
    println!("IF2501: {:.2}", data.last_price);
    
    // 6. 查询账户
    let account = conn.query_account().await?;
    println!("Balance: {:.2}", account.balance);
    
    // 7. 断开连接
    conn.disconnect().await?;
    
    Ok(())
}
```

### 在Adapter中使用

`CtpMarketAdapter` 会根据 `mock_mode` 自动选择Mock或Real模式:

```rust
use nof0_backend::markets::{CtpConfig, CtpMarketAdapter};

// 加载配置 (mock_mode = false)
let config = CtpConfig::from_file("etc/ctp_config.yaml")?;

// 创建Adapter (自动选择Real模式)
let adapter = CtpMarketAdapter::new(config);

// 连接 (内部使用RealCtpConnection)
adapter.connect().await?;
```

---

## ⚠️ 常见问题

### Q1: "CTP Real Mode is not enabled"

**原因**: 编译时没有启用 `ctp-real` feature。

**解决**:
```bash
# 必须使用 --features ctp-real
cargo build --features ctp-real
```

### Q2: "找不到 thostmduserapi_se.dll"

**原因**: CTP SDK动态库不在系统PATH中。

**解决方案1**: 复制到系统目录
```powershell
# Windows
Copy-Item thostmduserapi_se.dll C:\Windows\System32\
Copy-Item thosttraderapi_se.dll C:\Windows\System32\
```

**解决方案2**: 复制到项目目录
```powershell
# 复制到backend目录
Copy-Item thostmduserapi_se.dll D:\my\Documents\GitHub\nof0\backend\
Copy-Item thosttraderapi_se.dll D:\my\Documents\GitHub\nof0\backend\
```

### Q3: "Connection timeout"

**可能原因**:
1. 网络无法连接到CTP前置地址
2. 前置地址配置错误
3. 防火墙拦截

**排查步骤**:
```bash
# 1. 测试网络连通性 (Windows)
Test-NetConnection 180.168.146.187 -Port 10131

# 2. 检查配置文件
cat backend/etc/ctp_config.yaml

# 3. 临时关闭防火墙测试
```

### Q4: "Login failed: 认证失败"

**可能原因**:
- Broker ID错误 (SimNow应该是"9999")
- 账号或密码错误
- 账号未激活或已过期

**解决**:
1. 检查SimNow账号状态
2. 重置密码
3. 确认Broker ID是"9999"

### Q5: Mock模式 vs Real模式如何选择?

| 场景 | 推荐模式 | 原因 |
|------|---------|------|
| **开发调试** | Mock | 无需网络,快速迭代 |
| **单元测试** | Mock | 可预测的行为 |
| **集成测试** | Real (SimNow) | 测试真实流程 |
| **模拟盘** | Real (SimNow) | 真实行情数据 |
| **实盘交易** | Real (期货公司) | 真实交易 |

---

## 🔒 安全注意事项

### 1. 密码管理

**❌ 不要**:
```yaml
# 不要在配置文件中明文存储密码!
password: "my_password_123"
```

**✅ 应该**:
```yaml
# 使用环境变量
password: "${CTP_PASSWORD}"
```

### 2. 配置文件保护

```bash
# 添加到 .gitignore
echo "backend/etc/ctp_config.yaml" >> .gitignore

# 或者使用模板文件
cp ctp_config.example.yaml ctp_config.yaml
git add ctp_config.example.yaml  # 只提交模板
```

### 3. 生产环境

**实盘交易前必须**:
1. ✅ 在SimNow充分测试
2. ✅ 启用风控系统
3. ✅ 设置止损止盈
4. ✅ 小资金试跑
5. ✅ 监控告警

---

## 📊 性能对比

| 指标 | Mock模式 | Real模式 |
|------|---------|---------|
| **启动时间** | <1ms | 2-5秒 |
| **行情延迟** | 0ms | 50-200ms |
| **订单响应** | <1ms | 100-500ms |
| **数据准确性** | 模拟 | 真实 |
| **网络依赖** | 无 | 有 |

---

## 🚀 下一步

### 完成Real模式TODO

当前Real模式是**框架代码**,以下功能待实现:

1. **连接回调处理**
   - [ ] OnFrontConnected
   - [ ] OnFrontDisconnected
   - [ ] OnHeartBeatWarning

2. **登录流程**
   - [ ] ReqUserLogin
   - [ ] OnRspUserLogin
   - [ ] 错误处理

3. **行情订阅**
   - [ ] SubscribeMarketData
   - [ ] OnRtnDepthMarketData
   - [ ] 行情缓存更新

4. **交易功能**
   - [ ] ReqOrderInsert
   - [ ] OnRtnOrder
   - [ ] OnRtnTrade
   - [ ] 持仓自动更新

5. **查询功能**
   - [ ] ReqQryTradingAccount
   - [ ] ReqQryInvestorPosition
   - [ ] 查询节流控制

### 贡献代码

欢迎贡献Real模式的实现! 请参考:
- `src/markets/ctp/real_connection.rs` - 核心连接实现
- CTP API文档: http://www.sfit.com.cn/

---

## 📚 参考资源

### CTP官方

- **CTP开发者网站**: http://www.sfit.com.cn/
- **SimNow**: http://www.simnow.com.cn/
- **API文档**: CTP SDK中的文档

### Rust库

- **ctp2rs**: https://github.com/pseudocodes/ctp2rs
- **crates.io**: https://crates.io/crates/ctp2rs
- **async-trait**: https://docs.rs/async-trait/
- **tokio**: https://tokio.rs/

### 项目文档

- [CTP_ADAPTER.md](../../../markdown/CTP_ADAPTER.md) - 完整技术文档
- [CTP_ADAPTER_QUICKSTART.md](../../../markdown/CTP_ADAPTER_QUICKSTART.md) - 快速入门

---

**⚠️ 重要提示**: 当前Real模式是**框架代码**,核心功能(连接、行情、交易)尚未完全实现。建议先使用Mock模式进行开发和测试,Real模式正在积极开发中。

**贡献者**: 欢迎提交PR完善Real模式实现! 🚀
