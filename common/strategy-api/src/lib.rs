pub mod utils;

use std::collections::HashMap;
use std::future::Future;
use async_trait::async_trait;
use tokio::runtime::Runtime;

use domain_model::{Action, InstrumentId, Tick};
use indicators_api::Indicators;
use storage_rest_client::StorageClient;

#[async_trait]
pub trait Strategy {
    fn tune(&mut self, _: &HashMap<String, String>) {}
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn subscriptions(&self) -> Vec<InstrumentId>;

    fn on_tick_sync(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action> {
        with_tokio_runtime(self.on_tick(tick, api))
    }
    async fn on_tick(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action>;
}

fn with_tokio_runtime<T>(future: impl Future<Output=T>) -> T {
    let rt = Runtime::new().unwrap();
    rt.block_on(future)
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
