use http::Method;
use serde::{Deserialize, Serialize};

use super::super::Request;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
pub struct BalanceResponse {
    pub imr: String,
}

impl Request for BalanceRequest {
    const METHOD: Method = Method::GET;
    const SIGNED: bool = true;
    const ENDPOINT: &'static str = "/api/v5/account/balance";
    const HAS_PAYLOAD: bool = false;
    type Response = [BalanceResponse; 1];
}
