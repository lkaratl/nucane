use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use tracing::{trace};
use simulator_core::SimulationReport;
use simulator_rest_api::dto::{CreatePositionDto, CreateSimulationDeploymentDto, CreateSimulationDto};
use simulator_rest_api::endpoints::POST_SIMULATION;

pub struct SimulatorClient {
    url: String,
    client: Client,
}

impl SimulatorClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn run_simulation(&self,
                                start: DateTime<Utc>,
                                end: DateTime<Utc>,
                                positions: Vec<CreatePositionDto>,
                                strategies: Vec<CreateSimulationDeploymentDto>) -> Result<SimulationReport, Error> {
        let body = CreateSimulationDto {
            start: start.timestamp_millis(),
            end: end.timestamp_millis(),
            positions,
            strategies,
        };

        let endpoint = format!("{}{}", self.url, POST_SIMULATION);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.post(url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}
