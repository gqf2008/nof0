# CTP 配置说明

## ✅ 已验证的配置

### OpenCTP 公开测试环境

**连接状态**: ✅ 成功（2025-11-04 测试通过）

```yaml
broker_id: "9999"
md_address: "tcp://trading.openctp.cn:30011"  # 行情服务器
td_address: "tcp://trading.openctp.cn:30011"  # 交易服务器
app_id: "simnow_client_test"
auth_code: "0000000000000000"
```

**测试结果**:
- ✅ MD 服务器连接成功
- ✅ TD 服务器连接成功
- ✅ AppID 认证通过
- ✅ 登录成功
- ⚠️ 行情订阅需要 SimNow 有效账号

---

## 🔧 使用方法

### 方式 1: Mock 模式（推荐开发/演示）

```rust
let config = CtpConfig {
    broker_id: "9999".to_string(),
    investor_id: "test".to_string(),
    password: "test".to_string(),
    md_address: "tcp://trading.openctp.cn:30011".to_string(),
    td_address: "tcp://trading.openctp.cn:30011".to_string(),
    app_id: "simnow_client_test".to_string(),
    auth_code: "0000000000000000".to_string(),
    user_product_info: "nof0".to_string(),
    mock_mode: true,  // ✅ 使用 Mock 数据
};
```

**优势**:
- ✅ 零依赖，无需外部服务
- ✅ 稳定可靠
- ✅ 性能优秀（11µs 平均响应）
- ✅ 适合开发、测试、演示

### 方式 2: 真实连接（需要 SimNow 账号）

```rust
let config = CtpConfig {
    broker_id: "9999".to_string(),
    investor_id: "你的SimNow账号".to_string(),  // ⚠️ 需要注册
    password: "你的SimNow密码".to_string(),      // ⚠️ 需要注册
    md_address: "tcp://180.168.146.187:10131".to_string(),  // SimNow 7x24
    td_address: "tcp://180.168.146.187:10130".to_string(),
    app_id: "simnow_client_test".to_string(),
    auth_code: "0000000000000000".to_string(),
    user_product_info: "nof0".to_string(),
    mock_mode: false,  // ⚠️ 需要有效账号
};
```

**注册 SimNow**:
1. 访问: http://www.simnow.com.cn/
2. 注册模拟账号（免费）
3. 获取账号和密码
4. 查看可用合约列表

---

## 📡 环境对比

| 环境 | 地址 | 用途 | 行情 | 交易 | 账号要求 |
|------|------|------|------|------|----------|
| **OpenCTP 测试** | tcp://trading.openctp.cn:30011 | 连接测试 | ⚠️ | ⚠️ | AppID 即可 |
| **SimNow 7x24** | tcp://180.168.146.187:10131 | 模拟交易 | ✅ | ✅ | 需要注册 |
| **SimNow 白盘** | tcp://180.168.146.187:10211 | 模拟交易 | ✅ | ✅ | 需要注册 |
| **实盘** | 各期货公司提供 | 真实交易 | ✅ | ✅ | 实盘账号 |

---

## 🔌 DLL 文件

**位置**: `backend/lib/`

**文件**:
- `thostmduserapi_se.dll` - 行情 API
- `thosttraderapi_se.dll` - 交易 API

**部署**:
```powershell
# 自动复制到运行目录
Copy-Item "backend/lib/*.dll" "target/debug/examples/"
Copy-Item "backend/lib/*.dll" "target/release/examples/"
```

---

## 🧪 测试命令

```bash
# Mock 模式测试（推荐）
cargo test --features ctp-real
cargo run --example ctp_mock_test

# 真实连接测试（需要 SimNow 账号）
cargo run --features ctp-real --example ctp_market_test

# 订阅测试
cargo run --features ctp-real --example ctp_subscribe_test
```

---

## ⚙️ 配置切换

### 在 exchanges.yaml 中切换

```yaml
config:
  mock_mode: true   # Mock 模式（默认）
  # mock_mode: false  # 真实模式（需要 SimNow 账号）
```

### 在代码中切换

```rust
// 开发/演示：Mock 模式
let broker = CtpBroker::new(id, name, config_with_mock_true);

// 生产/测试：真实模式
let broker = CtpBroker::new(id, name, config_with_mock_false);
broker.connect().await?;  // 连接 CTP 服务器
broker.subscribe_market_data(symbols).await?;  // 订阅行情
```

---

## 📝 注意事项

1. **OpenCTP 公开测试环境**
   - ✅ 可用于连接测试
   - ⚠️ 行情订阅需要有效合约
   - ⚠️ 可能不提供完整行情数据

2. **SimNow 模拟环境**
   - ✅ 完整的模拟交易环境
   - ✅ 真实的行情数据
   - ✅ 免费注册使用

3. **实盘环境**
   - ⚠️ 需要开通期货账户
   - ⚠️ 涉及真实资金
   - ⚠️ 需要期货公司提供的 CTP 地址

---

## 🎯 推荐配置

### 开发阶段
```
mock_mode: true
适合: 功能开发、单元测试、演示
```

### 测试阶段
```
mock_mode: false
SimNow 账号
适合: 集成测试、行情验证
```

### 生产阶段
```
mock_mode: false
实盘账号
适合: 真实交易
```

---

## 📚 相关文档

- [CTP_CONNECTION_SUCCESS_REPORT.md](../examples/CTP_CONNECTION_SUCCESS_REPORT.md) - 连接测试报告
- [CTP_TEST_README.md](../examples/CTP_TEST_README.md) - 测试说明
- [CTP_INTEGRATION_STATUS.md](../examples/CTP_INTEGRATION_STATUS.md) - 集成状态

---

**最后更新**: 2025-11-04  
**测试状态**: ✅ OpenCTP 连接成功 | ✅ Mock 模式完美运行
