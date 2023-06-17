use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use uuid::Uuid;
use plugin_loader::Plugin;

pub async fn get_deployments() -> Arc<Mutex<Vec<Deployment>>> {
    REGISTRY.lock()
        .await
        .get_deployments()
}

pub async fn add_deployment(deployment: Deployment) {
    REGISTRY.lock()
        .await
        .deployments
        .lock()
        .await
        .push(deployment)
}

pub async fn delete_deployment(id: &Uuid) -> Option<Deployment> {
    let registry = REGISTRY.lock().await;
    let mut deployments = registry.deployments
        .lock()
        .await;
    let deployment_position =
        deployments.iter()
            .position(|deployment| deployment.id.eq(id));
    if let Some(position) = deployment_position {
        return Some(deployments.remove(position));
    }
    None
}

pub struct Deployment {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub params: HashMap<String, String>,
    pub plugin: Plugin,
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

struct Registry {
    deployments: Arc<Mutex<Vec<Deployment>>>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            deployments: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl Registry {
    fn get_deployments(&self) -> Arc<Mutex<Vec<Deployment>>> {
        Arc::clone(&self.deployments)
    }
}
