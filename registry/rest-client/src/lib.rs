use anyhow::Result;
use reqwest::{Client, Url};
use tracing::{trace};
use uuid::Uuid;
use registry_rest_api::dto::{PluginBinary, PluginInfo};
use registry_rest_api::endpoints::{DELETE_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION, GET_BINARY_BY_ID_OR_NAME_OR_VERSION, GET_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION};
use registry_rest_api::path_query::PluginQuery;


pub struct RegistryClient {
    url: String,
    client: Client,
}

impl RegistryClient {
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

    pub async fn get_info(&self, id: Option<Uuid>, name: Option<String>, version: Option<String>) -> Result<Vec<PluginInfo>> {
        let query = PluginQuery {
            id,
            name,
            version,
        };
        let endpoint = format!("{}{}", self.url, GET_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&serde_urlencoded::to_string(&query)?));
        trace!("Request url: {url:?}");
        let response = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn get_binary(&self, id: Option<Uuid>, name: Option<String>, version: Option<String>) -> Result<Vec<PluginBinary>> {
        let query = PluginQuery {
            id,
            name,
            version,
        };
        let endpoint = format!("{}{}", self.url, GET_BINARY_BY_ID_OR_NAME_OR_VERSION);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&serde_urlencoded::to_string(&query)?));
        trace!("Request url: {url:?}");
        let response = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn delete(&self, id: Option<Uuid>, name: Option<String>, version: Option<String>) -> Result<Vec<PluginInfo>> {
        let query = PluginQuery {
            id,
            name,
            version,
        };
        let endpoint = format!("{}{}", self.url, DELETE_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&serde_urlencoded::to_string(&query)?));
        trace!("Request url: {url:?}");
        let response = self.client.delete(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}
