use anyhow::Error;
use reqwest::{Client, Url};
use tracing::{trace};
use uuid::Uuid;

use domain_model::{Action, Tick};
use engine_rest_api::dto::{CreateDeploymentDto, DeploymentInfo};
use engine_rest_api::endpoints::{GET_POST_DEPLOYMENTS, POST_CREATE_ACTIONS, DELETE_DEPLOYMENTS_BY_ID};

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

    pub async fn create_deployment(&self, create_deployments: Vec<CreateDeploymentDto>) -> Result<Vec<DeploymentInfo>, Error> {
        let endpoint = format!("{}{}", self.url, GET_POST_DEPLOYMENTS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let response = self.client.post(url)
            .json(&create_deployments)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn remove_deployment(&self,
                                   id: Uuid) -> Result<(), Error> {
        let endpoint = format!("{}{}/{id}", self.url, DELETE_DEPLOYMENTS_BY_ID);
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
