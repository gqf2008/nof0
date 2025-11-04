# NOF0 统一交易 API 标准接口规范

## 设计目标

1. **统一接口**: 所有交易所(CTP、Crypto、股票等)使用相同的API接口
2. **可扩展性**: 通过 `exchange_id` 参数路由到不同交易所
3. **类型安全**: 使用 TypeScript/Rust 强类型定义
4. **标准响应**: 统一的响应格式和错误处理
5. **实时推送**: WebSocket 标准化消息格式

---

## API 设计原则

### 1. RESTful 规范
- **资源导向**: `/api/v1/{resource}`
- **HTTP 动词**: GET(查询), POST(创建), PUT(更新), DELETE(删除)
- **状态码**: 200(成功), 400(参数错误), 401(未授权), 404(不存在), 500(服务器错误)

### 2. 统一响应格式
```typescript
interface ApiResponse<T> {
  success: boolean;      // 操作是否成功
  code: number;          // 业务状态码
  message: string;       // 提示信息
  data?: T;              // 响应数据
  timestamp: number;     // 时间戳(毫秒)
  request_id?: string;   // 请求追踪ID
}
```

### 3. 错误码规范
```typescript
enum ErrorCode {
  SUCCESS = 0,
  INVALID_PARAMS = 1001,      // 参数错误
  UNAUTHORIZED = 1002,         // 未授权
  NOT_FOUND = 1003,            // 资源不存在
  EXCHANGE_OFFLINE = 2001,     // 交易所离线
  INSUFFICIENT_BALANCE = 2002, // 余额不足
  ORDER_FAILED = 2003,         // 下单失败
  INTERNAL_ERROR = 5000        // 内部错误
}
```

---

## 核心 API 接口

### 1. 配置管理

#### 获取交易所列表
```
GET /api/v1/exchanges
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "exchanges": [
      {
        "id": "ctp",
        "name": "CTP期货",
        "type": "futures",
        "enabled": true,
        "status": "online",
        "features": ["futures", "options"]
      },
      {
        "id": "binance",
        "name": "币安",
        "type": "crypto",
        "enabled": true,
        "status": "online",
        "features": ["spot", "futures"]
      }
    ]
  },
  "timestamp": 1730707200000
}
```

#### 获取交易所配置
```
GET /api/v1/exchanges/{exchange_id}/config
```

#### 保存交易所配置
```
POST /api/v1/exchanges/{exchange_id}/config
Content-Type: application/json

{
  "broker_id": "9999",
  "account": "user123",
  "auth_params": { ... }
}
```

---

### 2. 连接管理

#### 连接交易所
```
POST /api/v1/exchanges/{exchange_id}/connect
```

#### 断开连接
```
POST /api/v1/exchanges/{exchange_id}/disconnect
```

#### 获取连接状态
```
GET /api/v1/exchanges/{exchange_id}/status
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "exchange_id": "ctp",
    "connected": true,
    "mode": "live",
    "connections": {
      "market_data": {
        "connected": true,
        "reconnecting": false,
        "reconnect_attempts": 0
      },
      "trading": {
        "connected": true,
        "reconnecting": false,
        "reconnect_attempts": 0
      }
    },
    "broker_info": {
      "broker_id": "9999",
      "broker_name": "SimNow"
    }
  },
  "timestamp": 1730707200000
}
```

---

### 3. 行情数据

#### 订阅行情
```
POST /api/v1/exchanges/{exchange_id}/market/subscribe
Content-Type: application/json

{
  "symbols": ["BTC-USDT", "ETH-USDT"]
}
```

#### 取消订阅
```
POST /api/v1/exchanges/{exchange_id}/market/unsubscribe
Content-Type: application/json

{
  "symbols": ["BTC-USDT"]
}
```

