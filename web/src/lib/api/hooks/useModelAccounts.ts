"use client";
import useSWR from "swr";
import { activityAwareRefresh } from "./activityAware";
import { fetcher } from "../client";
import { getExchangeEndpoints } from "../exchange-endpoints";
import { useExchange } from "@/contexts/ExchangeContext";
import { useMemo } from "react";

export interface ModelAccount {
  model_id: string;
  model_name: string;
  strategy: string;
  risk_level: string;
  exchange_id: string;
  timestamp: number;
  account_value: number;
  dollar_equity: number;
  equity: number;
  realized_pnl: number;
  unrealized_pnl: number;
  total_unrealized_pnl: number;
  return_pct: number;
  cum_pnl_pct: number;
  sharpe_ratio: number;
  win_rate: number;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
}

interface ModelAccountsResponse {
  accounts: ModelAccount[];
}

export function useModelAccounts() {
  const { exchangeId } = useExchange();
  const endpoints = useMemo(() => getExchangeEndpoints(exchangeId), [exchangeId]);
  
  const { data, error, isLoading } = useSWR<ModelAccountsResponse>(
    endpoints.modelAccounts(),
    fetcher,
    {
      ...activityAwareRefresh(10_000),
    },
  );

  return {
    accounts: data?.accounts || [],
    isLoading,
    isError: !!error,
  };
}
