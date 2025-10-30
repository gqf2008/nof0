import { useExchange } from "@/contexts/ExchangeContext";

/**
 * 生成带交易所 ID 的 API URL
 * @param endpoint API 端点路径
 * @returns 完整的 API URL (带 exchange_id 查询参数)
 */
export function useExchangeAPI(endpoint: string): string {
  const { exchangeId } = useExchange();
  
  // 如果 endpoint 已经包含查询参数，使用 & 连接
  const separator = endpoint.includes("?") ? "&" : "?";
  
  return `${endpoint}${separator}exchange_id=${exchangeId}`;
}
