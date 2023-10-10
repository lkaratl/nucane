use chrono::{Duration, Utc};
use tracing::info;
use uuid::Uuid;

use domain_model::drawing::{Color, Icon, Line, LineStyle, Point};
use domain_model::Size::Source;
use domain_model::{
    Action, CreateOrder, CurrencyPair, Exchange, InstrumentId, MarketType, OrderAction,
    OrderActionType, OrderMarketType, OrderStatus, OrderType, PluginId, Side, Tick, Timeframe,
    Trigger,
};
use strategy_api::{utils, Strategy, StrategyApi};

#[derive(Clone)]
pub struct E2EStrategy {
    executed_once: bool,
    limit_order_id: Option<String>,
    limit_order_with_sl_tp_id: Option<String>,
    market_order_id: Option<String>,
}

impl Default for E2EStrategy {
    fn default() -> Self {
        let strategy = Self {
            executed_once: false,
            limit_order_id: None,
            limit_order_with_sl_tp_id: None,
            market_order_id: None,
        };
        utils::init_logger(
            &format!("{}-{}", strategy.name(), strategy.version()),
            "INFO",
        );
        info!("Create strategy");
        strategy
    }
}

impl E2EStrategy {
    pub async fn handle_tick(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action> {
        info!("handle"); // todo remove
        if !self.executed_once {
            self.executed_once = true;

            // info!("check indicators");
            // self.check_indicators(tick, api).await;

            info!("create drawings"); // todo remove
            self.create_drawings(tick, api).await;

            info!("create order actions"); // todo remove
            self.create_order_actions(tick)
        } else {
            info!("check orders"); // todo remove
            self.check_orders(api).await;
            Vec::new()
        }
    }

    async fn check_indicators(&self, tick: &Tick, api: &StrategyApi) {
        self.check_moving_avg(tick.instrument_id.pair, api).await;
    }

    async fn check_moving_avg(&self, pair: CurrencyPair, api: &StrategyApi) {
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair,
        };
        let moving_average = api
            .indicators
            .moving_average(&instrument_id, Timeframe::ThirtyM, 7)
            .await;
        info!("Moving AVG: {}", moving_average);
    }

    async fn create_drawings(&self, tick: &Tick, api: &StrategyApi) {
        let point = Point::new(
            tick.instrument_id.clone(),
            tick.simulation_id,
            "Check Point",
            Icon::Circle.into(),
            Color::Green.into(),
            "This point created only for test purposes"
                .to_string()
                .into(),
            (tick.timestamp, tick.price).into(),
        );
        let _ = api.storage_client.save_point(point).await;

        let line = Line::new(
            tick.instrument_id.clone(),
            tick.simulation_id,
            "Check Line",
            LineStyle::Dashed.into(),
            Color::Green.into(),
            (tick.timestamp - Duration::hours(4), tick.price * 0.9).into(),
            (tick.timestamp, tick.price).into(),
        );
        let _ = api.storage_client.save_line(line).await;
    }

    fn create_order_actions(&mut self, tick: &Tick) -> Vec<Action> {
        vec![
            self.create_limit_order_action(tick),
            self.create_limit_order_with_sl_tp_action(tick),
            self.create_market_order_action(tick),
        ]
    }

    fn create_limit_order_action(&mut self, tick: &Tick) -> Action {
        let limit_order_action =
            self.build_limit_order_action(tick.instrument_id.pair, tick.price * 0.9);
        let order_id = match &limit_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.limit_order_id = order_id;
        limit_order_action
    }

    fn create_limit_order_with_sl_tp_action(&mut self, tick: &Tick) -> Action {
        let limit_order_with_sl_tp_action =
            self.build_limit_order_action_with_sl_tp(tick.instrument_id.pair, tick.price * 0.9);
        let order_id = match &limit_order_with_sl_tp_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.limit_order_with_sl_tp_id = order_id;
        limit_order_with_sl_tp_action
    }

    fn create_market_order_action(&mut self, tick: &Tick) -> Action {
        let market_order_action = self.build_market_order_action(tick.instrument_id.pair);
        let order_id = match &market_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.market_order_id = order_id;
        market_order_action
    }

    fn build_limit_order_action(&self, pair: CurrencyPair, limit: f64) -> Action {
        Action::OrderAction(OrderAction {
            id: Uuid::new_v4(),
            simulation_id: None,
            plugin_id: self.plugin_id(),
            timestamp: Utc::now(),
            status: OrderStatus::Created,
            exchange: Exchange::OKX,
            order: OrderActionType::CreateOrder(CreateOrder {
                id: utils::string_id(),
                pair,
                market_type: OrderMarketType::Spot,
                order_type: OrderType::Limit(limit),
                side: Side::Buy,
                size: Source(10.0),
                stop_loss: None,
                take_profit: None,
            }),
        })
    }

