/**
 * 统一交易 API 类型定义
 * 
 * 与后端 backend/src/api/types.rs 保持一致
 */

// ==================== API 响应 ====================

export interface ApiResponse<T> {
  success: boolean;
  code: number;
  message: string;
  data?: T;
  timestamp: number;
  request_id?: string;
}

// ==================== 交易所相关 ====================

export type ExchangeType = 'futures' | 'crypto' | 'stock' | 'forex';

export interface ExchangeInfo {
  id: string;
  name: string;
  exchange_type: ExchangeType;
  enabled: boolean;
  status: string;
  features: string[];
}

export interface ConnectionStatus {
  exchange_id: string;
  connected: boolean;
  mode: string;
  connections: ConnectionDetails;
  broker_info?: BrokerInfo;
}

export interface ConnectionDetails {
  market_data: ConnectionState;
  trading: ConnectionState;
}

export interface ConnectionState {
  connected: boolean;
  reconnecting: boolean;
  reconnect_attempts: number;
}

export interface BrokerInfo {
  broker_id: string;
  broker_name: string;
}

// ==================== 行情数据 ====================

export interface MarketTicker {
  symbol: string;
  exchange_id: string;
  last_price: number;
  bid_price: number;
  ask_price: number;
  volume_24h: number;
  high_24h: number;
  low_24h: number;
  change_24h: number;
  change_percent_24h: number;
  timestamp: number;
  update_time: string;
}

export interface Kline {
  symbol: string;
  interval: string;
  open_time: number;
  close_time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
}

export interface SubscribeRequest {
  symbols: string[];
}

// ==================== 账户相关 ====================

export interface Account {
  exchange_id: string;
  account_id: string;
  balance: number;
  available: number;
  frozen: number;
  equity: number;
  margin: number;
  profit_loss: number;
  currency: string;
  update_time: string;
}

export interface Position {
  symbol: string;
  exchange_id: string;
  direction: PositionDirection;
  volume: number;
  available_volume: number;
  frozen_volume: number;
  avg_price: number;
  last_price: number;
  profit_loss: number;
  margin: number;
  open_time: string;
  update_time: string;
}

export type PositionDirection = 'long' | 'short';

// ==================== 交易相关 ====================

export interface OrderRequest {
  symbol: string;
  direction: OrderDirection;
  offset: OffsetFlag;
  price_type: PriceType;
  price: number;
  volume: number;
  strategy?: string;
}

export interface Order {
  order_id: string;
  client_order_id: string;
  symbol: string;
  exchange_id: string;
  direction: OrderDirection;
  offset: OffsetFlag;
  price: number;
  volume: number;
  filled_volume: number;
  status: OrderStatus;
  submit_time: string;
  update_time?: string;
}

export type OrderDirection = 'buy' | 'sell';
export type OffsetFlag = 'open' | 'close' | 'close_today' | 'close_yesterday';
export type PriceType = 'limit' | 'market' | 'stop' | 'stop_limit';
export type OrderStatus = 'pending' | 'partial_filled' | 'filled' | 'cancelled' | 'rejected';

export interface Trade {
  trade_id: string;
  order_id: string;
  symbol: string;
  exchange_id: string;
  direction: OrderDirection;
  price: number;
  volume: number;
  trade_time: string;
}

// ==================== 合约信息 ====================

export interface Instrument {
  symbol: string;
  name: string;
  exchange: string;
  exchange_id: string;
  instrument_type: string;
  contract_size: number;
  price_tick: number;
  margin_ratio: number;
  commission_ratio: number;
  trading_hours: string[];
  expire_date?: string;
}

// ==================== WebSocket 消息 ====================

export type WsMessage =
  | { type: 'subscribe'; channel: string; symbols: string[] }
  | { type: 'unsubscribe'; channel: string; symbols: string[] }
  | { type: 'market'; exchange_id: string; data: MarketTicker }
  | { type: 'order'; exchange_id: string; data: Order }
  | { type: 'position'; exchange_id: string; data: Position }
  | { type: 'account'; exchange_id: string; data: Account }
  | { type: 'ping' }
  | { type: 'pong' };
