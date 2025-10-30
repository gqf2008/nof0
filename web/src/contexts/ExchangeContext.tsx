import { createContext, useContext, ReactNode } from "react";

interface ExchangeContextType {
  exchangeId: string;
}

const ExchangeContext = createContext<ExchangeContextType | undefined>(undefined);

export function ExchangeProvider({ exchangeId, children }: { exchangeId: string; children: ReactNode }) {
  return (
    <ExchangeContext.Provider value={{ exchangeId }}>
      {children}
    </ExchangeContext.Provider>
  );
}

export function useExchange() {
  const context = useContext(ExchangeContext);
  if (!context) {
    throw new Error("useExchange must be used within ExchangeProvider");
  }
  return context;
}
