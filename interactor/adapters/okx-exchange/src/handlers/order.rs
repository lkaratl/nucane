use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{from_value, Value};
use tracing::info;

use domain_model::{Currency, CurrencyPair, Exchange, LP, MarginMode, Order, OrderMarketType, OrderStatus, OrderType, Side, Size, Trigger};
use eac::enums;
use eac::enums::{OrdState, OrdType, TdMode};
use eac::rest::OrderDetailsResponse;
use eac::websocket::{Action, Channel, WsMessageHandler};
use storage_core_api::StorageApi;

pub enum OrderInfo {
    Order(Order),
    LP(LP),
}

pub struct OrderHandler<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> OrderHandler<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }
}

#[async_trait]
impl<S: StorageApi> WsMessageHandler for OrderHandler<S> {
    type Type = Vec<OrderInfo>;

    async fn convert_data(
        &mut self,
        _arg: Channel,
        _action: Option<Action>,
        data: Vec<Value>,
    ) -> Option<Self::Type> {
        info!("Retrieved massage with raw payload: {}", serde_json::to_string_pretty(&data).unwrap());
        let mut orders = Vec::new();
        for item in data {
            let order_details: OrderDetailsResponse = from_value(item).unwrap();
            if !order_details.cl_ord_id.is_empty() {
                let status = match order_details.state {
                    OrdState::Canceled => OrderStatus::Canceled,
                    OrdState::Live => OrderStatus::InProgress,
                    OrdState::PartiallyFilled => OrderStatus::InProgress,
                    OrdState::Filled => {
                        if order_details.sl_trigger_px.is_none() && order_details.tp_trigger_px.is_none() {
                            OrderStatus::Completed
                        } else {
                            OrderStatus::InProgress
                        }
                    }
                };
                let pair = {
                    let mut inst_id = order_details.inst_id.split('-');
                    CurrencyPair {
                        target: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                        source: Currency::from_str(inst_id.next().unwrap()).unwrap(),
                    }
                };
                let margin_ccy = order_details.ccy;
                let market_type = {
                    match order_details.td_mode {
                        TdMode::Cross => OrderMarketType::Margin(MarginMode::Cross(Currency::from_str(&margin_ccy).unwrap())),
                        TdMode::Isolated => OrderMarketType::Margin(MarginMode::Isolated),
                        TdMode::Cash => OrderMarketType::Spot,
                    }
                };
                let side = match order_details.side {
                    enums::Side::Buy => Side::Buy,
                    enums::Side::Sell => Side::Sell,
                };
                let order_type = match order_details.ord_type {
                    OrdType::Market => OrderType::Market,
                    OrdType::Limit => OrderType::Limit(order_details.px.unwrap()),
                    order_type => panic!("Unsupported order type: {order_type:?}"),
                };
                let size = match order_details.ord_type {
                    OrdType::Market => match order_details.tgt_ccy.as_str() {
                        "quote_ccy" => Size::Source(order_details.sz),
                        "base_ccy" => Size::Target(order_details.sz),
                        _ => match side {
                            Side::Buy => Size::Source(order_details.sz),
                            Side::Sell => Size::Target(order_details.sz)
                        },
                    },
                    OrdType::Limit => Size::Target(order_details.sz),
                    order_type => panic!("Unsupported order type: {order_type:?}"),
                };
                let stop_loss =
                    if order_details.sl_trigger_px.is_some() && order_details.sl_ord_px.is_some() {
                        let sl_order_px = order_details.sl_ord_px.unwrap();
                        let sl_order_px = if sl_order_px == -1. {
                            OrderType::Market
                        } else {
                            OrderType::Limit(sl_order_px)
                        };
                        Trigger::new(
                            order_details.sl_trigger_px.unwrap(),
                            sl_order_px,
                        )
                    } else {
                        None
                    };
                let take_profit =
                    if order_details.tp_trigger_px.is_some() && order_details.tp_ord_px.is_some() {
                        let tp_order_px = order_details.sl_ord_px.unwrap();
                        let tp_order_px = if tp_order_px == -1. {
                            OrderType::Market
                        } else {
                            OrderType::Limit(tp_order_px)
                        };
                        Trigger::new(
                            order_details.tp_trigger_px.unwrap(),
                            tp_order_px,
                        )
                    } else {
                        None
                    };
                let order = if let Some(7) = order_details.source {
                    OrderInfo::LP(LP {
                        id: order_details.tag,
                        price: order_details.avg_px,
                        size,
                    })
                } else {
                    OrderInfo::Order(Order {
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
                        fee: order_details.fee.abs(),
                        avg_fill_price: order_details.avg_px,
                        stop_loss,
                        avg_sl_price: 0.,
                        take_profit,
                        avg_tp_price: 0.,
                    })
                };
                orders.push(order);
            }
        }
        Some(orders)
    }

    async fn handle(&mut self, message: Self::Type) {
        for order in message {
            match order {
                OrderInfo::Order(order) => self.storage_client.save_order(order).await.unwrap(),
                OrderInfo::LP(lp) => self.storage_client.save_lp(lp).await.unwrap(),
            }
        }
    }
}
