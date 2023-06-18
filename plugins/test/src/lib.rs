use chrono::Utc;
use uuid::Uuid;
use async_trait::async_trait;
use tracing::info;

use domain_model::{Action, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, OrderAction, OrderActionType, OrderMarketType, Side, OrderStatus, OrderType, Tick};
use strategy_api::{Strategy, StrategyApi, utils};

#[no_mangle]
pub extern fn load() -> Box<dyn Strategy> {
    Box::<TestStrategy>::default()
}

#[derive(Clone, Default)]
pub struct TestStrategy {
    pub executed: bool,
    pub order_id: Option<String>,
}

const STRATEGY_NAME: &str = "test";
const STRATEGY_VERSION: &str = "1.0";

#[async_trait]
impl Strategy for TestStrategy {
    fn name(&self) -> String {
        STRATEGY_NAME.to_string()
    }

    fn version(&self) -> String {
        STRATEGY_VERSION.to_string()
    }

    fn subscriptions(&self) -> Vec<InstrumentId> {
        vec![InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::FTM,
                source: Currency::USDT,
            },
        }]
    }

    async fn execute(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action> {
        if !self.executed {
            self.executed = true;
            self.order_id = Some(utils::string_id());
            return vec![
                // todo hide it to factory or builder
                Action::OrderAction(
                    OrderAction {
                        id: Uuid::new_v4(),
                        simulation_id: None,
                        strategy_name: self.name(),
                        strategy_version: self.version(),
                        timestamp: Utc::now(),
                        status: OrderStatus::Created,
                        exchange: Exchange::OKX,
                        order: OrderActionType::CreateOrder(
                            CreateOrder {
                                id: self.order_id.clone().unwrap(),
                                pair: CurrencyPair {
                                    target: Currency::FTM,
                                    source: Currency::USDT,
                                },
                                market_type: OrderMarketType::Spot,
                                order_type: OrderType::Market,
                                side: Side::Buy,
                                size: 10.0,
                            }
                        ),
                    }
                )
            ];
        } else if self.order_id.is_some() {
            let orders = api.storage.get_orders(self.order_id.clone(),
                                                None,
                                                None,
                                                None,
                                                None,
                                                None,
                                                None,
                                                None).await;
            if let Ok(orders) = orders {
                if let Some(order) = orders.first() {
                    if order.status == OrderStatus::Completed {
                        info!("Successfully Completed order with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                order.id, order.market_type, order.pair.target, order.pair.source, order.order_type);
                    } else if order.status == OrderStatus::InProgress {
                        info!("Order with id: {} InProgress", order.id);
                    }
                }
            }
        }
        Vec::new()
    }
}
