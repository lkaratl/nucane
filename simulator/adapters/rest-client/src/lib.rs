use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};
use tracing::trace;
use uuid::Uuid;

use domain_model::CreateSimulation;
use simulator_core_api::{SimulationReport, SimulatorApi};
use simulator_rest_api::endpoints::{GET_SIMULATION, GET_SIMULATIONS, POST_RUN_SIMULATION};

pub struct SimulatorRestClient {
    url: String,
    client: Client,
}

impl SimulatorRestClient {
    pub fn new(url: &str) -> Self {
        let mut url = String::from(url);
        if !url.starts_with("http") {
            url = format!("http://{}", url);
        }
        Self {
            url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SimulatorApi for SimulatorRestClient {
    async fn run_simulation(&self, simulation: CreateSimulation) -> Result<SimulationReport> {
        let endpoint = format!("{}{}", self.url, POST_RUN_SIMULATION);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self
            .client
            .post(url)
            .json(&simulation)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    async fn get_simulation_report(&self, id: Uuid) -> Result<SimulationReport> {
        let endpoint = format!("{}{}", self.url, GET_SIMULATION).replace(":id", &id.to_string());
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.get(url).send().await?.json().await?;
        Ok(response)
    }

    async fn get_simulation_reports(&self) -> Result<Vec<SimulationReport>> {
        let endpoint = format!("{}{}", self.url, GET_SIMULATIONS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.get(url).send().await?.json().await?;
        Ok(response)
    }
}
