mod config;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use domain_model::{Currency, Exchange, Position, Side, Simulation, SimulationPosition};
use engine_rest_client::EngineClient;
use storage_rest_client::StorageClient;

use simulator_core::{SimulationReport, SimulationService};
use simulator_rest_api::dto::{convert, CreateSimulationDto};
use simulator_rest_api::endpoints::POST_SIMULATION;
use crate::config::CONFIG;

pub async fn run() {
    info!("+ simulator running...");
    let strategy_engine_client = EngineClient::new("http://localhost:8081");
    let storage_client = StorageClient::new("http://localhost:8082");
    let simulation_service = Arc::new(SimulationService::new(strategy_engine_client, storage_client));

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

    let simulation = Simulation {
        id: simulation_id,
        timestamp: Utc::now(),
        start: Utc.timestamp_millis_opt(simulation.start).unwrap(),
        end: Utc.timestamp_millis_opt(simulation.end).unwrap(),
        positions,
        strategy_id: simulation.strategy_id,
        strategy_version: simulation.strategy_version,
        params: simulation.params,
    };
    let report = simulation_service.run(simulation).await;
    Json(report)
}
