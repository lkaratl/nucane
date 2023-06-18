use std::collections::HashMap;

use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};
use domain_model::{Currency, Exchange, Order, Side, SimulationPosition};

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
                                strategy_id: &str,
                                strategy_version: &str,
                                params: HashMap<String, String>, ) -> Result<SimulationReport, Error> {
        let body = CreateSimulationBody {
            start: start.timestamp_millis(),
            end: end.timestamp_millis(),
            positions,
            strategy_id: strategy_id.to_string(),
            strategy_version: strategy_version.to_string(),
            params,
        };

        let endpoint = format!("{}{}", self.url, "/api/v1/simulator/simulations");
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

#[derive(Serialize)]
struct CreateSimulationBody {
    pub start: i64,
    pub end: i64,
    pub positions: Vec<CreatePositionDto>,
    pub strategy_id: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePositionDto {
    pub exchange: Exchange,
    pub currency: Currency,
    pub side: Side,
    pub size: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationReport {
    pub ticks: usize,
    pub actions: u16,
    pub profit: f64,
    pub fees: f64,
    pub assets: Vec<SimulationPosition>,
    pub active_orders: Vec<Order>,
}
