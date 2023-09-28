use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{from_value, Value};
use tracing::trace;

use domain_model::{Currency, CurrencyPair, Exchange, MarginMode, Order, OrderMarketType, OrderStatus, OrderType, Side, Size, Trigger};
use eac::enums;
use eac::enums::{OrdState, OrdType, TdMode};
use eac::rest::OrderDetailsResponse;
use eac::websocket::{Action, Channel, WsMessageHandler};
use storage_core_api::StorageApi;

pub struct OrderHandler<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> OrderHandler<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self {
            storage_client,
        }
    }
}

#[async_trait]
impl<S: StorageApi> WsMessageHandler for OrderHandler<S> {
    type Type = Vec<Order>;

    async fn convert_data(&mut self, _arg: Channel, _action: Option<Action>, data: Vec<Value>) -> Option<Self::Type> {
        trace!("Retrieved massage with raw payload: {:?}", &data);
        let mut orders = Vec::new();
        for item in data {
            let order_details: OrderDetailsResponse = from_value(item).unwrap();
            if !order_details.cl_ord_id.is_empty() {
                let status = match order_details.state {
                    OrdState::Canceled => OrderStatus::Canceled,
                    OrdState::Live => OrderStatus::InProgress,
                    OrdState::PartiallyFilled => OrderStatus::InProgress,
                    OrdState::Filled => OrderStatus::Completed
                };
                let pair = {
                    let mut inst_id = order_details.inst_id.split('-');
                    CurrencyPair {
                        target: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                        source: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                    }
                };
                let market_type = {
                    match order_details.td_mode {
                        TdMode::Cross => OrderMarketType::Margin(MarginMode::Cross),
                        TdMode::Isolated => OrderMarketType::Margin(MarginMode::Isolated),
                        TdMode::Cash => OrderMarketType::Spot
                    }
                };
                let side = match order_details.side {
                    enums::Side::Buy => Side::Buy,
                    enums::Side::Sell => Side::Sell
                };
                let order_type = match order_details.ord_type {
                    OrdType::Market => OrderType::Market,
                    OrdType::Limit => OrderType::Limit(order_details.px.unwrap()),
                    order_type => panic!("Unsupported order type: {order_type:?}")
                };
                let size = match order_details.ord_type {
                    OrdType::Market => match order_details.tgt_ccy.as_str() {
                        "quote_ccy" => Size::Source(order_details.sz),
                        "base_ccy" => Size::Target(order_details.sz),
                        _ => panic!("Empty target currency")
                    }
                    OrdType::Limit => Size::Target(order_details.sz),
                    order_type => panic!("Unsupported order type: {order_type:?}")
                };
                let stop_loss = if order_details.sl_trigger_px.is_some() &&
                    order_details.sl_ord_px.is_some() {
                    Trigger::new(order_details.sl_trigger_px.unwrap(), order_details.sl_ord_px.unwrap())
                } else {
                    None
                };
                let take_profit = if order_details.tp_trigger_px.is_some() &&
                    order_details.tp_ord_px.is_some() {
                    Trigger::new(order_details.tp_trigger_px.unwrap(), order_details.tp_ord_px.unwrap())
                } else {
                    None
                };
                let order = Order {
                    id: order_details.cl_ord_id,
                    timestamp: Utc::now(),
                    simulation_id: None,
                    status,
                    exchange: Exchange::OKX,
                    pair,
                    market_type,
                    order_type,
                    side,
                    size,
                    avg_price: order_details.avg_px,
                    stop_loss,
                    take_profit,
                };
                orders.push(order);
            }
        }
        Some(orders)
    }

    async fn handle(&mut self, message: Self::Type) {
        for order in message {
            let _ = self.storage_client.save_order(order).await;
        }
    }
}
