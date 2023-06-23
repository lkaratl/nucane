pub mod config;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use chrono::{TimeZone, Utc};
use tracing::info;
use uuid::Uuid;

use domain_model::Simulation;
use engine_rest_client::EngineClient;
use interactor_rest_client::InteractorClient;
use storage_rest_client::StorageClient;

use simulator_core::{SimulationReport, SimulationService};
use simulator_rest_api::dto::{convert, convert_to_simulation_deployment, CreateSimulationDto};
use simulator_rest_api::endpoints::POST_SIMULATION;
use crate::config::CONFIG;

pub async fn run() {
    info!("+ simulator running...");
    let strategy_engine_client = EngineClient::new("http://localhost:8081");
    let storage_client = StorageClient::new("http://localhost:8082");
    let interactor_client = InteractorClient::new("http://localhost:8083");
    let simulation_service = Arc::new(SimulationService::new(strategy_engine_client, storage_client, interactor_client));

    let router = Router::new()
        .route(POST_SIMULATION, post(create_simulation))
        .with_state(Arc::clone(&simulation_service));

    let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), CONFIG.application.port);
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn create_simulation(State(simulation_service): State<Arc<SimulationService>>,
                           Json(simulation): Json<CreateSimulationDto>) -> Json<SimulationReport> {
    let simulation_id = Uuid::new_v4();
    let positions = simulation.positions.into_iter()
        .map(|position| convert(position, simulation_id))
        .collect();
    let deployments = simulation.strategies.into_iter()
        .map(convert_to_simulation_deployment)
        .collect();

    let simulation = Simulation {
        id: simulation_id,
        timestamp: Utc::now(),
        start: Utc.timestamp_millis_opt(simulation.start).unwrap(),
        end: Utc.timestamp_millis_opt(simulation.end).unwrap(),
        positions,
        deployments,
        ticks_len: 0,
        actions_count: 0,
        active_orders: Vec::new(),
    };
    let report = simulation_service.run(simulation).await;
    Json(report)
}
