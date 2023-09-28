use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use tokio::sync::Mutex;

use domain_model::{PluginBinary, PluginId, PluginInfo};
use registry_blob_api::BlobApi;

#[derive(Default)]
pub struct InMemoryBlobStorage {
    plugins: Arc<Mutex<RefCell<Vec<PluginBinary>>>>,
}

#[async_trait]
impl BlobApi for InMemoryBlobStorage {
    async fn get_plugins_info(&self) -> Vec<PluginInfo> {
        self.plugins.lock()
            .await
            .borrow()
            .iter()
            .cloned()
            .map(PluginInfo::from)
            .collect()
    }

    async fn get_plugins_info_by_name(&self, name: &str) -> Vec<PluginInfo> {
        self.plugins.lock()
            .await
            .borrow()
            .iter()
            .filter(|plugin| plugin.id.name == name)
            .cloned()
            .map(PluginInfo::from)
            .collect()
    }

    async fn get_plugin_binary(&self, id: PluginId) -> Option<Vec<u8>> {
        self.plugins.lock()
            .await
            .borrow()
            .iter()
            .find(|plugin| plugin.id == id)
            .map(|plugin| plugin.binary.clone())
    }

    async fn add_plugin(&self, plugin: PluginBinary, force: bool) -> Result<PluginInfo> {
        let existing_plugin_index = self.plugins.lock()
            .await
            .borrow_mut()
            .iter_mut()
            .position(|existing_plugin| existing_plugin.id.eq(&plugin.id));
        if let Some(existing_plugin_index) = existing_plugin_index {
            if force {
                if let Some(existing_plugin) = self.plugins.lock()
                    .await
                    .borrow_mut()
                    .get_mut(existing_plugin_index) {
                    existing_plugin.binary = plugin.binary;
                    return Ok(existing_plugin.deref().into());
                }
            } else {
                return bail!("Plugin with id: '{:?}' already exists in registry. Please use flag 'force=true' to override existing binary", plugin.id); // todo check this unreachable
            }
        }
        self.plugins.lock()
            .await
            .borrow_mut()
            .push(plugin.clone());
        Ok(plugin.into())
    }

    async fn delete_plugin(&self, id: PluginId) {
        self.plugins.lock()
            .await
            .borrow_mut()
            .retain(|plugin| !plugin.id.eq(&id));
    }
}
