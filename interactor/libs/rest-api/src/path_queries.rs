use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use domain_model::{Currency, Exchange, MarketType, Timeframe};

#[serde_inline_default]
#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct CandlesQuery {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub target: Currency,
    pub source: Currency,
    pub timeframe: Timeframe,
    pub from: Option<i64>,
    pub to: Option<i64>,
    #[serde_inline_default(100)]
    pub limit: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceQuery {
    pub timestamp: Option<i64>,
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub target: Currency,
    pub source: Currency,
}
