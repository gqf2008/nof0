// CTP API 客户端 (TypeScript)
// 前端调用后端CTP API的封装

export interface CtpConfig {
  broker_id: string;
  investor_id: string;
  password: string;
  md_address: string;
  td_address: string;
  app_id?: string;
  auth_code?: string;
  user_product_info: string;
}

export interface ConnectionStatus {
  connected: boolean;
  md_connected: boolean;
  td_connected: boolean;
  md_reconnecting: boolean;
  td_reconnecting: boolean;
  md_reconnect_attempts: number;
  td_reconnect_attempts: number;
}

export interface MarketData {
  instrument_id: string;
  last_price: number;
  bid_price: number;
  ask_price: number;
  bid_volume: number;
  ask_volume: number;
  volume: number;
  turnover: number;
  open_interest: number;
  update_time: string;
}

export interface Account {
  account_id: string;
  balance: number;
  available: number;
  margin: number;
  frozen_margin: number;
  commission: number;
  close_profit: number;
  position_profit: number;
}

export interface Position {
  instrument_id: string;
  direction: string;
  position: number;
  today_position: number;
  available: number;
  open_cost: number;
  position_profit: number;
}

export interface OrderRequest {
  instrument_id: string;
  direction: string; // '0'=买, '1'=卖
  offset_flag: string; // '0'=开仓, '1'=平仓, '3'=平今
  price: number;
  volume: number;
  price_type: string; // '1'=市价, '2'=限价
  hedge_flag: string; // '1'=投机, '2'=套利, '3'=套保
}

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data?: T;
}

class CtpApiClient {
  private baseUrl: string;
  private ws: WebSocket | null = null;
  private wsListeners: Map<string, Set<(data: any) => void>> = new Map();

  constructor(baseUrl: string = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  // ==================== 配置管理 ====================

  async saveConfig(config: CtpConfig): Promise<ApiResponse<void>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return response.json();
  }

  async getConfig(): Promise<ApiResponse<CtpConfig>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/config`);
    return response.json();
  }

  // ==================== 连接管理 ====================

  async connect(): Promise<ApiResponse<void>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/connect`, {
      method: 'POST',
    });
    return response.json();
  }

  async disconnect(): Promise<ApiResponse<void>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/disconnect`, {
      method: 'POST',
    });
    return response.json();
  }

  async getStatus(): Promise<ApiResponse<ConnectionStatus>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/status`);
    return response.json();
  }

  // ==================== 行情订阅 ====================

  async subscribe(instruments: string[]): Promise<ApiResponse<void>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/subscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ instruments }),
    });
    return response.json();
  }

  async unsubscribe(instruments: string[]): Promise<ApiResponse<void>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/unsubscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ instruments }),
    });
    return response.json();
  }

  async getMarketData(instrumentId: string): Promise<ApiResponse<MarketData>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/market/${instrumentId}`);
    return response.json();
  }

  // ==================== 账户查询 ====================

  async queryAccount(): Promise<ApiResponse<Account>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/account`);
    return response.json();
  }

  async queryPositions(): Promise<ApiResponse<Position[]>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/positions`);
    return response.json();
  }

  // ==================== 交易操作 ====================

  async placeOrder(order: OrderRequest): Promise<ApiResponse<string>> {
    const response = await fetch(`${this.baseUrl}/api/ctp/order`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(order),
    });
    return response.json();
  }

  // ==================== WebSocket实时推送 ====================

  connectWebSocket(wsUrl?: string): void {
    const url = wsUrl || this.baseUrl.replace('http', 'ws') + '/api/ctp/ws';
    
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('✅ WebSocket connected');
    };

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        const { type, data } = message;

        // 触发对应类型的监听器
        const listeners = this.wsListeners.get(type);
        if (listeners) {
          listeners.forEach(callback => callback(data));
        }
      } catch (error) {
        console.error('WebSocket message parse error:', error);
      }
    };

    this.ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('🔌 WebSocket disconnected');
      // 3秒后自动重连
      setTimeout(() => this.connectWebSocket(wsUrl), 3000);
    };
  }

  disconnectWebSocket(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  // 监听特定类型的消息
  on(type: 'market' | 'account' | 'positions', callback: (data: any) => void): () => void {
    if (!this.wsListeners.has(type)) {
      this.wsListeners.set(type, new Set());
    }
    this.wsListeners.get(type)!.add(callback);

    // 返回取消监听的函数
    return () => {
      const listeners = this.wsListeners.get(type);
      if (listeners) {
        listeners.delete(callback);
      }
    };
  }
}

// 导出单例 (使用与后端相同的端口 8788)
export const ctpApi = new CtpApiClient('http://localhost:8788');
