use std::sync::Arc;

use tracing::info;
use engine_rest_client::EngineRestClient;
use interactor_rest_client::InteractorRestClient;
use simulator_config::CONFIG;
use simulator_core::Simulator;
use storage_rest_client::StorageRestClient;

pub async fn run() {
    info!("▶ simulator running...");
    let interactor_client = Arc::new(InteractorRestClient::new(&CONFIG.interactor.url));
    let engine_client = Arc::new(EngineRestClient::new(&CONFIG.storage.url));
    let storage_client = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let engine = Simulator::new(engine_client, storage_client, interactor_client);
    simulator_rest_api_server::run(CONFIG.application.port, engine).await;
}
