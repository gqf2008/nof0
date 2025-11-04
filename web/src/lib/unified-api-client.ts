/**
 * 统一交易 API 客户端
 * 
 * 提供标准化的交易接口调用方法
 */

import type {
  ApiResponse,
  ExchangeInfo,
  ConnectionStatus,
  MarketTicker,
  Kline,
  SubscribeRequest,
  Account,
  Position,
  OrderRequest,
  Order,
  OrderStatus,
  Trade,
  Instrument,
} from '@/types/unified-api';

export class UnifiedApiClient {
  constructor(private baseUrl: string = 'http://localhost:8788') {}

  // ==================== 交易所管理 ====================

  /**
   * 获取所有交易所列表
   */
  async listExchanges(): Promise<ApiResponse<{ exchanges: ExchangeInfo[] }>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges`);
    return res.json();
  }

  /**
   * 获取交易所连接状态
   */
  async getStatus(exchangeId: string): Promise<ApiResponse<ConnectionStatus>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/status`);
    return res.json();
  }

  /**
   * 连接交易所
   */
  async connect(exchangeId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/connect`, {
      method: 'POST',
    });
    return res.json();
  }

  /**
   * 断开交易所连接
   */
  async disconnect(exchangeId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/disconnect`, {
      method: 'POST',
    });
    return res.json();
  }

  // ==================== 行情数据 ====================

  /**
   * 订阅行情
   */
  async subscribeMarket(exchangeId: string, symbols: string[]): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/subscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ symbols } as SubscribeRequest),
    });
    return res.json();
  }

  /**
   * 取消订阅行情
   */
  async unsubscribeMarket(exchangeId: string, symbols: string[]): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/unsubscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ symbols } as SubscribeRequest),
    });
    return res.json();
  }

  /**
   * 获取单个合约行情
   */
  async getTicker(exchangeId: string, symbol: string): Promise<ApiResponse<MarketTicker>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/ticker/${symbol}`);
    return res.json();
  }

  /**
   * 获取批量行情
   */
  async getTickers(exchangeId: string, symbols: string[]): Promise<ApiResponse<MarketTicker[]>> {
    const symbolsParam = symbols.join(',');
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/tickers?symbols=${symbolsParam}`);
    return res.json();
  }

  /**
   * 获取K线数据
   */
  async getKlines(
    exchangeId: string,
    symbol: string,
    interval: string,
    limit?: number
  ): Promise<ApiResponse<Kline[]>> {
    const params = new URLSearchParams({ interval });
    if (limit) params.append('limit', limit.toString());
    
    const res = await fetch(
      `${this.baseUrl}/api/v1/exchanges/${exchangeId}/market/klines/${symbol}?${params}`
    );
    return res.json();
  }

  // ==================== 账户查询 ====================

  /**
   * 查询账户信息
   */
  async getAccount(exchangeId: string): Promise<ApiResponse<Account>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/account`);
    return res.json();
  }

  /**
   * 查询持仓
   */
  async getPositions(exchangeId: string): Promise<ApiResponse<Position[]>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/positions`);
    return res.json();
  }

  // ==================== 交易操作 ====================

  /**
   * 下单
   */
  async placeOrder(exchangeId: string, order: OrderRequest): Promise<ApiResponse<Order>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(order),
    });
    return res.json();
  }

  /**
   * 撤单
   */
  async cancelOrder(exchangeId: string, orderId: string): Promise<ApiResponse<void>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders/${orderId}`, {
      method: 'DELETE',
    });
    return res.json();
  }

  /**
   * 查询单个订单
   */
  async getOrder(exchangeId: string, orderId: string): Promise<ApiResponse<Order>> {
    const res = await fetch(`${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders/${orderId}`);
    return res.json();
  }

  /**
   * 查询订单列表
   */
  async getOrders(
    exchangeId: string,
    status?: OrderStatus,
    limit?: number
  ): Promise<ApiResponse<Order[]>> {
    const params = new URLSearchParams();
    if (status) params.append('status', status);
    if (limit) params.append('limit', limit.toString());
    
    const res = await fetch(
      `${this.baseUrl}/api/v1/exchanges/${exchangeId}/orders?${params}`
    );
    return res.json();
  }

  /**
   * 查询成交记录
   */
  async getTrades(
    exchangeId: string,
    startTime?: string,
    endTime?: string
  ): Promise<ApiResponse<Trade[]>> {
    const params = new URLSearchParams();
    if (startTime) params.append('start_time', startTime);
    if (endTime) params.append('end_time', endTime);
    
    const res = await fetch(
      `${this.baseUrl}/api/v1/exchanges/${exchangeId}/trades?${params}`
    );
    return res.json();
  }

  // ==================== 合约查询 ====================

  /**
   * 查询合约列表
   */
  async getInstruments(
    exchangeId: string,
    instrumentType?: string
  ): Promise<ApiResponse<Instrument[]>> {
    const params = new URLSearchParams();
    if (instrumentType) params.append('type', instrumentType);
    
    const res = await fetch(
      `${this.baseUrl}/api/v1/exchanges/${exchangeId}/instruments?${params}`
    );
    return res.json();
  }

  /**
   * 查询合约详情
   */
  async getInstrument(exchangeId: string, symbol: string): Promise<ApiResponse<Instrument>> {
    const res = await fetch(
      `${this.baseUrl}/api/v1/exchanges/${exchangeId}/instruments/${symbol}`
    );
    return res.json();
  }
}

// 导出单例
export const unifiedApi = new UnifiedApiClient();
