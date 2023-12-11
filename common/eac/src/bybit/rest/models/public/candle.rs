use http::Method;
use serde::{Deserialize, Serialize};

use crate::bybit::enums::Category;
use crate::bybit::rest::Request;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleResponse {
    pub symbol: String,
    pub category: Category,
    pub list: Vec<(
        String, // start time
        String, // open price
        String, // highest price
        String, // lowest price
        String, // close price
        String, // volume
        String, // turnover
    )>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesRequest {
    pub category: Category,
    pub symbol: String,
    pub interval: String,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub limit: Option<u16>,
}

impl Request for CandlesRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = false;
    const ENDPOINT: &'static str = "/v5/market/kline";
    const HAS_PAYLOAD: bool = true;
    const REQUESTS_PER_SECOND: u8 = 10;
    type Response = CandleResponse;
}
