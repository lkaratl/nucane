pub mod utils;

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;
use async_trait::async_trait;
use tokio::time::error::Elapsed;
use tracing::{error, span};
use tracing::Level;

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
        let tick_id = format!("{} '{}' {}-{}='{}'", tick.instrument_id.exchange, tick.timestamp,
                              tick.instrument_id.pair.target, tick.instrument_id.pair.source, tick.price);
        let _span = span!(Level::INFO, "strategy", name = self.name(), version = self.version(), tick_id).entered();
        let runtime = with_tokio_runtime(self.on_tick(tick, api));
        match runtime {
            Ok(actions) => actions,
            Err(_) => {
                error!("Timeout during tick processing, strategy: '{}:{}'", self.name(), self.version());
                Vec::new()
            }
        }
    }

    async fn on_tick(&mut self, tick: &Tick, api: &StrategyApi) -> Vec<Action>;
}

#[tokio::main]
async fn with_tokio_runtime<T: Default>(future: impl Future<Output=T>) -> Result<T, Elapsed> {
    tokio::time::timeout(Duration::from_secs(5), future).await
}

pub struct StrategyApi {
    pub storage: StorageClient,
    pub indicators: Indicators,
}

impl StrategyApi {
    pub fn new(storage_url: &str) -> Self {
        Self {
            storage: StorageClient::new(storage_url),
            indicators: Indicators::new(StorageClient::new(storage_url)),
        }
    }
}
