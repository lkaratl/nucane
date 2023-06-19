pub mod utils;

use std::collections::HashMap;
use async_trait::async_trait;

use domain_model::{Action, InstrumentId, Tick};
use indicators_api::Indicators;
use storage_rest_client::StorageClient;

#[async_trait]
pub trait Strategy {
    fn tune(&mut self, _: &HashMap<String, String>) {}
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn subscriptions(&self) -> Vec<InstrumentId>;
    async fn execute(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action>;
}

pub struct StrategyApi {
    pub storage: StorageClient,
    pub indicators: Indicators,
}

impl Default for StrategyApi {
    fn default() -> Self {
        Self {
            storage: StorageClient::new("http://localhost:8082"),
            indicators: Indicators::new(StorageClient::new("http://localhost:8082")),
        }
    }
}
