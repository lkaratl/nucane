use std::sync::Arc;

use uuid::Uuid;

use domain_model::{Currency, Exchange, LP, MarketType, OrderStatus, OrderType, Side};
use interactor_core_api::InteractorApi;
use storage_persistence_api::OrderRepository;

pub struct OrderService<R: OrderRepository, I: InteractorApi> {
    repository: R,
    interactor_client: Arc<I>,
}

impl<R: OrderRepository, I: InteractorApi> OrderService<R, I> {
    pub fn new(repository: R, interactor_client: Arc<I>) -> Self {
        Self {
            repository,
            interactor_client,
        }
    }

    pub async fn save(&self, order: domain_model::Order) {
        self.repository
            .save(order)
            .await
            .expect("Error during order saving");
    }

    pub async fn save_lp(&self, lp: LP) {
        let order = self.get(Some(lp.id), None, None, None, None, None, None, None, None).await;
        if let Some(mut order) = order.first().cloned() {
            if order.stop_loss.is_some() && order.take_profit.is_some() {
                let sl_price = match order.clone().stop_loss.unwrap().order_px {
                    OrderType::Limit(price) => price,
                    OrderType::Market => order.clone().stop_loss.unwrap().trigger_px
                };
                let tp_price = match order.clone().take_profit.unwrap().order_px {
                    OrderType::Limit(price) => price,
                    OrderType::Market => order.clone().take_profit.unwrap().trigger_px
                };
                let sl_offset = (lp.price - sl_price).abs();
                let tp_offset = (lp.price - tp_price).abs();
                if tp_offset < sl_offset {
                    order.avg_tp_price = lp.price;
                    order.size = lp.size;
                } else {
                    order.avg_sl_price = lp.price;
                    order.size = lp.size;
                }
            } else if order.stop_loss.is_some() {
                order.avg_sl_price = lp.price;
                order.size = lp.size;
            } else if order.take_profit.is_some() {
                order.avg_tp_price = lp.price;
                order.size = lp.size;
            }
            order.fee += lp.fee;
            order.status = OrderStatus::Completed;
            self.save(order).await;
        }
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
        // if let Some(order_id) = &id {
        //     if let Some(exchange) = exchange {
        //         trace!("Sync order with id: '{order_id}'");
        //         if let Ok(Some(order)) = self.interactor_client.get_order(exchange, order_id).await {
        //             let status = if let Some(existing_order) = self.repository
        //                 .get(
        //                     id.clone(),
        //                     simulation_id,
        //                     Some(exchange),
        //                     market_type,
        //                     target,
        //                     source,
        //                     status.clone(),
        //                     side,
        //                     order_type,
        //                 )
        //                 .await
        //                 .unwrap().first() {
        //                 existing_order.status.clone()
        //             } else {
        //                 OrderStatus::InProgress
        //             };
        //             if !status.is_finished() {
        //                 self.repository.save(order).await
        //                     .expect("Error during order sync");
        //             }
        //         }
        //     }
        // }
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
