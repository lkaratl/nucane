use std::collections::HashMap;

use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_urlencoded::to_string as to_ustring;
use tracing::{debug, trace};
use uuid::Uuid;

use domain_model::{Action, AuditEvent, Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position, Side, Tick, Timeframe};

pub struct StrategyEngineClient {
    url: String,
    client: Client,
}

impl StrategyEngineClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn create_deployment(&self,
                                   simulation_id: Option<Uuid>,
                                   strategy_id: &str,
                                   strategy_version: &str,
                                   params: HashMap<String, String>, ) -> Result<DeploymentInfo, Error> {
        let body = CreateDeploymentBody {
            simulation_id,
            strategy_id: strategy_id.to_string(),
            strategy_version: strategy_version.to_string(),
            params,
        };

        let endpoint = format!("{}{}", self.url, "/api/v1/strategy-engine/deployments");
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

    pub async fn remove_deployment(&self,
                                   id: Uuid) -> Result<(), Error> {
        let endpoint = format!("{}{}/{id}", self.url, "/api/v1/strategy-engine/deployments");
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.delete(url)
            .send()
            .await?;
        Ok(())
    }

    pub async fn create_actions(&self, tick: &Tick) -> Result<Vec<Action>, Error> {
        let endpoint = format!("{}{}", self.url, "/api/v1/strategy-engine/actions");
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.post(url)
            .json(tick)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}

#[derive(Serialize)]
struct CreateDeploymentBody {
    pub strategy_id: String,
    pub simulation_id: Option<Uuid>,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeploymentInfo {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub strategy_id: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_remove_deployment() {
        let mut params = HashMap::new();
        params.insert("test".to_string(), "test".to_string());
        let client = StrategyEngineClient::new("http://localhost:8081");
        let response = client.create_deployment(
            None,
            "test",
            "1.0",
            params).await
            .unwrap();

        client.remove_deployment(response.id)
            .await
            .unwrap();
    }
}
