use serde::{Deserialize, Serialize};

use domain_model::{Currency, Exchange, MarketType, Timeframe};

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationChartQuery {
    pub timeframe: Option<Timeframe>,
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub target: Currency,
    pub source: Currency,
}
