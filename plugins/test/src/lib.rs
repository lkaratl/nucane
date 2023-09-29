use async_trait::async_trait;
use chrono::Utc;
use tracing::{error, info};
use uuid::Uuid;

use domain_model::{Action, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, OrderAction, OrderActionType, OrderMarketType, OrderStatus, OrderType, Side, Tick, Trigger};
use domain_model::Size::Source;
use strategy_api::{Strategy, StrategyApi, utils};

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern fn load() -> Box<dyn Strategy> {
    Box::<TestStrategy>::default()
}

const STRATEGY_NAME: &str = "test";
const STRATEGY_VERSION: i64 = 1;
const LOGGING_LEVEL: &str = "INFO";

#[derive(Clone)]
pub struct TestStrategy {
    pub executed: bool,
    pub spot_market_buy: Option<String>,
    pub spot_limit_buy_with_sl_and_tp: Option<String>,
}

impl Default for TestStrategy {
    fn default() -> Self {
        utils::init_logger(&format!("{STRATEGY_NAME}-{STRATEGY_VERSION}"), LOGGING_LEVEL);
        Self {
            executed: false,
            spot_market_buy: None,
            spot_limit_buy_with_sl_and_tp: None,
        }
    }
}

#[async_trait]
impl Strategy for TestStrategy {
    fn name(&self) -> String {
        STRATEGY_NAME.to_string()
    }

    fn version(&self) -> i64 {
        STRATEGY_VERSION
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

    async fn on_tick(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action> {
        if !self.executed {
            self.executed = true;
            self.spot_market_buy = Some(utils::string_id());
            self.spot_limit_buy_with_sl_and_tp = Some(utils::string_id());
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
                                id: self.spot_market_buy.clone().unwrap(),
                                pair: CurrencyPair {
                                    target: Currency::BTC,
                                    source: Currency::USDT,
                                },
                                market_type: OrderMarketType::Spot,
                                order_type: OrderType::Market,
                                side: Side::Buy,
                                size: Source(10.0),
                                stop_loss: None,
                                take_profit: None,
                            }
                        ),
                    }
                ),
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
                                id: self.spot_limit_buy_with_sl_and_tp.clone().unwrap(),
                                pair: CurrencyPair {
                                    target: Currency::BTC,
                                    source: Currency::USDT,
                                },
                                market_type: OrderMarketType::Spot,
                                order_type: OrderType::Limit(tick.price * 0.9),
                                side: Side::Buy,
                                size: Source(10.0),
                                stop_loss: Trigger::new(tick.price * 0.5, tick.price * 0.4),
                                take_profit: Trigger::new(tick.price * 2.0, tick.price * 2.1),
                            }
                        ),
                    }
                ),
            ];
        } else {
            self.check_orders(api).await;
            Vec::new()
        }
    }
}

impl TestStrategy {
    async fn check_orders(&mut self, api: &StrategyApi) {
        if self.spot_market_buy.is_some() {
            let orders = api.storage_client.get_orders(self.spot_market_buy.clone(),
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
                        info!("Spot market buy order successfully filled");
                    } else if order.status == OrderStatus::InProgress {
                        info!("Spot market buy order InProgress");
                    }
                    self.spot_market_buy = None;
                }
            } else {
                error!("Error: {:?}", orders.err());
            }
        }
        if self.spot_limit_buy_with_sl_and_tp.is_some() {
            let orders = api.storage_client.get_orders(self.spot_limit_buy_with_sl_and_tp.clone(),
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
                        info!("Spot limit buy order with sl and tp successfully filled");
                    } else if order.status == OrderStatus::InProgress {
                        info!("Spot limit buy order with sl and tp InProgress");
                    }
                    self.spot_limit_buy_with_sl_and_tp = None;
                }
            }
        }
    }
}
