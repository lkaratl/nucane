use std::collections::HashMap;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;
use async_trait::async_trait;

use domain_model::{Action, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Order, OrderAction, OrderActionType, OrderMarketType, Side, OrderStatus, OrderType, Tick, Timeframe};
use domain_model::MarginMode::{Cross, Isolated};
use strategy_api::{Strategy, StrategyApi, utils};

#[no_mangle]
pub extern fn load() -> Box<dyn Strategy> {
    Box::<SimulationE2EStrategy>::default()
}

#[derive(Clone, Default)]
pub struct SimulationE2EStrategy {
    executed: bool,
    order_id: Option<String>,
}

const STRATEGY_NAME: &str = "simulation-e2e";
const STRATEGY_VERSION: &str = "1.0";
const PARAMETER_NAME: &str = "test-parameter";

#[async_trait]
impl Strategy for SimulationE2EStrategy {
    fn tune(&mut self, config: &HashMap<String, String>) {
        config.get(PARAMETER_NAME).unwrap().to_string();
    }

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
                target: Currency::BTC,
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
                                    target: Currency::BTC,
                                    source: Currency::USDT,
                                },
                                market_type: OrderMarketType::Margin(Cross),
                                order_type: OrderType::Limit(tick.price),
                                side: Side::Buy,
                                size: 1000.0,
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

                        let btc_positions = api.storage.get_positions(Some(Exchange::OKX), Some(Currency::BTC), None)
                            .await
                            .unwrap();
                        let btc_position = btc_positions
                            .first()
                            .unwrap();
                        assert_ne!(btc_position.size, 0.0);

                        self.order_id = None;
                    } else if order.status == OrderStatus::InProgress {
                        info!("Order with id: {} InProgress", order.id);
                    }

                    let instrument_id = InstrumentId {
                        exchange: Exchange::OKX,
                        market_type: MarketType::Spot,
                        pair: CurrencyPair {
                            target: Currency::BTC,
                            source: Currency::USDT,
                        },
                    };
                    let moving_average = api.indicators.moving_average(&instrument_id, Timeframe::ThirtyM, 7).await;
                    info!("Moving AVG: {}", moving_average);
                }
            }
        }
        Vec::new()
    }
}