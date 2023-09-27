use tracing::info;

use registry_config::CONFIG;
use registry_core::Registry;
use registry_inmemory_blob_storage::InMemoryBlobStorage;

pub async fn run() {
    info!("▶ registry running...");

    let blob_storage = InMemoryBlobStorage::default();
    let registry = Registry::new(blob_storage);
    registry_rest_api_server::run(CONFIG.application.port, registry).await;
}
