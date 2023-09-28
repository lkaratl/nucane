use async_trait::async_trait;
use reqwest::{Client, Url};
use tracing::trace;
use domain_model::CreateSimulation;
use simulator_core_api::{SimulationReport, SimulatorApi};
use simulator_rest_api::endpoints::POST_RUN_SIMULATION;

pub struct SimulatorRestClient {
    url: String,
    client: Client,
}

impl SimulatorRestClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SimulatorApi for SimulatorRestClient {
    async fn run_simulation(&self, simulation: CreateSimulation) -> anyhow::Result<SimulationReport> {
        let endpoint = format!("{}{}", self.url, POST_RUN_SIMULATION);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.post(url)
            .json(&simulation)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}
