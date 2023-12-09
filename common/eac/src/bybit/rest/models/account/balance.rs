use http::Method;
use serde::{Deserialize, Serialize};

use super::super::Request;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct BalanceRequest {
    pub ccy: Option<String>,
}

impl BalanceRequest {
    pub fn multiple<S>(currencies: &[S]) -> Self
        where
            S: AsRef<str>,
    {
        Self {
            ccy: Some(
                currencies
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub imr: String,
    #[serde(deserialize_with = "crate::bybit::parser::from_str")]
    pub total_eq: f64,
}

impl Request for BalanceRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/account/balance";
    const HAS_PAYLOAD: bool = false;
    type Response = [BalanceResponse; 1];
}
