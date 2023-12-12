use chrono::{DateTime, Utc};
use http::Method;
use serde::{Deserialize, Serialize};

use crate::okx::rest::Request;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleResponse(
    #[serde(deserialize_with = "crate::okx::parser::ts_milliseconds")]
    pub DateTime<Utc>, // ts
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Open price
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Highest price
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Lowest price
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Close price
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Volume
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Target volume
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub f64, // Source volume
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub String, // Status
);

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
