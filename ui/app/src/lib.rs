use std::sync::Arc;

use tracing::info;

use indicators::Indicators;
use simulator_rest_client::SimulatorRestClient;
use storage_core_api_cache::StorageCoreApiCache;
use storage_rest_client::StorageRestClient;
use ui_charming_builder::CharmingBuilder;
use ui_config::CONFIG;
use ui_core::Ui;

pub async fn run() {
    info!("▶ ui running...");
    let chart_builder = Arc::new(CharmingBuilder);
    let simulator_client = Arc::new(SimulatorRestClient::new(&CONFIG.simulator.url));
    let storage_client = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let storage_client_cached = Arc::new(StorageCoreApiCache::new(storage_client).await);
    let indicator = Arc::new(Indicators::new(Arc::clone(&storage_client_cached)));
    let ui = Ui::new(simulator_client, storage_client_cached, chart_builder, indicator);
    ui_rest_api_server::run(CONFIG.application.port, ui).await;
}
