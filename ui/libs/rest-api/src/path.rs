use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain_model::{Currency, Exchange, MarketType, Timeframe};

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationChartQuery {
    pub timeframe: Option<Timeframe>,
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub target: Currency,
    pub source: Currency,
}

#[derive(Deserialize)]
pub struct SimulationChartParams {
    pub simulation_id: Uuid,
    pub deployment_id: Uuid,
}
