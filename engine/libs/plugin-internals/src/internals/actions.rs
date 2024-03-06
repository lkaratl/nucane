use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain_model::{Action, CancelOrder, CreateOrder, CurrencyPair, Exchange, OrderAction, OrderActionType, OrderMarketType, OrderStatus, OrderType, PluginId, Side, Size, Trigger};
use plugin_api::{ActionsInternalApi, utils};

pub struct DefaultActionInternals {
    simulation_id: Option<Uuid>,
    plugin_id: PluginId,
}

impl DefaultActionInternals {
    pub fn new(simulation_id: Option<Uuid>, plugin_id: PluginId) -> Self {
        Self {
            simulation_id,
            plugin_id,
        }
    }
}

#[async_trait]
impl ActionsInternalApi for DefaultActionInternals {
    fn create_order_action(
        &self,
        exchange: Exchange,
        pair: CurrencyPair,
        market_type: OrderMarketType,
        order_type: OrderType,
        size: Size,
        side: Side,
        stop_loss: Option<Trigger>,
        take_profit: Option<Trigger>,
    ) -> Action {
        Action::OrderAction(OrderAction {
            id: Uuid::new_v4(),
            simulation_id: self.simulation_id,
            plugin_id: self.plugin_id.clone(),
            timestamp: Utc::now(),
            status: OrderStatus::Created,
            exchange,
            order: OrderActionType::CreateOrder(CreateOrder {
                id: utils::string_id(),
                pair,
                market_type,
                order_type,
                side,
                size,
                stop_loss,
                take_profit,
            }),
        })
    }

    fn cancel_order_action(
        &self,
        exchange: Exchange,
        pair: CurrencyPair,
        market_type: OrderMarketType,
        order_id: &str) -> Action {
        Action::OrderAction(OrderAction {
            id: Uuid::new_v4(),
            simulation_id: self.simulation_id,
            plugin_id: self.plugin_id.clone(),
            timestamp: Utc::now(),
            status: OrderStatus::Created,
            exchange,
            order: OrderActionType::CancelOrder(CancelOrder {
                id: order_id.into(),
                pair,
                market_type,
            }),
        })
    }
}
