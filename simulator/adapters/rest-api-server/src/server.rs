use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use domain_model::CreateSimulation;
use simulator_core_api::{SimulationReport, SimulatorApi};
use simulator_rest_api::endpoints::{GET_SIMULATION, GET_SIMULATIONS, POST_RUN_SIMULATION};

pub async fn run(port: u16, simulator: impl SimulatorApi) {
    let simulator = Arc::new(simulator);
    let router = Router::new()
        .route(POST_RUN_SIMULATION, post(create_simulation))
        .route(GET_SIMULATIONS, get(get_simulation_reports))
        .route(GET_SIMULATION, get(get_simulation_report))
        .with_state(simulator);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn create_simulation(
    State(simulator): State<Arc<dyn SimulatorApi>>,
    Json(simulation): Json<CreateSimulation>,
) -> Json<SimulationReport> {
    let report = simulator.run_simulation(simulation).await.unwrap();
    Json(report)
}

async fn get_simulation_reports(
    State(simulator): State<Arc<dyn SimulatorApi>>,
) -> Json<Vec<SimulationReport>> {
    let reports = simulator.get_simulation_reports().await.unwrap();
    Json(reports)
}

async fn get_simulation_report(
    State(simulator): State<Arc<dyn SimulatorApi>>,
    Path(simulation_id): Path<Uuid>,
) -> Json<SimulationReport> {
    let reports = simulator
        .get_simulation_report(simulation_id)
        .await
        .unwrap();
    Json(reports)
}
