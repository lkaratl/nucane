use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain_model::{
    Action, CreateOrder, CurrencyPair, Exchange, OrderAction, OrderActionType, OrderMarketType,
    OrderStatus, OrderType, PluginId, Side, Size, Trigger,
};
use plugin_api::{utils, ActionsInternalApi};

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
        pair: CurrencyPair,
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
            exchange: Exchange::OKX,
            order: OrderActionType::CreateOrder(CreateOrder {
                id: utils::string_id(),
                pair,
                market_type: OrderMarketType::Spot,
                order_type,
                side,
                size,
                stop_loss,
                take_profit,
            }),
        })
    }
}
