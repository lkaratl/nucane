pub mod endpoints {
    pub const POST_SIMULATION: &str = "/api/v1/simulator/simulations";
}

pub mod dto {
    use std::collections::HashMap;
    use uuid::Uuid;
    use domain_model::{Currency, Exchange, Side, SimulationPosition};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateSimulationDto {
        pub start: i64,
        pub end: i64,
        pub positions: Vec<CreatePositionDto>,
        pub strategy_id: String,
        pub strategy_version: String,
        pub params: HashMap<String, String>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreatePositionDto {
        pub exchange: Exchange,
        pub currency: Currency,
        pub side: Side,
        pub size: f64,
    }

    pub fn convert(value: CreatePositionDto, simulation_id: Uuid) -> SimulationPosition {
        SimulationPosition {
            simulation_id,
            exchange: value.exchange,
            currency: value.currency,
            start: value.size,
            end: value.size,
            diff: 0.0,
            fees: 0.0,
        }
    }
}
