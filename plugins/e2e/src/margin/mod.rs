use std::sync::Arc;

use tracing::info;

use domain_model::{Action, Exchange, OrderActionType, OrderStatus, Tick, Trigger};
use domain_model::MarginMode::Isolated;
use domain_model::OrderMarketType::Margin;
use domain_model::OrderType::Market;
use domain_model::Side::Sell;
use domain_model::Size::Target;
use plugin_api::PluginInternalApi;

use crate::plugin::E2EPlugin;

#[allow(dead_code)]
impl E2EPlugin {
    pub async fn handle_margin_tick(&mut self, tick: &Tick, api: Arc<dyn PluginInternalApi>) -> Vec<Action> {
        if !self.state.margin_executed_once {
            self.state.margin_executed_once = true;
            info!("Create margin actions");
            self.create_margin_order_actions(tick, api).await
        } else {
            info!("Check margin orders");
            self.check_margin_orders(api).await;
            Vec::new()
        }
    }

    async fn create_margin_order_actions(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Vec<Action> {
        vec![
            // self.create_margin_market_order_action(tick, api.clone()).await,
            self.create_margin_market_order_with_sl_tp_action(tick, api.clone()).await,
        ]
    }

    async fn create_margin_market_order_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let order_action = api.actions().create_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            Margin(Isolated),
            Market,
            Target(100.0 / tick.price),
            Sell,
            None,
            None,
        );
        let order_id = match &order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.margin_market_order_id = order_id;
        order_action
    }

    async fn create_margin_market_order_with_sl_tp_action(
        &mut self,
        tick: &Tick,
        api: Arc<dyn PluginInternalApi>,
    ) -> Action {
        let order_action = api.actions().create_order_action(
            tick.instrument_id.exchange,
            tick.instrument_id.pair,
            Margin(Isolated),
            Market,
            Target(100.0 / tick.price),
            Sell,
            Trigger::new(tick.price * 1.0005, Market),
            Trigger::new(tick.price * 0.9995, Market),
        );
        let order_id = match &order_action {
            Action::OrderAction(order_action) => match &order_action.order {
                OrderActionType::CreateOrder(create_order) => Some(create_order.id.clone()),
                _ => None,
            },
        };
        self.state.margin_market_order_id_market_sl_market_tp_id = order_id;
        order_action
    }

    async fn check_margin_orders(&mut self, api: Arc<dyn PluginInternalApi>) {
        // self.check_margin_market_order(api.clone()).await;
        self.check_margin_market_order_with_sl_tp(api.clone()).await;
    }

    async fn check_margin_market_order(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(order_id) = &self.state.margin_market_order_id {
            let order = api.orders().get_order_by_id(Exchange::OKX, order_id).await;
            if let Some(market_order) = order {
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

    async fn check_margin_market_order_with_sl_tp(&mut self, api: Arc<dyn PluginInternalApi>) {
        if let Some(order_id) = &self.state.margin_market_order_id_market_sl_market_tp_id
        {
            let order = api
                .orders()
                .get_order_by_id(Exchange::OKX, order_id)
                .await;
            if let Some(limit_order_with_sl_tp) = order {
                assert!(limit_order_with_sl_tp.stop_loss.is_some());
                assert!(limit_order_with_sl_tp.take_profit.is_some());

                if limit_order_with_sl_tp.status == OrderStatus::Completed {
                    info!("Successfully complete market order with SL and TP with id: {}, market type: {:?}, target currency: {}, source currency: {}, order type: {:?}",
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
                        "Market order with SL and TP with id: {} InProgress",
                        limit_order_with_sl_tp.id
                    );
                }
            }
        }
    }
}
