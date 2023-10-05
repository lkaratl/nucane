use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use domain_model::Timeframe;

#[async_trait]
pub trait UiApi: Send + Sync + 'static {
    async fn get_simulation_chart_html(
        &self,
        simulation_id: Uuid,
        timeframe: Option<Timeframe>,
    ) -> Result<String>;
}
