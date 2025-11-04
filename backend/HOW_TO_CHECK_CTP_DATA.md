# 如何检查 CTP 数据接收状态

## 🎯 快速检查方法

### 方法 1: 使用状态检查工具（推荐）

```bash
# 检查 Mock 模式数据
cargo run --features ctp-real --example ctp_status

# 输出示例：
✅ 状态: 优秀
   所有测试通过，数据接收正常
   响应时间: 303.2µs
```

### 方法 2: 查看实时日志

```bash
# 启动程序并查看详细日志
RUST_LOG=debug cargo run --features ctp-real

# 关键日志标识：
📊 CTP MD: Market data - IF2501 @ 4500.00 (16:26:43)  # ✅ 接收到数据
⚠️ 真实 CTP 获取失败，回退到 mock                      # ⚠️ 使用 Mock 数据
```

### 方法 3: 监控工具持续观察

```bash
# Mock 模式监控
cargo run --features ctp-real --example ctp_monitor --mock

# Real 模式监控（需要连接）
cargo run --features ctp-real --example ctp_monitor

# 输出示例：
📈 统计 #1 (过去 5 秒)
   数据接收: 12 条
   累计接收: 24 条
   ✅ 正在接收数据 (12 条/5秒)
```

---

## 📊 判断数据来源

### Mock 模式数据特征

```
✅ Mock 模式运行标识：
   - 日志显示: "🎭 使用 Mock 模式"
   - 响应速度: < 1ms（微秒级）
   - 数据稳定: 每次获取都成功
   - 无需连接: 不需要 CTP 服务器
```

### Real 模式数据特征

```
✅ Real 模式运行标识：
   - 日志显示: "🔌 初始化真实 CTP 连接..."
   - 连接日志: "📡 CTP MD: Front connected"
   - 登录日志: "✅ CTP MD: Login successful"
   - 订阅日志: "✅ CTP MD: Subscribed to IF2501"
   - 行情回调: "📊 CTP MD: Market data - IF2501 @ 4500.00"
```

---

## 🔍 详细检查步骤

### 步骤 1: 检查程序运行模式

**查看配置文件**:
```yaml
# backend/config/exchanges.yaml
config:
  mock_mode: true   # ✅ Mock 模式
  # mock_mode: false  # 🔌 Real 模式
```

**查看启动日志**:
```bash
# Mock 模式
2025-11-04T08:36:36Z INFO 🎭 使用 Mock 模式

# Real 模式
2025-11-04T08:36:36Z INFO 🔌 初始化真实 CTP 连接...
2025-11-04T08:36:36Z INFO 📡 Connecting to MD server...
```

### 步骤 2: 检查连接状态（Real 模式）

**成功连接的标志**:
```
✅ 必须看到以下日志：
   📡 CTP MD: Front connected          # MD 服务器连接
   ✅ CTP MD: Login successful         # MD 登录成功
   📡 CTP TD: Front connected          # TD 服务器连接
   ✅ CTP TD: Authentication successful # TD 认证成功
   ✅ CTP TD: Login successful         # TD 登录成功
```

**连接失败的标志**:
```
❌ 可能看到以下错误：
   LoadLibraryExW error               # DLL 文件缺失
   Connection timeout                 # 网络连接超时
   Authentication failed              # 认证失败
```

### 步骤 3: 检查订阅状态（Real 模式）

**成功订阅的标志**:
```
✅ 订阅成功：
   📊 Subscribing to 3 instruments
   ✅ Subscription request sent for: ["IF2501", "IC2501", "rb2505"]
   ✅ CTP MD: Subscribed to IF2501
```

**订阅失败的标志**:
```
❌ 订阅失败：
   ❌ CTP MD: Subscribe to IF2501 failed: CTP Error 1
   查询合约失败，没有此合约
```

### 步骤 4: 检查数据接收

**数据正常接收的标志**:
```bash
# 运行状态检查
cargo run --features ctp-real --example ctp_status

# 看到以下输出表示正常：
✅ 数据获取成功
   合约: IF2501
   最新价: 4500.00
   响应时间: 303.2µs
   ✅ 响应速度: 优秀
```

