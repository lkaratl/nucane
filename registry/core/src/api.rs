use domain_model::{PluginBinary, PluginId, PluginInfo};
use registry_blob_api::BlobApi;
use registry_core_api::RegistryApi;
use anyhow::Result;
use async_trait::async_trait;

pub struct Registry<B: BlobApi> {
    plugins_storage: B,
}

impl<B: BlobApi> Registry<B> {
    pub fn new(plugins_storage: B) -> Self {
        Self {
            plugins_storage
        }
    }
}

#[async_trait]
impl<B: BlobApi> RegistryApi for Registry<B> {
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
                .map(|plugin|
                    PluginBinary::new(&plugin.strategy.name(),
                                      plugin.strategy.version(),
                                      &binary))
        } else {
            None
        }
    }

    async fn add_plugin(&self, binary: &[u8], force: bool) -> Result<PluginInfo> {
        let active_plugin = plugin_loader::load(binary)?;
        let name = &active_plugin.strategy.name();
        let version = active_plugin.strategy.version();
        let plugin = PluginBinary::new(name, version, binary);
        let result = self.plugins_storage.add_plugin(plugin, force).await;
        // synapse::writer(&CONFIG.broker.url).send(&plugin.as_event(PluginEventType::Updated)); // todo use rest client
        result
    }

    async fn delete_plugin(&self, id: PluginId) {
        self.plugins_storage.delete_plugin(id).await;
    }
}
