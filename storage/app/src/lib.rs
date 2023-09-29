use std::sync::Arc;

use tracing::info;

use interactor_rest_client::InteractorRestClient;
use storage_config::CONFIG;
use storage_core::Storage;
use storage_postgres_persistence::initiator::init_db;
use storage_postgres_persistence::repositories::{CandlePostgresRepository, OrderPostgresRepository, PositionPostgresRepository};

pub async fn run() {
    info!("▶ storage running...");
    let db =init_db(&CONFIG.database.url, &CONFIG.application.name).await;

    let interactor_client = InteractorRestClient::new(&CONFIG.interactor.url);
    let order_repository = OrderPostgresRepository::new(Arc::clone(&db));
    let position_repository = PositionPostgresRepository::new(Arc::clone(&db));
    let candle_repository = CandlePostgresRepository::new(Arc::clone(&db));

    let storage = Storage::new(interactor_client, order_repository, position_repository, candle_repository);
    storage_rest_api_server::run(CONFIG.application.port, storage).await;
}

