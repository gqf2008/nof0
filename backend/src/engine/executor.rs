use crate::markets::{MarketAdapter, Order};
use tracing::info;

pub struct OrderExecutor {
    market: Box<dyn MarketAdapter>,
}

impl OrderExecutor {
    pub fn new(market: Box<dyn MarketAdapter>) -> Self {
        Self { market }
    }

    pub async fn execute(&self, order: Order) -> Result<String, anyhow::Error> {
        info!("Executing order: {:?}", order);
        self.market.place_order(order).await
    }
}