#### 获取单个行情
```
GET /api/v1/exchanges/{exchange_id}/market/ticker/{symbol}
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "symbol": "BTC-USDT",
    "exchange_id": "binance",
    "last_price": 43250.00,
    "bid_price": 43248.50,
    "ask_price": 43251.50,
    "volume_24h": 125000.00,
    "high_24h": 43500.00,
    "low_24h": 42800.00,
    "change_24h": 450.00,
    "change_percent_24h": 1.05,
    "timestamp": 1730707200000,
    "update_time": "2024-11-04T12:00:00Z"
  },
  "timestamp": 1730707200000
}
```

#### 获取批量行情
```
GET /api/v1/exchanges/{exchange_id}/market/tickers?symbols=BTC-USDT,ETH-USDT
```

#### 获取K线数据
```
GET /api/v1/exchanges/{exchange_id}/market/klines/{symbol}?interval=1m&limit=100
```

---

### 4. 账户查询

#### 查询账户信息
```
GET /api/v1/exchanges/{exchange_id}/account
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "exchange_id": "ctp",
    "account_id": "123456",
    "balance": 1000000.00,
    "available": 950000.00,
    "frozen": 50000.00,
    "equity": 1050000.00,
    "margin": 80000.00,
    "profit_loss": 50000.00,
    "currency": "CNY",
    "update_time": "2024-11-04T12:00:00Z"
  },
  "timestamp": 1730707200000
}
```

#### 查询持仓
```
GET /api/v1/exchanges/{exchange_id}/positions
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "positions": [
      {
        "symbol": "rb2501",
        "exchange_id": "ctp",
        "direction": "long",
        "volume": 10,
        "available_volume": 8,
        "frozen_volume": 2,
        "avg_price": 3500.00,
        "last_price": 3550.00,
        "profit_loss": 5000.00,
        "margin": 35000.00,
        "open_time": "2024-11-04T10:00:00Z",
        "update_time": "2024-11-04T12:00:00Z"
      }
    ]
  },
  "timestamp": 1730707200000
}
```

---

### 5. 交易操作

#### 下单
```
POST /api/v1/exchanges/{exchange_id}/orders
Content-Type: application/json

{
  "symbol": "rb2501",
  "direction": "buy",
  "offset": "open",
  "price_type": "limit",
  "price": 3500.00,
  "volume": 5,
  "strategy": "hedge"
}
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "下单成功",
  "data": {
    "order_id": "20241104120000001",
    "client_order_id": "client_001",
    "symbol": "rb2501",
    "direction": "buy",
    "offset": "open",
    "price": 3500.00,
    "volume": 5,
    "filled_volume": 0,
    "status": "pending",
    "submit_time": "2024-11-04T12:00:00Z"
  },
  "timestamp": 1730707200000
}
```

#### 撤单
```
DELETE /api/v1/exchanges/{exchange_id}/orders/{order_id}
```

#### 查询订单
```
GET /api/v1/exchanges/{exchange_id}/orders/{order_id}
```

#### 查询所有订单
```
GET /api/v1/exchanges/{exchange_id}/orders?status=pending&limit=50
```

#### 查询成交记录
```
GET /api/v1/exchanges/{exchange_id}/trades?start_time=2024-11-01&end_time=2024-11-04
```

---

### 6. 合约查询

#### 查询合约列表
```
GET /api/v1/exchanges/{exchange_id}/instruments?type=futures
```

#### 查询合约详情
```
GET /api/v1/exchanges/{exchange_id}/instruments/{symbol}
```

**响应示例**:
```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": {
    "symbol": "rb2501",
    "name": "螺纹钢2501",
    "exchange": "SHFE",
    "exchange_id": "ctp",
    "type": "futures",
    "contract_size": 10,
    "price_tick": 1.0,
    "margin_ratio": 0.08,
    "commission_ratio": 0.0001,
    "trading_hours": ["09:00-15:00", "21:00-23:00"],
    "expire_date": "2025-01-15"
  },
  "timestamp": 1730707200000
}
```

---

## WebSocket 实时推送

