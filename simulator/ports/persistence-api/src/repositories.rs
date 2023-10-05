use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use simulator_core_api::SimulationReport;

#[async_trait]
pub trait SimulationReportRepository: Send + Sync + 'static {
    async fn save(&self, simulation_report: SimulationReport) -> Result<()>;

    async fn get(&self, id: Option<Uuid>) -> Vec<SimulationReport>;
}
