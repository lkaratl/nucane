use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use domain_model::{PluginBinary, PluginId, PluginInfo};
use engine_core_api::api::EngineApi;
use registry_blob_api::BlobApi;
use registry_core_api::RegistryApi;

pub struct Registry<B: BlobApi, E: EngineApi> {
    plugins_storage: B,
    engine_client: Arc<E>,
}

impl<B: BlobApi, E: EngineApi> Registry<B, E> {
    pub fn new(plugins_storage: B, engine_client: Arc<E>) -> Self {
        Self {
            plugins_storage,
            engine_client,
        }
    }
}

#[async_trait]
impl<B: BlobApi, E: EngineApi> RegistryApi for Registry<B, E> {
    async fn get_plugins_info(&self) -> Vec<PluginInfo> {
        self.plugins_storage.get_plugins_info().await
    }

    async fn get_plugins_info_by_name(&self, name: &str) -> Vec<PluginInfo> {
        self.plugins_storage.get_plugins_info_by_name(name).await
    }

    async fn get_plugin_binary(&self, id: PluginId) -> Option<PluginBinary> {
        let binary = self.plugins_storage.get_plugin_binary(id).await;
        if let Some(binary) = binary {
            plugin_loader::load(&binary)
                .ok()
                .map(|plugin| PluginBinary::new(plugin.api.id(), &binary))
        } else {
            None
        }
    }

    async fn add_plugin(&self, binary: &[u8], force: bool) -> Result<PluginInfo> {
        let active_plugin = plugin_loader::load(binary)?;
        let plugin = PluginBinary::new(active_plugin.api.id(), binary);
        let result = self.plugins_storage.add_plugin(plugin, force).await;
        if let Ok(plugin_info) = &result {
            self.engine_client
                .update_plugin(plugin_info.id.clone())
                .await;
        }
        result
    }

    async fn delete_plugin(&self, id: PluginId) {
        self.plugins_storage.delete_plugin(id).await;
    }
}
