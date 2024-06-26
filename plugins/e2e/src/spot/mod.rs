use std::sync::Arc;

use tracing::info;

use domain_model::{Action, Exchange, OrderActionType, OrderStatus, Tick, Trigger};
use domain_model::OrderMarketType::Spot;
use domain_model::OrderType::{Limit, Market};
use domain_model::Side::Buy;
use domain_model::Size::Source;
use plugin_api::PluginInternalApi;

use crate::plugin::E2EPlugin;

#[allow(dead_code)]
impl E2EPlugin {
    pub async fn handle_spot_tick(&mut self, tick: &Tick, api: Arc<dyn PluginInternalApi>) -> Vec<Action> {
        if !self.state.spot_executed_once {
            self.state.spot_executed_once = true;
            info!("Create spot actions");
            self.create_order_actions(tick, api).await
        } else {
            info!("Check spot orders");
            self.check_spot_orders(api).await;
            Vec::new()
        }
    }

    async fn create_order_actions(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Vec<Action> {
        vec![
            // self.create_spot_market_order_action(tick, api.clone()).await,
            // self.create_spot_limit_order_action(tick, api.clone()).await,
            self.create_spot_limit_order_with_sl_tp_action(tick, api.clone())
                .await,
            self.cancel_order(tick, api.clone(), &self.state.spot_limit_order_with_sl_tp_id.clone().unwrap())
        ]
    }

    async fn create_spot_market_order_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let market_order_action = api.actions().create_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            Spot,
            Market,
            Source(100.0),
            Buy,
            None,
            None,
        );
        let order_id = match &market_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.spot_market_order_id = order_id;
        market_order_action
    }

    async fn create_spot_limit_order_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let limit_order_action = api.actions().create_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            Spot,
            Limit(tick.price * 0.95),
            Source(100.0),
            Buy,
            None,
            None,
        );
        let order_id = match &limit_order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.spot_limit_order_id = order_id;
        limit_order_action
    }

    async fn create_spot_limit_order_with_sl_tp_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let limit = tick.price * 0.95;
        let limit_order_with_sl_tp_action = api.actions().create_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            Spot,
            Limit(limit),
            Source(100.0),
            Buy,
            Trigger::new(limit * 0.9, Market),
            Trigger::new(limit * 1.1, Limit(limit * 1.11)),
        );
        let order_id = match &limit_order_with_sl_tp_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.spot_limit_order_with_sl_tp_id = order_id;
        limit_order_with_sl_tp_action
    }

    fn cancel_order(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
        order_id: &str,
    ) -> Action {
        api.actions().cancel_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            order_id,
        )
    }

    async fn check_spot_orders(&mut self, api: Arc<dyn PluginInternalApi>) {
        self.check_spot_market_order(api.clone()).await;
        self.check_spot_limit_order(api.clone()).await;
        self.check_spot_limit_order_with_sl_tp(api.clone()).await;
    }

    async fn check_spot_limit_order(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(limit_order_id) = &self.state.spot_limit_order_id {
            let limit_order = api.orders().get_order_by_id(Exchange::OKX, limit_order_id).await;
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

                    self.state.spot_limit_order_id = None;
                } else if limit_order.status == OrderStatus::InProgress {
                    info!("Limit order with id: {} InProgress", limit_order.id);
                }
            }
        }
    }

    async fn check_spot_limit_order_with_sl_tp(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(limit_order_with_sl_tp_id) =
            &self.state.spot_limit_order_with_sl_tp_id
        {
            let limit_order_with_sl_tp = api
                .orders()
                .get_order_by_id(Exchange::OKX, limit_order_with_sl_tp_id)
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

                    self.state.spot_limit_order_with_sl_tp_id = None;
                } else if limit_order_with_sl_tp.status == OrderStatus::InProgress {
                    info!(
                        "Limit order with SL and TP with id: {} InProgress",
                        limit_order_with_sl_tp.id
                    );
                }
            }
        }
    }

    async fn check_spot_market_order(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(market_order_id) = &self.state.spot_market_order_id {
            let market_order = api.orders().get_order_by_id(Exchange::OKX, market_order_id).await;
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

                    self.state.spot_market_order_id = None;
                } else if market_order.status == OrderStatus::InProgress {
                    info!("Market order with id: {} InProgress", market_order.id);
                }
            }
        }
    }
}
