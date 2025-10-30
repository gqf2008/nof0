import { Route } from "react-router-dom";
import { useEffect, useState } from "react";
import CryptoExchangePage from "@/pages/CryptoExchangePage";
import CtpMonitorPage from "@/pages/CtpMonitorPage";
import GenericExchangePage from "@/pages/GenericExchangePage";

interface ExchangeConfig {
  id: string;
  name: string;
  route: string;
  enabled: boolean;
}

interface ExchangesConfigResponse {
  exchanges: ExchangeConfig[];
}

// 交易所ID到组件的映射
const EXCHANGE_COMPONENTS: Record<string, React.ComponentType> = {
  crypto: CryptoExchangePage,
  ctp: CtpMonitorPage,
  // 其他交易所使用通用页面
};

export default function DynamicExchangeRoutes() {
  const [exchanges, setExchanges] = useState<ExchangeConfig[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // 从后端获取交易所配置
    fetch("/api/config/exchanges")
      .then((res) => res.json())
      .then((data: ExchangesConfigResponse) => {
        // 只使用启用的交易所
        const enabledExchanges = data.exchanges.filter((e) => e.enabled);
        setExchanges(enabledExchanges);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load exchange routes:", err);
        setLoading(false);
      });
  }, []);

  if (loading) {
    // 加载时返回空路由
    return null;
  }

  // 根据配置动态生成路由
  return (
    <>
      {exchanges.map((exchange) => {
        const Component = EXCHANGE_COMPONENTS[exchange.id] || GenericExchangePage;
        return (
          <Route
            key={exchange.id}
            path={exchange.route}
            element={<Component />}
          />
        );
      })}
    </>
  );
}
