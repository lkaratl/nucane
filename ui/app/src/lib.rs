use std::sync::Arc;

use tracing::info;

use simulator_rest_client::SimulatorRestClient;
use storage_rest_client::StorageRestClient;
use ui_charming_builder::CharmingBuilder;
use ui_config::CONFIG;
use ui_core::Ui;

pub async fn run() {
    info!("▶ ui running...");
    let chart_builder = Arc::new(CharmingBuilder);
    let simulator_client = Arc::new(SimulatorRestClient::new(&CONFIG.simulator.url));
    let storage_client = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let ui = Ui::new(simulator_client, storage_client, chart_builder);
    ui_rest_api_server::run(CONFIG.application.port, ui).await;
}