**数据接收异常的标志**:
```
❌ 数据获取失败: Market data not found
⚠️ 未接收到新数据
```

---

## 📋 常见问题排查

### Q1: 如何确认使用的是 Mock 还是 Real 模式？

**答**: 查看启动日志第一行
```bash
🎭 使用 Mock 模式          # Mock 模式
🔌 初始化真实 CTP 连接...  # Real 模式
```

### Q2: Mock 模式下会收到真实数据吗？

**答**: 不会。Mock 模式使用内置的模拟数据：
- ✅ 数据格式完全真实
- ✅ 价格波动合理
- ❌ 不是市场真实行情
- ✅ 适合开发和测试

### Q3: Real 模式连接成功但没有数据？

**可能原因**:
1. **未订阅合约** - 需要调用 `subscribe_market_data()`
2. **合约代码错误** - OpenCTP 可能不支持该合约
3. **需要 SimNow 账号** - 行情订阅需要有效账号

**解决方法**:
```bash
# 1. 检查订阅日志
grep "Subscribed" log.txt

# 2. 注册 SimNow 账号
访问: http://www.simnow.com.cn/

# 3. 使用 Mock 模式继续开发
mock_mode: true
```

### Q4: 如何查看实时行情推送？

**方法**:
```bash
# 设置 DEBUG 级别日志
RUST_LOG=debug cargo run --features ctp-real

# 查找行情推送日志
grep "Market data" log.txt

# 输出示例：
📊 CTP MD: Market data - IF2501 @ 4500.00 (16:26:43)
```

### Q5: 响应时间多少算正常？

**参考标准**:
```
Mock 模式:
   < 1ms (微秒级)    ✅ 优秀
   1-10ms            ✅ 正常
   > 10ms            ⚠️ 异常

Real 模式:
   < 100ms           ✅ 优秀
   100-500ms         ✅ 良好
   > 500ms           ⚠️ 较慢
```

---

## 🛠️ 可用工具清单

| 工具 | 命令 | 用途 |
|------|------|------|
| **状态检查** | `cargo run --example ctp_status` | 快速检查数据接收状态 |
| **实时监控** | `cargo run --example ctp_monitor` | 持续监控数据接收 |
| **集成测试** | `cargo run --example ctp_integration` | 测试 Mock/Real 模式 |
| **完整测试** | `cargo run --example ctp_market_test` | 完整功能测试 |
| **订阅测试** | `cargo run --example ctp_subscribe_test` | 专门测试订阅功能 |

---

## 📈 监控指标

### 关键指标

1. **连接状态**
   - ✅ 已连接: MD + TD 都显示 "connected"
   - ❌ 未连接: 看到 "disconnected" 或连接错误

2. **数据接收速率**
   - Mock 模式: 即时响应（微秒级）
   - Real 模式: 根据市场活跃度，通常每秒多次更新

3. **响应时间**
   - Mock: < 1ms
   - Real: < 500ms

4. **成功率**
   - 期望: 100%
   - 异常: < 95%

---

## 💡 最佳实践

### 开发阶段
```yaml
# 使用 Mock 模式
mock_mode: true

# 优势：
✅ 零配置
✅ 稳定可靠
✅ 快速响应
✅ 适合功能开发
```

### 测试阶段
```yaml
# 切换到 Real 模式
mock_mode: false

# 验证：
1. 连接状态 ✅
2. 订阅功能 ✅
3. 数据接收 ✅
4. 错误处理 ✅
```

### 监控方法
```bash
# 定期运行状态检查
*/5 * * * * cargo run --example ctp_status >> status.log

# 实时监控
cargo run --example ctp_monitor | tee -a monitor.log

# 分析日志
grep -E "(✅|❌|⚠️)" log.txt
```

---

## 📚 相关文档

- `CTP_CONFIG.md` - 配置说明
- `CTP_INTEGRATION_COMPLETE.md` - 集成报告
- `backend/examples/ctp_status.rs` - 状态检查工具源码
- `backend/examples/ctp_monitor.rs` - 监控工具源码

---

**最后更新**: 2025-11-04  
**工具版本**: v1.0  
**测试状态**: ✅ 所有工具测试通过
