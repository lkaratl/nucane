use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain_model::{CreateSimulation, Order, SimulationDeployment, SimulationPosition};

#[async_trait]
pub trait SimulatorApi: Send + Sync + 'static {
    async fn run_simulation(&self, simulation: CreateSimulation) -> Result<SimulationReport>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationReport {
    pub simulation_id: Uuid,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub deployments: Vec<SimulationDeployment>,
    pub ticks: usize,
    pub actions: u16,
    pub profit: f64,
    pub profit_clear: f64,
    pub fees: f64,
    pub assets: Vec<SimulationPosition>,
    pub active_orders: Vec<Order>,
}
