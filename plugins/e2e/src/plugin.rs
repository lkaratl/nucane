use std::sync::Arc;

use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use domain_model::{
    Action, CurrencyPair, Exchange, InstrumentId, MarketType, OrderActionType, OrderStatus, Side,
    Size, Tick, Timeframe, Trigger,
};
use domain_model::drawing::{Color, Icon, LineStyle};
use domain_model::OrderType::{Limit, Market};
use plugin_api::{Line, PluginApi, PluginInternalApi, Point, utils};

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    executed_once: bool,
    limit_order_id: Option<String>,
    limit_order_with_sl_tp_id: Option<String>,
    market_order_id: Option<String>,
}

pub struct E2EPlugin {
    state: State,
}

impl Default for E2EPlugin {
    fn default() -> Self {
        let plugin = Self {
            state: Default::default()
        };
        utils::init_logger(
            &format!("{}-{}", plugin.id().name, plugin.id().version),
            "INFO",
        );
        plugin
    }
}

impl E2EPlugin {
    pub fn get_state(&self) -> Value {
        serde_json::to_value(&self.state).unwrap()
    }

    pub fn set_state(&mut self, state: Value) {
        self.state = serde_json::from_value(state).unwrap()
    }

    pub async fn handle_tick(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Vec<Action> {
        if !self.state.executed_once {
            self.state.executed_once = true;
            info!("Create actions");

            // self.check_indicators(tick, api.clone()).await;
            self.create_drawings(tick, api.clone()).await;
            self.create_order_actions(tick, api).await
        } else {
            info!("Check orders");
            self.check_orders(api).await;
            Vec::new()
        }
    }

    async fn check_indicators(&self, tick: &Tick, api: Arc<dyn PluginInternalApi>) {
        self.check_moving_avg(tick.instrument_id.pair, api).await;
    }

    async fn check_moving_avg(&self, pair: CurrencyPair, api: Arc<dyn PluginInternalApi>) {
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair,
        };

        let moving_average = api
            .indicators()
            .moving_avg(&instrument_id, Timeframe::OneH, 7)
            .await;
        info!("Moving AVG: {}", moving_average);
    }

    async fn create_drawings(&self, tick: &Tick, api: Arc<dyn PluginInternalApi>) {
        let point = Point::new(
            tick.instrument_id.clone(),
            "Check Point",
            Icon::Circle.into(),
            Color::Green.into(),
            "This point created only for test purposes"
                .to_string()
                .into(),
            (tick.timestamp + Duration::days(1), tick.price).into(),
        );
        api.drawings().save_point(point).await;

        let line = Line::new(
            tick.instrument_id.clone(),
            "Check Line",
            LineStyle::Dashed.into(),
            Color::Green.into(),
            (tick.timestamp, tick.price).into(),
            (tick.timestamp + Duration::days(1), tick.price * 1.1).into(),
        );
        api.drawings().save_line(line).await;
    }

    async fn create_order_actions(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Vec<Action> {
        vec![
            self.create_limit_order_action(tick, api.clone()).await,
            self.create_limit_order_with_sl_tp_action(tick, api.clone())
                .await,
            self.create_market_order_action(tick, api.clone()).await,
        ]
    }

    async fn create_limit_order_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let limit_order_action = api.actions().create_order_action(
            tick.instrument_id.pair,
            Limit(tick.price * 0.95),
            Size::Source(10.0),
            Side::Buy,
            None,
            None,
        );
        let order_id = match &limit_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.limit_order_id = order_id;
        limit_order_action
    }

