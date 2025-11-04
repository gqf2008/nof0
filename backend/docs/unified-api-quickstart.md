# 统一 API 快速开始

## 概述

NOF0 统一 API 提供了标准化的交易接口,允许前端通过相同的API调用不同的交易所(CTP期货、数字货币、股票等)。

## 架构设计

```
┌─────────────┐
│   前端应用   │
└──────┬──────┘
       │ HTTP/WebSocket
       │
┌──────▼──────────────────────────────┐
│     统一 API 路由层 (/api/v1/*)     │
│  ┌──────────────────────────────┐   │
│  │   ExchangeManager            │   │
│  │  - 管理多个交易所适配器       │   │
│  │  - 基于 exchange_id 路由     │   │
│  └──────────┬───────────────────┘   │
└─────────────┼───────────────────────┘
              │
     ┌────────┴────────┬────────────┐
     │                 │            │
┌────▼─────┐  ┌───────▼────┐  ┌───▼─────┐
│   CTP    │  │   Crypto   │  │  Stock  │
│ Adapter  │  │  Adapter   │  │ Adapter │
└────┬─────┘  └─────┬──────┘  └────┬────┘
     │              │              │
┌────▼─────┐  ┌────▼──────┐  ┌───▼─────┐
│ CTP SDK  │  │  Binance  │  │   XTP   │
│  (real)  │  │    API    │  │   API   │
└──────────┘  └───────────┘  └─────────┘
```

## 核心特性

1. **统一接口**: 所有交易所使用相同的API规范
2. **动态路由**: 通过 `exchange_id` 参数路由到不同交易所
3. **标准响应**: 统一的 `ApiResponse<T>` 格式
4. **类型安全**: 完整的 TypeScript/Rust 类型定义
5. **可扩展**: 实现 Trait 即可添加新交易所

## 快速开始

### 1. 后端集成

#### 在 server.rs 中注册统一 API:

```rust
use nof0_backend::api::{ExchangeManager, create_unified_routes, adapters::CtpMockAdapter};
use std::sync::Arc;

pub async fn run_http_server(addr: SocketAddr, url: String) -> anyhow::Result<()> {
    // 创建交易所管理器
    let mut exchange_manager = ExchangeManager::new();
    
    // 注册 CTP Mock 适配器
    let ctp_adapter = Arc::new(CtpMockAdapter::new("ctp", "CTP期货"));
    exchange_manager.register(ctp_adapter);
    
    // 可以注册更多交易所
    // let binance_adapter = Arc::new(BinanceAdapter::new("binance", "币安"));
    // exchange_manager.register(binance_adapter);
    
    let exchange_manager = Arc::new(exchange_manager);
    
    // 创建统一 API 路由
    let unified_routes = create_unified_routes(exchange_manager.clone());
    
    // 合并到主路由
    let app = Router::new()
        .merge(unified_routes)
        .route("/health", get(health))
        // ... 其他路由
        .layer(cors);
    
    // 启动服务器
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .await
        .context("server error")?;
    
    Ok(())
}
```

### 2. 前端调用

#### 创建 API 客户端:

