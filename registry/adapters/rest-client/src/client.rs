use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};
use tracing::{trace};
use domain_model::{Plugin, PluginId, PluginInfo};
use registry_core_api::RegistryApi;
use registry_rest_api::endpoints::{DELETE_PLUGINS, GET_PLUGIN_BINARY, GET_PLUGINS_INFO};
use registry_rest_api::path_query::PluginQuery;

pub struct RegistryRestClient {
    url: String,
    client: Client,
}

impl RegistryRestClient {
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
impl RegistryApi for RegistryRestClient {
    async fn get_plugins_info(&self) -> Vec<PluginInfo> {
        let endpoint = format!("{}{}", self.url, GET_PLUGINS_INFO);
        let mut url = Url::parse(&endpoint).unwrap();
        trace!("Request url: {url:?}");
        self.client.get(url)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap()
    }

    async fn get_plugins_info_by_name(&self, name: &str) -> Vec<PluginInfo> {
        let query = PluginQuery {
            name: Some(name.to_string()),
            version: None
        };
        let endpoint = format!("{}{}", self.url, GET_PLUGINS_INFO);
        let mut url = Url::parse(&endpoint).unwrap();
        url.set_query(Some(&serde_urlencoded::to_string(&query).unwrap()));
        trace!("Request url: {url:?}");
        self.client.get(url)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap()
    }

    async fn get_plugin_binary(&self, id: PluginId) -> Option<Plugin> {
        let query = PluginQuery {
            name: Some(id.name),
            version: Some(id.version),
        };
        let endpoint = format!("{}{}", self.url, GET_PLUGIN_BINARY);
        let mut url = Url::parse(&endpoint).unwrap();
        url.set_query(Some(&serde_urlencoded::to_string(&query).unwrap()));
        trace!("Request url: {url:?}");
        let response = self.client.get(url)
            .send()
            .await.unwrap()
            .json()
            .await.unwrap();
        Some(response)
    }

    async fn add_plugin(&self, _binary: &[u8], _force: bool) -> Result<PluginInfo> {
        unimplemented!("currently denied to use")
    }

    async fn delete_plugin(&self, id: PluginId) {
        let query = PluginQuery {
            name: Some(id.name),
            version: Some(id.version),
        };
        let endpoint = format!("{}{}", self.url, DELETE_PLUGINS);
        let mut url = Url::parse(&endpoint).unwrap();
        url.set_query(Some(&serde_urlencoded::to_string(&query).unwrap()));
        trace!("Request url: {url:?}");
        self.client.delete(url)
            .send()
            .await.unwrap();
    }
}
