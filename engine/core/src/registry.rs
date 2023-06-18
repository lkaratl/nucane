use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;
use plugin_loader::Plugin;

pub async fn get_deployments() -> Vec<Arc<Mutex<Deployment>>> {
    find(|_| true).await
}

pub async fn find(predicate: impl Fn(&MutexGuard<Deployment>) -> bool) -> Vec<Arc<Mutex<Deployment>>> {
    let deployments = REGISTRY.lock()
        .await
        .get_deployments();
    let deployments = deployments.lock()
        .await;
    let mut result = Vec::new();
    for deployment in deployments.iter() {
        let guard = deployment.lock().await;
        if predicate(&guard) {
            result.push(Arc::clone(&deployment));
        }
    }
    result
}

pub async fn add_deployment(deployment: Arc<Mutex<Deployment>>) {
    REGISTRY.lock()
        .await
        .get_deployments()
        .lock()
        .await
        .push(deployment);
}

pub async fn delete_if(predicate: impl Fn(&Deployment) -> bool) -> Vec<Arc<Mutex<Deployment>>> {
    let registry = REGISTRY.lock().await;
    let deployments = registry.get_deployments();
    let mut deployments = deployments
        .lock()
        .await;
    let mut result = Vec::new();
    let mut is_removed = true;
    while is_removed {
        let mut deployment_position = None;
        for (i, deployment) in deployments.iter().enumerate() {
            let guard = deployment.lock().await;
            if predicate(&guard) {
                deployment_position = Some(i);
            }
        }
        if let Some(position) = deployment_position {
            result.push(deployments.remove(position));
        } else {
            is_removed = false;
        }
    }
    result
}

#[derive(Debug)]
pub struct Deployment {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub params: HashMap<String, String>,
    pub plugin: Plugin,
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

#[derive(Default)]
struct Registry {
    deployments: Arc<Mutex<Vec<Arc<Mutex<Deployment>>>>>,
}

impl Registry {
    fn get_deployments(&self) -> Arc<Mutex<Vec<Arc<Mutex<Deployment>>>>> {
        Arc::clone(&self.deployments)
    }
}
