use serde::{Deserialize, Serialize};
use domain_model::{Currency, Exchange, MarketType, OrderStatus, OrderType, Side, Timeframe};

#[derive(Debug, Deserialize, Serialize)]
pub struct CandlesQuery {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub target: Currency,
    pub source: Currency,
    pub timeframe: Option<Timeframe>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrdersQuery {
    pub id: Option<String>,
    pub exchange: Option<Exchange>,
    pub market_type: Option<MarketType>,
    // todo not convenient to pass Margin(Cross)
    pub target: Option<Currency>,
    pub source: Option<Currency>,
    pub status: Option<OrderStatus>,
    pub side: Option<Side>,
    pub order_type: Option<OrderType>, // todo not convenient to pass Limit(XX.XX)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PositionsQuery {
    pub exchange: Option<Exchange>,
    pub currency: Option<Currency>,
    pub side: Option<Side>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditQuery {
    pub from_timestamp: Option<i64>,
    pub tags: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandleSyncQuery {
    pub timeframes: String,
    pub from: i64,
    pub to: Option<i64>,
}
