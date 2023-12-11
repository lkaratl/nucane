use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::bybit::enums::{Category, OrderCancelType, OrderStatus, OrderTimeInForce, OrderType, Side};
use crate::bybit::parser::ts_milliseconds;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailsResponse {
    pub category: Category,
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
    pub cum_exec_qty: String,
    pub cum_exec_value: String,
    pub cum_exec_fee: String,
    pub fee_currency: String,
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleResponse {
    pub start: u64,
    pub end: u64,
    pub timestamp: u64,
    pub interval: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub open: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub close: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub high: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub low: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub volume: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub turnover: f64,
    pub confirm: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TickerResponse {
    pub symbol: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub last_price: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub high_price24h: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub low_price24h: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub prev_price24h: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub price24h_pcnt: f64,
}
