use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use domain_model::Timeframe;
use ui_core_api::UiApi;

pub struct Ui;

impl Ui {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UiApi for Ui {
    async fn get_simulation_chart_html(
        &self,
        simulation_id: Uuid,
        timeframe: Option<Timeframe>,
    ) -> Result<String> {
        Ok("<p>Hello, World!</p>".to_string())
    }
}
