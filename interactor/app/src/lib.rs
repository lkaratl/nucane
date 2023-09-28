use std::sync::Arc;
use tracing::info;
use engine_rest_client::EngineRestClient;

use interactor_config::CONFIG;
use interactor_core::Interactor;
use interactor_inmemory_persistence::InMemorySubscriptionRepository;
use storage_rest_client::StorageRestClient;

pub async fn run() {
    info!("▶ interactor running ...");
    if CONFIG.eac.demo {
        info!(" ▸ interactor: DEMO mode");
    } else {
        info!(" ▸ interactor: LIVE mode");
    }
    let engine_client = Arc::new(EngineRestClient::new(&CONFIG.engine.url));
    let storage_client = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let subscription_repository = InMemorySubscriptionRepository::default();
    let interactor = Interactor::new(engine_client, storage_client, subscription_repository);
    interactor_rest_api_server::run(CONFIG.application.port, interactor).await;
}

