/**
 * 统一 API React Hooks
 * 
 * 提供便捷的 React 数据获取和状态管理
 */

import { useState, useEffect, useCallback } from 'react';
import { unifiedApi } from '@/lib/unified-api-client';
import type {
  ExchangeInfo,
  ConnectionStatus,
  MarketTicker,
  Account,
  Position,
  Order,
  OrderRequest,
} from '@/types/unified-api';

/**
 * 获取交易所列表
 */
export function useExchanges() {
  const [exchanges, setExchanges] = useState<ExchangeInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchExchanges = useCallback(async () => {
    try {
      setLoading(true);
      const res = await unifiedApi.listExchanges();
      if (res.success && res.data) {
        setExchanges(res.data.exchanges);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取交易所列表失败');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchExchanges();
  }, [fetchExchanges]);

  return { exchanges, loading, error, refetch: fetchExchanges };
}

/**
 * 交易所连接管理
 */
export function useExchange(exchangeId: string) {
  const [status, setStatus] = useState<ConnectionStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await unifiedApi.getStatus(exchangeId);
      if (res.success && res.data) {
        setStatus(res.data);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取状态失败');
    }
  }, [exchangeId]);

  const connect = useCallback(async () => {
    setLoading(true);
    try {
      const res = await unifiedApi.connect(exchangeId);
      if (res.success) {
        await fetchStatus();
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '连接失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId, fetchStatus]);

  const disconnect = useCallback(async () => {
    setLoading(true);
    try {
      const res = await unifiedApi.disconnect(exchangeId);
      if (res.success) {
        await fetchStatus();
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '断开失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId, fetchStatus]);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // 每5秒刷新状态
    return () => clearInterval(interval);
  }, [fetchStatus]);

  return { status, loading, error, connect, disconnect, refetch: fetchStatus };
}

/**
 * 行情数据
 */
export function useMarketTicker(exchangeId: string, symbol: string, refreshInterval = 2000) {
  const [ticker, setTicker] = useState<MarketTicker | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTicker = useCallback(async () => {
    try {
      setLoading(true);
      const res = await unifiedApi.getTicker(exchangeId, symbol);
      if (res.success && res.data) {
        setTicker(res.data);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取行情失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId, symbol]);

  useEffect(() => {
    fetchTicker();
    if (refreshInterval > 0) {
      const interval = setInterval(fetchTicker, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchTicker, refreshInterval]);

  return { ticker, loading, error, refetch: fetchTicker };
}

/**
 * 账户信息
 */
export function useAccount(exchangeId: string, refreshInterval = 5000) {
  const [account, setAccount] = useState<Account | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAccount = useCallback(async () => {
    try {
      setLoading(true);
      const res = await unifiedApi.getAccount(exchangeId);
      if (res.success && res.data) {
        setAccount(res.data);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取账户失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId]);

  useEffect(() => {
    fetchAccount();
    if (refreshInterval > 0) {
      const interval = setInterval(fetchAccount, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchAccount, refreshInterval]);

  return { account, loading, error, refetch: fetchAccount };
}

/**
 * 持仓信息
 */
export function usePositions(exchangeId: string, refreshInterval = 3000) {
  const [positions, setPositions] = useState<Position[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchPositions = useCallback(async () => {
    try {
      setLoading(true);
      const res = await unifiedApi.getPositions(exchangeId);
      if (res.success && res.data) {
        setPositions(res.data);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取持仓失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId]);

  useEffect(() => {
    fetchPositions();
    if (refreshInterval > 0) {
      const interval = setInterval(fetchPositions, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchPositions, refreshInterval]);

  return { positions, loading, error, refetch: fetchPositions };
}

/**
 * 订单操作
 */
export function useOrders(exchangeId: string) {
  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOrders = useCallback(async () => {
    try {
      setLoading(true);
      const res = await unifiedApi.getOrders(exchangeId);
      if (res.success && res.data) {
        setOrders(res.data);
        setError(null);
      } else {
        setError(res.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取订单失败');
    } finally {
      setLoading(false);
    }
  }, [exchangeId]);

  const placeOrder = useCallback(
    async (orderReq: OrderRequest) => {
      setLoading(true);
      try {
        const res = await unifiedApi.placeOrder(exchangeId, orderReq);
        if (res.success) {
          await fetchOrders(); // 刷新订单列表
          setError(null);
          return res.data;
        } else {
          setError(res.message);
          return null;
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : '下单失败');
        return null;
      } finally {
        setLoading(false);
      }
    },
    [exchangeId, fetchOrders]
  );

  const cancelOrder = useCallback(
    async (orderId: string) => {
      setLoading(true);
      try {
        const res = await unifiedApi.cancelOrder(exchangeId, orderId);
        if (res.success) {
          await fetchOrders(); // 刷新订单列表
          setError(null);
          return true;
        } else {
          setError(res.message);
          return false;
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : '撤单失败');
        return false;
      } finally {
        setLoading(false);
      }
    },
    [exchangeId, fetchOrders]
  );

  useEffect(() => {
    fetchOrders();
  }, [fetchOrders]);

  return { orders, loading, error, placeOrder, cancelOrder, refetch: fetchOrders };
}