```typescript
// lib/unified-api-client.ts
import type { 
  ApiResponse, 
  ConnectionStatus,
  MarketTicker,
  Account,
  Position,
  OrderRequest,
  Order
} from '@/types/api';

export class UnifiedApiClient {
  constructor(private baseUrl: string = 'http://localhost:8788') {}

  // ==================== 交易所管理 ====================
  
  async listExchanges(): Promise<ApiResponse<{ exchanges: ExchangeInfo[] }>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges`);
    return res.json();
  }

  async getStatus(exchangeId: string): Promise<ApiResponse<ConnectionStatus>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/status`);
    return res.json();
  }

  async connect(exchangeId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/connect`, {
      method: 'POST',
    });
    return res.json();
  }

  async disconnect(exchangeId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/disconnect`, {
      method: 'POST',
    });
    return res.json();
  }

  // ==================== 行情数据 ====================
  
  async subscribeMarket(exchangeId: string, symbols: string[]): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/subscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ symbols }),
    });
    return res.json();
  }

  async getTicker(exchangeId: string, symbol: string): Promise<ApiResponse<MarketTicker>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/ticker/${symbol}`);
    return res.json();
  }

  async getTickers(exchangeId: string, symbols: string[]): Promise<ApiResponse<MarketTicker[]>> {
    const symbolsParam = symbols.join(',');
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/tickers?symbols=${symbolsParam}`);
    return res.json();
  }

  // ==================== 账户查询 ====================
  
  async getAccount(exchangeId: string): Promise<ApiResponse<Account>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/account`);
    return res.json();
  }

  async getPositions(exchangeId: string): Promise<ApiResponse<Position[]>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/positions`);
    return res.json();
  }

  // ==================== 交易操作 ====================
  
  async placeOrder(exchangeId: string, order: OrderRequest): Promise<ApiResponse<Order>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(order),
    });
    return res.json();
  }

  async cancelOrder(exchangeId: string, orderId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders/${orderId}`, {
      method: 'DELETE',
    });
    return res.json();
  }
}

// 导出单例
export const unifiedApi = new UnifiedApiClient();
```

#### React Hook 封装:

```typescript
// hooks/useExchange.ts
import { useState, useEffect } from 'react';
import { unifiedApi } from '@/lib/unified-api-client';
import type { ConnectionStatus, MarketTicker } from '@/types/api';

export function useExchange(exchangeId: string) {
  const [status, setStatus] = useState<ConnectionStatus | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadStatus();
  }, [exchangeId]);

  const loadStatus = async () => {
    const res = await unifiedApi.getStatus(exchangeId);
    if (res.success && res.data) {
      setStatus(res.data);
    }
  };

  const connect = async () => {
    setLoading(true);
    try {
      const res = await unifiedApi.connect(exchangeId);
      if (res.success) {
        await loadStatus();
      }
    } finally {
      setLoading(false);
    }
  };

  const disconnect = async () => {
    setLoading(true);
    try {
      const res = await unifiedApi.disconnect(exchangeId);
      if (res.success) {
        await loadStatus();
      }
    } finally {
      setLoading(false);
    }
  };

  return { status, loading, connect, disconnect, refresh: loadStatus };
}

export function useMarketData(exchangeId: string, symbol: string) {
  const [ticker, setTicker] = useState<MarketTicker | null>(null);

  useEffect(() => {
    const fetchTicker = async () => {
      const res = await unifiedApi.getTicker(exchangeId, symbol);
      if (res.success && res.data) {
        setTicker(res.data);
      }
    };

    fetchTicker();
    const interval = setInterval(fetchTicker, 2000); // 2秒刷新
    return () => clearInterval(interval);
  }, [exchangeId, symbol]);

  return ticker;
}
```

#### 组件示例:

```tsx
// components/ExchangeMonitor.tsx
import { useExchange, useMarketData } from '@/hooks/useExchange';