    async fn create_limit_order_with_sl_tp_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let limit = tick.price * 0.95;
        let limit_order_with_sl_tp_action = api.actions().create_order_action(
            tick.instrument_id.pair,
            Limit(limit),
            Size::Source(10.0),
            Side::Buy,
            Trigger::new(limit * 0.5, limit * 0.4),
            Trigger::new(limit * 2.0, limit * 2.1),
        );
        let order_id = match &limit_order_with_sl_tp_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.limit_order_with_sl_tp_id = order_id;
        limit_order_with_sl_tp_action
    }

    async fn create_market_order_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let market_order_action = api.actions().create_order_action(
            tick.instrument_id.pair,
            Market,
            Size::Source(10.0),
            Side::Buy,
            None,
            None,
        );
        let order_id = match &market_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.market_order_id = order_id;
        market_order_action
    }

    async fn check_orders(&mut self, api: Arc<dyn PluginInternalApi>) {
        self.check_limit_order(api.clone()).await;
        self.check_limit_order_with_sl_tp(api.clone()).await;
        self.check_market_order(api.clone()).await;
    }

    async fn check_limit_order(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(limit_order_id) = &self.state.limit_order_id {
            let limit_order = api.orders().get_order_by_id(limit_order_id).await;
            if let Some(limit_order) = limit_order {
                if limit_order.status == OrderStatus::Completed {
                    info!("Successfully complete limit order with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                            limit_order.id, limit_order.market_type, limit_order.pair.target, limit_order.pair.source, limit_order.order_type);
                    let position = api
                        .positions()
                        .get_position(limit_order.exchange, limit_order.pair.target)
                        .await;
                    assert!(position.is_some());
                    assert_ne!(position.unwrap().size, 0.0);

                    self.state.limit_order_id = None;
                } else if limit_order.status == OrderStatus::InProgress {
                    info!("Limit order with id: {} InProgress", limit_order.id);
                }
            }
        }
    }

    async fn check_limit_order_with_sl_tp(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(limit_order_with_sl_tp_id) =
            &self.state.limit_order_with_sl_tp_id
        {
            let limit_order_with_sl_tp = api
                .orders()
                .get_order_by_id(limit_order_with_sl_tp_id)
                .await;
            if let Some(limit_order_with_sl_tp) = limit_order_with_sl_tp {
                assert!(limit_order_with_sl_tp.stop_loss.is_some());
                assert!(limit_order_with_sl_tp.take_profit.is_some());

                if limit_order_with_sl_tp.status == OrderStatus::Completed {
                    info!("Successfully complete limit order with SL and TP with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
                            limit_order_with_sl_tp.id, limit_order_with_sl_tp.market_type, limit_order_with_sl_tp.pair.target, limit_order_with_sl_tp.pair.source, limit_order_with_sl_tp.order_type);
                    let position = api
                        .positions()
                        .get_position(
                            limit_order_with_sl_tp.exchange,
                            limit_order_with_sl_tp.pair.target,
                        )
                        .await;
                    assert!(position.is_some());
                    assert_ne!(position.unwrap().size, 0.0);

                    self.state.limit_order_with_sl_tp_id = None;
                } else if limit_order_with_sl_tp.status == OrderStatus::InProgress {
                    info!(
                        "Limit order with SL and TP with id: {} InProgress",
                        limit_order_with_sl_tp.id
                    );
                }
            }
        }
    }

    async fn check_market_order(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(market_order_id) = &self.state.market_order_id {
            let market_order = api.orders().get_order_by_id(market_order_id).await;
            if let Some(market_order) = market_order {
                if market_order.status == OrderStatus::Completed {
                    info!("Successfully complete market order with id: {}, market type: {:?}, target currency: {}, source currency: {}",
                            market_order.id, market_order.market_type, market_order.pair.target, market_order.pair.source);
                    let position = api
                        .positions()
                        .get_position(market_order.exchange, market_order.pair.target)
                        .await;
                    assert!(position.is_some());
                    assert_ne!(position.unwrap().size, 0.0);

                    self.state.market_order_id = None;
                } else if market_order.status == OrderStatus::InProgress {
                    info!("Market order with id: {} InProgress", market_order.id);
                }
            }
        }
    }
}
