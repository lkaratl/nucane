use uuid::Uuid;
use crate::model::Plugin;
use crate::registry;
use anyhow::Result;
use domain_model::PluginEventType;
use registry_config::CONFIG;

#[derive(Default)]
pub struct RegistryService {}

impl RegistryService {
    pub async fn get_plugins(&self) -> Vec<Plugin> {
        registry::find(|_| true).await
    }

    pub async fn get_plugin_by_id(&self, id: Uuid) -> Option<Plugin> {
        registry::find(|plugin| plugin.id == id).await
            .first()
            .cloned()
    }

    pub async fn get_plugins_by_name(&self, name: &str) -> Vec<Plugin> {
        registry::find(|plugin| plugin.name == name).await
    }

    pub async fn get_plugin_by_name_and_version(&self, name: &str, version: &str) -> Option<Plugin> {
        registry::find(|plugin| plugin.name == name && plugin.version == version).await
            .first()
            .cloned()
    }

    pub async fn add_plugin(&self, binary: &[u8]) -> Result<Plugin> { // todo support win/unix/mac with validation
        let plugin = plugin_loader::load(binary)?;
        let name = &plugin.strategy.name();
        let version = &plugin.strategy.version();

        let existing_plugin = self.remove_plugin_by_name_and_version(name, version).await
            .as_mut()
            .map(|plugin| {
                plugin.binary = binary.to_vec();
                plugin
            })
            .cloned();

        let already_exists = existing_plugin.is_some();
        let plugin = existing_plugin.unwrap_or(Plugin::new(name, version, binary));
        registry::add_plugin(plugin.clone()).await;

        if already_exists {
            // synapse::writer(&CONFIG.broker.url).send(&plugin.as_event(PluginEventType::Updated)); // todo use rest client
        }
        Ok(plugin)
    }

    pub async fn remove_plugin_by_id(&self, id: Uuid) -> Option<Plugin> {
        registry::delete_if(|plugin| plugin.id == id).await
            .first()
            .cloned()
    }

    pub async fn remove_plugins_by_name(&self, name: &str) -> Vec<Plugin> {
        registry::delete_if(|plugin| plugin.name == name).await
    }

    pub async fn remove_plugin_by_name_and_version(&self, name: &str, version: &str) -> Option<Plugin> {
        registry::delete_if(|plugin| plugin.name == name && plugin.version == version).await
            .first()
            .cloned()
    }
}
