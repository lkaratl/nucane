use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::bybit::parser::ts_milliseconds;

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceLimit {
    inst_id: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    buy_lmt: f64,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    sell_lmt: f64,
    #[serde(deserialize_with = "ts_milliseconds")]
    ts: DateTime<Utc>,
}
