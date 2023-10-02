use async_trait::async_trait;
use reqwest::{Client, Url};
use tracing::trace;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, NewDeployment, PluginId, Tick};
use engine_core_api::api::{EngineApi, EngineError};
use engine_rest_api::endpoints::{
    DELETE_DEPLOYMENT, GET_DEPLOYMENTS, POST_CREATE_ACTIONS, POST_CREATE_DEPLOYMENTS,
};

pub struct EngineRestClient {
    url: String,
    client: Client,
}

impl EngineRestClient {
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
impl EngineApi for EngineRestClient {
    async fn get_deployments_info(&self) -> Vec<DeploymentInfo> {
        let endpoint = format!("{}{}", self.url, GET_DEPLOYMENTS);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn deploy(
        &self,
        deployments: &[NewDeployment],
    ) -> Result<Vec<DeploymentInfo>, EngineError> {
        let endpoint = format!("{}{}", self.url, POST_CREATE_DEPLOYMENTS);
        let url = Url::parse(&endpoint).map_err(|_| EngineError::PluginLoadingError)?;
        trace!("Request url: {url:?}");
        let response = self
            .client
            .post(url)
            .json(&deployments)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        Ok(response)
    }

    async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let endpoint = format!("{}{}", self.url, POST_CREATE_ACTIONS);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client
            .post(url)
            .json(tick)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn delete_deployment(&self, id: Uuid) -> Option<DeploymentInfo> {
        let endpoint = format!("{}{}", self.url, DELETE_DEPLOYMENT).replace(":id", &id.to_string());
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client
            .delete(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn update_plugin(&self, plugin_id: PluginId) {
        let endpoint = format!("{}{}", self.url, POST_CREATE_ACTIONS);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client.put(url).json(&plugin_id).send().await.unwrap();
    }
}
