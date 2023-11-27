use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain_model::{CreateSimulation, Order, SimulationDeployment, SimulationPosition};

#[async_trait]
pub trait SimulatorApi: Send + Sync + 'static {
    async fn run_simulation(&self, simulation: CreateSimulation) -> Result<SimulationReport>;
    async fn get_simulation_report(&self, id: Uuid) -> Result<SimulationReport>;
    async fn get_simulation_reports(&self) -> Result<Vec<SimulationReport>>;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationReport {
    pub simulation_id: Uuid,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub deployments: Vec<SimulationDeployment>,
    pub ticks: u32,
    pub actions: u32,
    pub profit: f64,
    pub profit_clear: f64,
    pub fees: f64,
    pub assets: Vec<SimulationPosition>,
    pub active_orders: Vec<Order>,

    pub sl_count: u64,
    pub tp_count: u64,
    pub sl_percent: f64,
    pub tp_percent: f64,
    pub max_sl_streak: u64,
    pub max_tp_streak: u64,
}
