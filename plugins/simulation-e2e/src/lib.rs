use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use domain_model::Size::Source;
use domain_model::{
    Action, CreateOrder, Currency, CurrencyPair, Exchange, InstrumentId, MarketType, OrderAction,
    OrderActionType, OrderMarketType, OrderStatus, OrderType, PluginId, Side, Tick, Timeframe,
};
use strategy_api::{utils, Strategy, StrategyApi};

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn load() -> Box<dyn Strategy> {
    Box::<SimulationE2EStrategy>::default()
}

const STRATEGY_NAME: &str = "simulation-e2e";
const STRATEGY_VERSION: i64 = 1;
const PARAMETER_NAME: &str = "test-parameter";
const LOGGING_LEVEL: &str = "INFO";

#[derive(Clone)]
pub struct SimulationE2EStrategy {
    executed: bool,
    order_id: Option<String>,
}

impl Default for SimulationE2EStrategy {
    fn default() -> Self {
        utils::init_logger(
            &format!("{STRATEGY_NAME}-{STRATEGY_VERSION}"),
            LOGGING_LEVEL,
        );
        Self {
            executed: false,
            order_id: None,
        }
    }
}

#[async_trait]
impl Strategy for SimulationE2EStrategy {
    fn tune(&mut self, config: &HashMap<String, String>) {
        config.get(PARAMETER_NAME).unwrap().to_string();
    }

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
            self.order_id = Some(utils::string_id());
            let plugin_id = PluginId::new(&self.name(), self.version());
            return vec![
                // todo hide it to factory or builder
                Action::OrderAction(OrderAction {
                    id: Uuid::new_v4(),
                    simulation_id: None,
                    plugin_id,
                    timestamp: Utc::now(),
                    status: OrderStatus::Created,
                    exchange: Exchange::OKX,
                    order: OrderActionType::CreateOrder(CreateOrder {
                        id: self.order_id.clone().unwrap(),
                        pair: CurrencyPair {
                            target: Currency::BTC,
                            source: Currency::USDT,
                        },
                        market_type: OrderMarketType::Spot,
                        order_type: OrderType::Limit(tick.price * 0.9),
                        side: Side::Buy,
                        size: Source(10.0),
                        stop_loss: None,
                        take_profit: None,
                    }),
                }),
            ];
        } else if self.order_id.is_some() {
            let orders = api
                .storage_client
                .get_orders(
                    self.order_id.clone(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            if let Ok(orders) = orders {
                if let Some(order) = orders.first() {
                    if order.status == OrderStatus::Completed {
                        info!("Successfully Completed order with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                order.id, order.market_type, order.pair.target, order.pair.source, order.order_type);

                        let btc_positions = api
                            .storage_client
                            .get_positions(Some(Exchange::OKX), Some(Currency::BTC), None)
                            .await
                            .unwrap();
                        let btc_position = btc_positions.first().unwrap();
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
                    let moving_average = api
                        .indicators
                        .moving_average(&instrument_id, Timeframe::ThirtyM, 7)
                        .await;
                    info!("Moving AVG: {}", moving_average);
                }
            }
        }
        Vec::new()
    }
}
