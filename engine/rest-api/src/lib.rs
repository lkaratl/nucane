pub mod endpoints {
    pub const GET_POST_DEPLOYMENTS: &str = "/api/v1/engine/deployments";
    pub const PUT_DELETE_DEPLOYMENTS_BY_ID: &str = "/api/v1/engine/deployments/:id";
    pub const POST_CREATE_ACTIONS: &str = "/api/v1/engine/actions";
}

pub mod dto {
    use std::collections::HashMap;
    use uuid::Uuid;
    use domain_model::InstrumentId;
    use engine_core::registry::Deployment;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DeploymentInfo {
        pub id: Uuid,
        pub simulation_id: Option<Uuid>,
        pub strategy_id: String,
        pub strategy_version: String,
        pub params: HashMap<String, String>,
        pub subscriptions: Vec<InstrumentId>,
    }

    impl From<&Deployment> for DeploymentInfo {
        fn from(value: &Deployment) -> Self {
            DeploymentInfo {
                id: value.id,
                simulation_id: value.simulation_id,
                strategy_id: value.plugin.strategy.name(),
                strategy_version: value.plugin.strategy.version(),
                params: value.params.clone(),
                subscriptions: value.plugin.strategy.subscriptions(),
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct CreateDeployment {
        pub simulation_id: Option<Uuid>,
        pub strategy_name: String,
        pub strategy_version: String,
        pub params: HashMap<String, String>,
    }
}
