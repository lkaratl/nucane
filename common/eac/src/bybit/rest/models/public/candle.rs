use http::Method;
use serde::{Deserialize, Serialize};

use crate::bybit::rest::Request;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesHistoryRequest {
    pub inst_id: String,
    pub bar: Option<String>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<u8>,
}

impl Request for CandlesHistoryRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str = "/api/v5/market/history-candles";
    const HAS_PAYLOAD: bool = true;
    const REQUESTS_PER_SECOND: u8 = 10;
    type Response = Vec<CandleResponse>;
}