    fn build_limit_order_action_with_sl_tp(&self, pair: CurrencyPair, limit: f64) -> Action {
        Action::OrderAction(OrderAction {
            id: Uuid::new_v4(),
            simulation_id: None,
            plugin_id: self.plugin_id(),
            timestamp: Utc::now(),
            status: OrderStatus::Created,
            exchange: Exchange::OKX,
            order: OrderActionType::CreateOrder(CreateOrder {
                id: utils::string_id(),
                pair,
                market_type: OrderMarketType::Spot,
                order_type: OrderType::Limit(limit),
                side: Side::Buy,
                size: Source(10.0),
                stop_loss: Trigger::new(limit * 0.5, limit * 0.4),
                take_profit: Trigger::new(limit * 2.0, limit * 2.1),
            }),
        })
    }

    fn build_market_order_action(&self, pair: CurrencyPair) -> Action {
        Action::OrderAction(OrderAction {
            id: Uuid::new_v4(),
            simulation_id: None,
            plugin_id: self.plugin_id(),
            timestamp: Utc::now(),
            status: OrderStatus::Created,
            exchange: Exchange::OKX,
            order: OrderActionType::CreateOrder(CreateOrder {
                id: utils::string_id(),
                pair,
                market_type: OrderMarketType::Spot,
                order_type: OrderType::Market,
                side: Side::Buy,
                size: Source(10.0),
                stop_loss: None,
                take_profit: None,
            }),
        })
    }

    fn plugin_id(&self) -> PluginId {
        PluginId::new(&self.name(), self.version())
    }

    async fn check_orders(&mut self, api: &StrategyApi) {
        self.check_limit_order(api).await;
        self.check_limit_order_with_sl_tp(api).await;
        self.check_market_order(api).await;
    }

    async fn check_limit_order(&mut self, api: &StrategyApi) {
        if self.limit_order_id.is_some() {
            let orders = api
                .storage_client
                .get_orders(
                    self.limit_order_id.clone(),
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
                if let Some(limit_order) = orders.first() {
                    if limit_order.status == OrderStatus::Completed {
                        info!("Successfully complete limit order with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                            limit_order.id, limit_order.market_type, limit_order.pair.target, limit_order.pair.source, limit_order.order_type);
                        let positions = api
                            .storage_client
                            .get_positions(Some(Exchange::OKX), Some(limit_order.pair.target), None)
                            .await
                            .unwrap();
                        let position = positions.first().unwrap();
                        assert_ne!(position.size, 0.0);
                        self.limit_order_id = None;
                    } else if limit_order.status == OrderStatus::InProgress {
                        info!("Limit order with id: {} InProgress", limit_order.id);
                    }
                }
            }
        }
    }

    async fn check_limit_order_with_sl_tp(&mut self, api: &StrategyApi) {
        if self.limit_order_with_sl_tp_id.is_some() {
            let orders = api
                .storage_client
                .get_orders(
                    self.limit_order_with_sl_tp_id.clone(),
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
                if let Some(limit_order_with_tp_sl) = orders.first() {
                    assert!(limit_order_with_tp_sl.stop_loss.is_some());
                    assert!(limit_order_with_tp_sl.take_profit.is_some());

                    if limit_order_with_tp_sl.status == OrderStatus::Completed {
                        info!("Successfully complete limit order with SL and TP with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                            limit_order_with_tp_sl.id, limit_order_with_tp_sl.market_type, limit_order_with_tp_sl.pair.target, limit_order_with_tp_sl.pair.source, limit_order_with_tp_sl.order_type);
                        let positions = api
                            .storage_client
                            .get_positions(
                                Some(Exchange::OKX),
                                Some(limit_order_with_tp_sl.pair.target),
                                None,
                            )
                            .await
                            .unwrap();
                        let position = positions.first().unwrap();
                        assert_ne!(position.size, 0.0);
                        self.limit_order_id = None;
                    } else if limit_order_with_tp_sl.status == OrderStatus::InProgress {
                        info!(
                            "Limit order with SL and TP with id: {} InProgress",
                            limit_order_with_tp_sl.id
                        );
                    }
                }
            }
        }
    }

    async fn check_market_order(&mut self, api: &StrategyApi) {
        if self.market_order_id.is_some() {
            let orders = api
                .storage_client
                .get_orders(
                    self.market_order_id.clone(),
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
                if let Some(market_order) = orders.first() {
                    if market_order.status == OrderStatus::Completed {
                        info!("Successfully complete market order with id: {}, market type: {:?}, target currency: {}, source currency: {}",
                            market_order.id, market_order.market_type, market_order.pair.target, market_order.pair.source);
                        let positions = api
                            .storage_client
                            .get_positions(
                                Some(Exchange::OKX),
                                Some(market_order.pair.target),
                                None,
                            )
                            .await
                            .unwrap();
                        let position = positions.first().unwrap();
                        assert_ne!(position.size, 0.0);
                        self.market_order_id = None;
                    } else if market_order.status == OrderStatus::InProgress {
                        info!("Market order with id: {} InProgress", market_order.id);
                    }
                }
            }
        }
    }
}
