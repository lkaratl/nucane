use std::sync::Arc;

use tracing::info;

use interactor_config::CONFIG;
use interactor_core::{Interactor, ServiceFacade, SubscriptionManager};
use interactor_inmemory_persistence::InMemorySubscriptionRepository;

pub async fn run() {
    info!("▶ interactor running ...");
    if CONFIG.eac.demo {
        info!(" ▸ interactor: DEMO mode");
    } else {
        info!(" ▸ interactor: LIVE mode");
    }
    let service_facade = Arc::new(ServiceFacade::new());
    let subscription_repository = InMemorySubscriptionRepository::default();
    let subscription_manager = SubscriptionManager::new(Arc::clone(&service_facade), subscription_repository);
    let interactor = Interactor::new(service_facade, subscription_manager);
    interactor_rest_api_server::run(CONFIG.application.port, interactor).await;
}

