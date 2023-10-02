use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;

use domain_model::CreateSimulation;
use simulator_core_api::{SimulationReport, SimulatorApi};
use simulator_rest_api::endpoints::POST_RUN_SIMULATION;

pub async fn run(port: u16, simulator: impl SimulatorApi) {
    let simulator = Arc::new(simulator);
    let router = Router::new()
        .route(POST_RUN_SIMULATION, post(create_simulation))
        .with_state(simulator);

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn create_simulation(State(simulator): State<Arc<dyn SimulatorApi>>,
                           Json(simulation): Json<CreateSimulation>) -> Json<SimulationReport> {
    let report = simulator.run_simulation(simulation)
        .await
        .unwrap();
    Json(report)
}
