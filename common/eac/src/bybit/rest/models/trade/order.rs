use chrono::{DateTime, Utc};
use http::Method;
use serde::{Deserialize, Serialize};

use crate::bybit::enums::{Category, OrderCancelType, OrderFilter, OrderStatus, OrderTimeInForce, OrderType, Side};
use crate::bybit::parser::ts_milliseconds;

use super::super::Request;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    pub category: Category,
    pub symbol: String,
    pub is_leverage: Option<u8>,
    pub side: Side,
    pub order_type: OrderType,
    pub qty: f64,
    pub price: Option<f64>,
    pub trigger_direction: Option<u8>,
    pub order_filter: Option<OrderFilter>,
    pub trigger_price: Option<String>,
    pub trigger_by: Option<String>,
    pub order_iv: Option<String>,
    pub time_in_force: Option<OrderTimeInForce>,
    pub position_idx: Option<u8>,
    pub order_link_id: Option<String>,
    pub take_profit: Option<String>,
    pub stop_loss: Option<String>,
    pub tp_trigger_by: Option<String>,
    pub sl_trigger_by: Option<String>,
    pub reduce_only: Option<bool>,
    pub close_on_trigger: Option<bool>,
    pub smp_type: Option<String>,
    pub mmp: Option<bool>,
    pub tpsl_mode: Option<String>,
    pub tp_limit_price: Option<String>,
    pub sl_limit_price: Option<String>,
    pub tp_order_type: Option<String>,
    pub sl_order_type: Option<String>,
}

impl PlaceOrderRequest {
    pub fn market(order_id: Option<String>, symbol: &str, category: Category, side: Side, qty: Size, is_leverage: bool) -> Self {
        let qty = match qty {
            Size::Target(qty) => {
                if side == Side::Buy {
                    panic!("Can't create order with Target size and Buy side. Please use Source size");
                }
                qty
            }
            Size::Source(qty) => {
                if side == Side::Sell {
                    panic!("Can't create order with Source size and Sell side. Please use Target size");
                }
                qty
            }
        };
        let is_leverage = if is_leverage { 1 } else { 0 }.into();
        Self {
            category,
            symbol: symbol.to_string(),
            is_leverage,
            side,
            order_type: OrderType::Market,
            qty,
            price: None,
            trigger_direction: None,
            order_filter: None,
            trigger_price: None,
            trigger_by: None,
            order_iv: None,
            time_in_force: None,
            position_idx: None,
            order_link_id: order_id,
            take_profit: None,
            stop_loss: None,
            tp_trigger_by: None,
            sl_trigger_by: None,
            reduce_only: None,
            close_on_trigger: None,
            smp_type: None,
            mmp: None,
            tpsl_mode: None,
            tp_limit_price: None,
            sl_limit_price: None,
            tp_order_type: None,
            sl_order_type: None,
        }
    }
}

pub enum Size {
    Target(f64),
    Source(f64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderResponse {
    pub order_id: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str_opt")]
    pub order_link_id: Option<String>,
}

impl Request for PlaceOrderRequest {
    const METHOD: Method = Method::POST;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/v5/order/create";
    const HAS_PAYLOAD: bool = true;
    type Response = PlaceOrderResponse;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsRequest {
    pub category: Category,
    pub symbol: Option<String>,
    pub base_coin: Option<String>,
    pub settle_coin: Option<String>,
    pub order_id: Option<String>,
    pub order_link_id: Option<String>,
    pub order_filter: Option<OrderFilter>,
    pub order_status: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdersDetailsResponse {
    pub list: Vec<OrderDetailsResponse>,
    pub next_page_cursor: String,
    pub category: Category,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsResponse {
    pub category: Option<Category>,
    pub order_id: String,
    pub order_link_id: String,
    pub is_leverage: String,
    pub block_trade_id: String,
    pub symbol: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub price: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub qty: f64,
    pub side: Side,
    pub position_idx: u8,
    pub order_status: OrderStatus,
    pub cancel_type: OrderCancelType,
    pub reject_reason: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub avg_price: f64,
    pub leaves_qty: String,
    pub leaves_value: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub cum_exec_qty: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub cum_exec_value: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub cum_exec_fee: f64,
    pub fee_currency: Option<String>,
    pub time_in_force: OrderTimeInForce,
    pub order_type: OrderType,
    pub stop_order_type: String,
    pub oco_trigger_type: Option<String>,
    pub order_iv: String,
    pub trigger_price: String,
    pub take_profit: String,
    pub stop_loss: String,
    pub tpsl_mode: Option<String>,
    pub tp_limit_price: Option<String>,
    pub sl_limit_price: Option<String>,
    pub tp_trigger_by: Option<String>,
    pub sl_trigger_by: Option<String>,
    pub trigger_direction: u8,
    pub trigger_by: String,
    pub last_price_on_created: String,
    pub reduce_only: bool,
    pub close_on_trigger: bool,
    pub place_type: String,
    pub smp_type: String,
    pub smp_group: i64,
    pub smp_order_id: String,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub updated_time: DateTime<Utc>,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub created_time: DateTime<Utc>,
}

impl Request for OrderDetailsRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/v5/order/history";
    const HAS_PAYLOAD: bool = true;
    type Response = OrdersDetailsResponse;
}
