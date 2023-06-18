use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use balance::*;
pub use positions::*;

use crate::okx::parser::ts_milliseconds;

mod balance;
mod positions;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(deserialize_with = "ts_milliseconds")]
    pub u_time: DateTime<Utc>,
    pub total_eq: String,
    pub iso_eq: String,
    pub adj_eq: String,
    pub ord_froz: String,
    pub imr: String,
    pub mmr: String,
    pub notional_usd: String,
    pub mgn_ratio: String,
    pub details: Vec<Asset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub avail_bal: f64,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub avail_eq: f64,
    pub ccy: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub cash_bal: f64,
    #[serde(deserialize_with = "ts_milliseconds")]
    pub u_time: DateTime<Utc>,
    pub dis_eq: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub eq: f64,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub eq_usd: f64,
    pub frozen_bal: String,
    pub interest: String,
    pub iso_eq: String,
    pub liab: String,
    pub max_loan: String,
    pub mgn_ratio: String,
    pub notional_lever: String,
    pub ord_frozen: String,
    pub upl: String,
    pub cross_liab: String,
    pub iso_liab: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub coin_usd_price: f64,
    pub stgy_eq: String,
    pub spot_in_use_amt: String,
    pub iso_upl: String,
}
