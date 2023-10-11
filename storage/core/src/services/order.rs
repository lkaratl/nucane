use uuid::Uuid;

use domain_model::{Currency, Exchange, MarketType, OrderStatus, OrderType, Side};
use storage_persistence_api::OrderRepository;

pub struct OrderService<R: OrderRepository> {
    repository: R,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn save(&self, order: domain_model::Order) {
        self.repository
            .save(order)
            .await
            .expect("Error during order saving");
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get(
        &self,
        id: Option<String>,
        simulation_id: Option<Uuid>,
        exchange: Option<Exchange>,
        market_type: Option<MarketType>,
        target: Option<Currency>,
        source: Option<Currency>,
        status: Option<OrderStatus>,
        side: Option<Side>,
        order_type: Option<OrderType>,
    ) -> Vec<domain_model::Order> {
        self.repository
            .get(
                id,
                simulation_id,
                exchange,
                market_type,
                target,
                source,
                status,
                side,
                order_type,
            )
            .await
            .unwrap()
    }
}
