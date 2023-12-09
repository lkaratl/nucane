use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TickerLtResponse {
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
