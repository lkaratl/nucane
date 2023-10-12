use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::error::Elapsed;
use tracing::Level;
use tracing::{error, span};

use domain_model::{Action, InstrumentId, PluginId, Tick};
use indicators_api::Indicators;
use storage_core_api::StorageApi;

#[async_trait]
pub trait PluginApi: Send + Sync {
    fn id(&self) -> PluginId;
    fn configure(&mut self, _config: &HashMap<String, String>) {}
    fn instruments(&self) -> Vec<InstrumentId>;
    fn indicators(&self) -> Vec<()> {
        Vec::new()
    }
    // todo create common indicators enum
    fn on_tick_sync(&mut self, tick: &Tick, api: &PluginInternalApi) -> Vec<Action> {
        let tick_id = format!(
            "{} '{}' {}-{}='{}'",
            tick.instrument_id.exchange,
            tick.timestamp,
            tick.instrument_id.pair.target,
            tick.instrument_id.pair.source,
            tick.price
        );
        let _span = span!(
            Level::INFO,
            "strategy",
            name = self.id().name,
            version = self.id().version,
            tick_id
        )
        .entered();
        let runtime = with_tokio_runtime(self.on_tick(tick, api));
        match runtime {
            Ok(actions) => actions,
            Err(error) => {
                error!(
                    "Timeout during tick processing, strategy: '{}:{}'. Error: '{error}'",
                    self.id().name,
                    self.id().version
                );
                Vec::new()
            }
        }
    }

    async fn on_tick(&mut self, tick: &Tick, api: &PluginInternalApi) -> Vec<Action>;
}

#[tokio::main]
async fn with_tokio_runtime<T: Default>(future: impl Future<Output = T>) -> Result<T, Elapsed> {
    tokio::time::timeout(Duration::from_secs(60), future).await
}

pub struct PluginInternalApi {
    pub storage_client: Arc<dyn StorageApi>,
    pub indicators: Indicators,
}

impl PluginInternalApi {
    pub fn new(storage_client: Arc<dyn StorageApi>) -> Self {
        Self {
            storage_client: Arc::clone(&storage_client),
            indicators: Indicators::new(storage_client),
        }
    }
}
