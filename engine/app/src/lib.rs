use std::sync::Arc;

use tracing::info;

use engine_config::CONFIG;
use engine_core::Engine;
use interactor_rest_client::InteractorRestClient;
use registry_rest_client::RegistryRestClient;
use storage_rest_client::StorageRestClient;

pub async fn run() {
    info!("▶ engine running...");
    let interactor_client = Arc::new(InteractorRestClient::new(&CONFIG.interactor.url));
    let registry = Arc::new(RegistryRestClient::new(&CONFIG.registry.url));
    let storage = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let engine = Engine::new(interactor_client, registry, storage);
    engine_rest_api_server::run(CONFIG.application.port, engine).await;
}
