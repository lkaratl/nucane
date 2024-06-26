use std::sync::Arc;

use tracing::info;

use engine_rest_client::EngineRestClient;
use interactor_rest_client::InteractorRestClient;
use simulator_config::CONFIG;
use simulator_core::Simulator;
use simulator_postgres_persistence::initiator::init_db;
use simulator_postgres_persistence::repositories::SimulationReportPostgresRepository;
use storage_rest_client::StorageRestClient;

pub async fn run() {
    info!("▶ simulator running...");
    let db = init_db(&CONFIG.database.url, &CONFIG.application.name).await;
    let simulation_report_repository = Arc::new(SimulationReportPostgresRepository::new(db));
    let interactor_client = Arc::new(InteractorRestClient::new(&CONFIG.interactor.url));
    let engine_client = Arc::new(EngineRestClient::new(&CONFIG.engine.url));
    let storage_client = Arc::new(StorageRestClient::new(&CONFIG.storage.url));
    let engine = Simulator::new(
        engine_client,
        storage_client,
        interactor_client,
        simulation_report_repository,
    );
    simulator_rest_api_server::run(CONFIG.application.port, engine).await;
}
