/**
 * 交易所相关的 API 端点
 * 所有端点都会自动添加 exchange_id 查询参数
 */

const local = (p: string) => `/api/nof1${p}`;

/**
 * 添加 exchange_id 参数到 URL
 */
function withExchangeId(url: string, exchangeId: string): string {
  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}exchange_id=${exchangeId}`;
}

/**
 * 生成带交易所 ID 的 API 端点
 */
export function getExchangeEndpoints(exchangeId: string) {
  return {
    cryptoPrices: () => withExchangeId(local("/crypto-prices"), exchangeId),
    
    positions: (limit = 1000) => 
      withExchangeId(local(`/positions?limit=${limit}`), exchangeId),
    
    trades: () => 
      withExchangeId(local("/trades"), exchangeId),
    
    accountTotals: (lastHourlyMarker?: number) =>
      withExchangeId(
        local(`/account-totals${lastHourlyMarker != null ? `?lastHourlyMarker=${lastHourlyMarker}` : ""}`),
        exchangeId
      ),
    
    modelAccounts: () => 
      withExchangeId(local("/model-accounts"), exchangeId),
    
    sinceInceptionValues: () => 
      withExchangeId(local("/since-inception-values"), exchangeId),
    
    leaderboard: () => 
      withExchangeId(local("/leaderboard"), exchangeId),
    
    analytics: () => 
      withExchangeId(local("/analytics"), exchangeId),
    
    conversations: () => 
      withExchangeId(local("/conversations"), exchangeId),
  };
}
