use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use crate::model::Plugin;

pub async fn find(predicate: impl FnMut(&&Plugin) -> bool) -> Vec<Plugin> {
    REGISTRY.lock()
        .await
        .get_plugins()
        .lock()
        .await
        .iter()
        .filter(predicate)
        .cloned()
        .collect()
}

pub async fn add_plugin(plugin: Plugin) {
    REGISTRY.lock()
        .await
        .get_plugins()
        .lock()
        .await
        .push(plugin)
}

pub async fn delete_if(predicate: impl Fn(&Plugin) -> bool) -> Vec<Plugin> {
    let registry = REGISTRY.lock().await;
    let mut plugins = registry.plugins
        .lock()
        .await;
    let mut result = Vec::new();
    let mut is_removed = true;
    while is_removed {
        let plugin_position =
            plugins.iter()
                .position(&predicate);
        if let Some(position) = plugin_position {
            result.push(plugins.remove(position));
        } else {
            is_removed = false;
        }
    }
    result
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

struct Registry {
    plugins: Arc<Mutex<Vec<Plugin>>>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            plugins: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl Registry {
    fn get_plugins(&self) -> Arc<Mutex<Vec<Plugin>>> {
        Arc::clone(&self.plugins)
    }
}
