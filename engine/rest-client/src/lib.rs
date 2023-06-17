use std::collections::HashMap;
use anyhow::Error;
use reqwest::{Client, Url};
use tracing::{trace};
use uuid::Uuid;

use domain_model::{Action, Tick};
use engine_rest_api::dto::{CreateDeployment, DeploymentInfo};
use engine_rest_api::endpoints::{GET_POST_DEPLOYMENTS, POST_CREATE_ACTIONS, PUT_DELETE_DEPLOYMENTS_BY_ID};

pub struct EngineClient {
    url: String,
    client: Client,
}

impl EngineClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn create_deployment(&self,
                                   simulation_id: Option<Uuid>,
                                   strategy_name: &str,
                                   strategy_version: &str,
                                   params: HashMap<String, String>, ) -> Result<DeploymentInfo, Error> {
        let body = CreateDeployment {
            simulation_id,
            strategy_name: strategy_name.to_string(),
            strategy_version: strategy_version.to_string(),
            params,
        };

        let endpoint = format!("{}{}", self.url, GET_POST_DEPLOYMENTS);
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
        let endpoint = format!("{}{}/{id}", self.url, PUT_DELETE_DEPLOYMENTS_BY_ID);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.delete(url)
            .send()
            .await?;
        Ok(())
    }

    pub async fn create_actions(&self, tick: &Tick) -> Result<Vec<Action>, Error> {
        let endpoint = format!("{}{}", self.url, POST_CREATE_ACTIONS);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_remove_deployment() {
        let mut params = HashMap::new();
        params.insert("test".to_string(), "test".to_string());
        let client = EngineClient::new("http://localhost:8081");
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
