use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use interactor_config::CONFIG;
use interactor_core::{Interactor, ServiceFacade, SubscriptionManager};

pub async fn run() {
    if CONFIG.eac.demo {
        info!("+ interactor running in DEMO mode..."); // todo move this log to best place
    } else {
        info!("+ interactor running in LIVE mode...");
    }
    let service_facade = ServiceFacade::new();
    let subscription_manager = SubscriptionManager::new(Arc::new(Mutex::new(ServiceFacade::new())));
    let interactor = Interactor::new(service_facade, subscription_manager);
    interactor_rest_api_server::run(CONFIG.application.port, interactor)
}
