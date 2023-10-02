use std::sync::Arc;

use tracing::info;

use engine_rest_client::EngineRestClient;
use registry_config::CONFIG;
use registry_core::Registry;
use registry_inmemory_blob_storage::InMemoryBlobStorage;

pub async fn run() {
    info!("▶ registry running...");

    let blob_storage = InMemoryBlobStorage::default();
    let engine_client = Arc::new(EngineRestClient::new(&CONFIG.engine.url));
    let registry = Registry::new(blob_storage, engine_client);
    registry_rest_api_server::run(CONFIG.application.port, registry).await;
}