export function ExchangeMonitor({ exchangeId }: { exchangeId: string }) {
  const { status, loading, connect, disconnect } = useExchange(exchangeId);
  const ticker = useMarketData(exchangeId, 'rb2501');

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4">
        {status?.broker_info?.broker_name || exchangeId}
      </h2>

      {/* 连接状态 */}
      <div className="mb-4">
        <div className="flex items-center gap-2">
          <span className={status?.connected ? 'text-green-500' : 'text-red-500'}>
            ● {status?.connected ? '已连接' : '未连接'}
          </span>
          <span className="text-gray-500">({status?.mode})</span>
        </div>
        
        <button
          onClick={status?.connected ? disconnect : connect}
          disabled={loading}
          className="mt-2 px-4 py-2 bg-blue-500 text-white rounded"
        >
          {loading ? '处理中...' : (status?.connected ? '断开' : '连接')}
        </button>
      </div>

      {/* 行情数据 */}
      {ticker && (
        <div className="bg-gray-100 p-4 rounded">
          <h3 className="font-bold mb-2">{ticker.symbol} 行情</h3>
          <div className="grid grid-cols-2 gap-2">
            <div>最新价: {ticker.last_price}</div>
            <div>涨跌: {ticker.change_24h.toFixed(2)}</div>
            <div>买价: {ticker.bid_price}</div>
            <div>卖价: {ticker.ask_price}</div>
          </div>
        </div>
      )}
    </div>
  );
}
```

## API 端点速查

### 交易所管理
- `GET /api/v1/exchanges` - 列出所有交易所
- `GET /api/v1/exchanges/{exchange_id}/status` - 获取连接状态
- `POST /api/v1/exchanges/{exchange_id}/connect` - 连接交易所
- `POST /api/v1/exchanges/{exchange_id}/disconnect` - 断开连接

### 行情数据
- `POST /api/v1/exchanges/{exchange_id}/market/subscribe` - 订阅行情
- `GET /api/v1/exchanges/{exchange_id}/market/ticker/{symbol}` - 获取单个行情
- `GET /api/v1/exchanges/{exchange_id}/market/tickers?symbols=...` - 批量获取行情
- `GET /api/v1/exchanges/{exchange_id}/market/klines/{symbol}?interval=1m&limit=100` - K线数据

### 账户查询
- `GET /api/v1/exchanges/{exchange_id}/account` - 查询账户
- `GET /api/v1/exchanges/{exchange_id}/positions` - 查询持仓

### 交易操作
- `POST /api/v1/exchanges/{exchange_id}/orders` - 下单
- `GET /api/v1/exchanges/{exchange_id}/orders` - 查询订单列表
- `GET /api/v1/exchanges/{exchange_id}/orders/{order_id}` - 查询单个订单
- `DELETE /api/v1/exchanges/{exchange_id}/orders/{order_id}` - 撤单
- `GET /api/v1/exchanges/{exchange_id}/trades` - 成交记录

## 响应格式

所有 API 返回统一格式:

```json
{
  "success": true,
  "code": 0,
  "message": "success",
  "data": { ... },
  "timestamp": 1730707200000,
  "request_id": "uuid-here"
}
```

错误响应:

```json
{
  "success": false,
  "code": 2001,
  "message": "交易所离线",
  "timestamp": 1730707200000
}
```

## 实现新的交易所适配器

```rust
use async_trait::async_trait;
use nof0_backend::api::{traits::*, types::*};

pub struct MyExchangeAdapter {
    id: String,
    name: String,
    // ... 其他字段
}

#[async_trait]
impl ExchangeAdapter for MyExchangeAdapter {
    fn exchange_id(&self) -> &str { &self.id }
    fn exchange_name(&self) -> &str { &self.name }
    fn exchange_type(&self) -> ExchangeType { ExchangeType::Crypto }
    
    async fn connect(&self) -> Result<()> {
        // 实现连接逻辑
        Ok(())
    }
    
    // ... 实现其他方法
}

#[async_trait]
impl MarketDataProvider for MyExchangeAdapter {
    async fn get_ticker(&self, symbol: &str) -> Result<MarketTicker> {
        // 实现获取行情逻辑
        todo!()
    }
    
    // ... 实现其他方法
}

// 实现 AccountProvider, TradingProvider, InstrumentProvider...
```

## 测试

```bash
# 启动服务器
cargo run --features ctp-real

# 测试 API
curl http://localhost:8788/api/v1/exchanges
curl http://localhost:8788/api/v1/exchanges/ctp/status
curl -X POST http://localhost:8788/api/v1/exchanges/ctp/connect
curl http://localhost:8788/api/v1/exchanges/ctp/market/ticker/rb2501
```

## 下一步

1. ✅ 核心接口定义完成
2. ✅ Mock 适配器实现
3. ✅ 路由层实现
4. ⏳ 集成到 server.rs
5. ⏳ 前端 TypeScript SDK
6. ⏳ 真实 CTP 适配器
7. ⏳ Crypto 适配器(Binance/OKX)
8. ⏳ WebSocket 实时推送
9. ⏳ 完善文档和示例

## 参考

- [API 标准接口规范](./api-standard.md)
- [类型定义](../src/api/types.rs)
- [Trait 接口](../src/api/traits.rs)
