use serde::{Deserialize, Serialize};

use domain_model::Timeframe;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationChartQuery {
    pub timeframe: Option<Timeframe>,
}
