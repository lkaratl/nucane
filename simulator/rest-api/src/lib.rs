pub mod endpoints {
    pub const POST_SIMULATION: &str = "/api/v1/simulator/simulations";
}

pub mod dto {
    use std::collections::HashMap;
    use uuid::Uuid;
    use domain_model::{Currency, Exchange, Side, SimulationDeployment, SimulationPosition, Timeframe};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateSimulationDto {
        pub start: i64,
        pub end: i64,
        pub positions: Vec<CreatePositionDto>,
        pub strategies: Vec<CreateSimulationDeploymentDto>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateSimulationDeploymentDto {
        pub simulation_id: Option<Uuid>,
        pub timeframe: Timeframe,
        pub strategy_name: String,
        pub strategy_version: String,
        pub params: HashMap<String, String>,
    }

    pub fn convert_to_simulation_deployment(value: CreateSimulationDeploymentDto, ) -> SimulationDeployment {
        SimulationDeployment {
            deployment_id: None,
            timeframe: value.timeframe,
            strategy_name: value.strategy_name,
            strategy_version: value.strategy_version,
            params: value.params,
            subscriptions: Vec::new(),
        }
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