### 连接地址
```
ws://localhost:8788/api/v1/ws?exchange_id=ctp
```

### 消息格式

#### 订阅行情
```json
{
  "type": "subscribe",
  "channel": "market",
  "symbols": ["BTC-USDT", "ETH-USDT"]
}
```

#### 行情推送
```json
{
  "type": "market",
  "exchange_id": "binance",
  "data": {
    "symbol": "BTC-USDT",
    "last_price": 43250.00,
    "volume": 1.5,
    "timestamp": 1730707200000
  }
}
```

#### 订单状态推送
```json
{
  "type": "order",
  "exchange_id": "ctp",
  "data": {
    "order_id": "20241104120000001",
    "status": "filled",
    "filled_volume": 5,
    "avg_price": 3502.00,
    "update_time": "2024-11-04T12:01:00Z"
  }
}
```

#### 持仓变动推送
```json
{
  "type": "position",
  "exchange_id": "ctp",
  "data": {
    "symbol": "rb2501",
    "direction": "long",
    "volume": 15,
    "profit_loss": 7500.00
  }
}
```

#### 账户变动推送
```json
{
  "type": "account",
  "exchange_id": "ctp",
  "data": {
    "balance": 1005000.00,
    "available": 955000.00,
    "equity": 1055000.00
  }
}
```

---

## 数据类型定义

### TypeScript 类型
```typescript
// 交易所类型
type ExchangeType = 'futures' | 'crypto' | 'stock' | 'forex';

// 订单方向
type OrderDirection = 'buy' | 'sell';

// 开平标志
type OffsetFlag = 'open' | 'close' | 'close_today' | 'close_yesterday';

// 价格类型
type PriceType = 'limit' | 'market' | 'stop' | 'stop_limit';

// 订单状态
type OrderStatus = 'pending' | 'partial_filled' | 'filled' | 'cancelled' | 'rejected';

// 持仓方向
type PositionDirection = 'long' | 'short';
```

---

## 实现计划

### Phase 1: 核心接口抽象
- [ ] 定义统一的 Trait/Interface
- [ ] 实现 ApiResponse 标准化
- [ ] 创建 ExchangeManager 管理多交易所

### Phase 2: 适配器实现
- [ ] CTP 适配器
- [ ] Crypto 适配器
- [ ] Mock 适配器(测试用)

### Phase 3: 路由层
- [ ] 统一路由 `/api/v1/*`
- [ ] 基于 exchange_id 的动态路由
- [ ] 中间件(鉴权、限流、日志)

### Phase 4: WebSocket 推送
- [ ] 连接管理
- [ ] 消息分发
- [ ] 断线重连

### Phase 5: 前端集成
- [ ] TypeScript SDK 生成
- [ ] React Hooks 封装
- [ ] 示例页面

---

## 使用示例

### 前端调用示例
```typescript
import { UnifiedApiClient } from '@/lib/unified-api-client';

const api = new UnifiedApiClient('http://localhost:8788');

// 连接CTP
await api.connect('ctp');

// 订阅行情
await api.subscribeMarket('ctp', ['rb2501', 'au2412']);

// 查询账户
const account = await api.getAccount('ctp');

// 下单
const order = await api.placeOrder('ctp', {
  symbol: 'rb2501',
  direction: 'buy',
  price: 3500,
  volume: 5
});

// 监听行情推送
api.on('market', (data) => {
  console.log('行情更新:', data);
});
```

---

## 总结

这套标准接口具有以下优势:

1. ✅ **统一体验**: 所有交易所使用相同的接口,降低学习成本
2. ✅ **易于扩展**: 新增交易所只需实现标准接口
3. ✅ **类型安全**: TypeScript/Rust 强类型定义
4. ✅ **可测试性**: Mock 适配器方便测试
5. ✅ **可维护性**: 清晰的分层架构

下一步可以开始实现核心的 Trait 定义和路由层。
