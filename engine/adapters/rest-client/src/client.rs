use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Client, Url};
use tracing::trace;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, PluginId, Tick};
use engine_core_api::api::{EngineApi, EngineError};
use engine_rest_api::endpoints::{DELETE_DEPLOYMENT, POST_CREATE_ACTIONS};

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
        todo!()
    }

    async fn deploy(&self, simulation_id: Option<Uuid>, strategy_name: &str, strategy_version: i64, params: &HashMap<String, String>) -> Result<DeploymentInfo, EngineError> {
        // let endpoint = format!("{}{}", self.url, POST_CREATE_DEPLOYMENTS);
        // let url = Url::parse(&endpoint)
        //     .map_err(|_| EngineError::PluginLoadingError)?;
        // trace!("Request url: {url:?}");
        // let response = self.client.post(url)
        //     .json(&create_deployments)
        //     .send()
        //     .await.unwrap()
        //     .json()
        //     .await.unwrap();
        // Ok(response)
        todo!()
    }

    async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let endpoint = format!("{}{}", self.url, POST_CREATE_ACTIONS);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client.post(url)
            .json(tick)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap()
    }

    async fn delete_deployment(&self, id: Uuid) -> Option<DeploymentInfo> {
        let endpoint = format!("{}{}/{id}", self.url, DELETE_DEPLOYMENT);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client.delete(url)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap()
    }

    async fn update_plugin(&self, plugin_id: PluginId) {
        let endpoint = format!("{}{}", self.url, POST_CREATE_ACTIONS);
        let url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client.put(url)
            .json(&plugin_id)
            .send()
            .await.unwrap();
    }
}
