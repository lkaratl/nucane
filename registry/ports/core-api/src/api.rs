use anyhow::Result;
use async_trait::async_trait;
use domain_model::{PluginBinary, PluginId, PluginInfo};

#[async_trait]
pub trait RegistryApi: Send + Sync + 'static {
    async fn get_plugins_info(&self) -> Vec<PluginInfo>;
    async fn get_plugins_info_by_name(&self, name: &str) -> Vec<PluginInfo>;
    async fn get_plugin_binary(&self, id: PluginId) -> Option<PluginBinary>;
    async fn add_plugin(&self, binary: &[u8], force: bool) -> Result<PluginInfo>;
    async fn delete_plugin(&self, id: PluginId);
}
