use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, DeploymentStatus, NewDeployment, PluginId, Tick};
use plugin_loader::Plugin;

#[async_trait]
pub trait EngineApi: Send + Sync + 'static {
    async fn get_deployments_info(&self) -> Vec<DeploymentInfo>;
    async fn deploy(&self, deployments: &[NewDeployment]) -> Result<Vec<DeploymentInfo>, EngineError>;
    async fn get_actions(&self, tick: &Tick) -> Vec<Action>;
    async fn delete_deployment(&self, id: Uuid) -> Option<DeploymentInfo>;
    async fn update_plugin(&self, plugin_id: PluginId);
}

#[derive(Debug)]
pub struct Deployment {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub params: HashMap<String, String>,
    pub plugin: Plugin,
}

impl From<&Deployment> for DeploymentInfo {
    fn from(value: &Deployment) -> Self {
        Self {
            id: value.id,
            status: DeploymentStatus::Created,
            simulation_id: value.simulation_id,
            plugin_id: PluginId::new(&value.plugin.strategy.name(),
                                     value.plugin.strategy.version()),
            params: value.params.clone(),
            subscriptions: value.plugin.strategy.subscriptions(),
        }
    }
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Plugin not found in registry")]
    PluginNotFound,
    #[error("Failed to load plugin")]
    PluginLoadingError,
}
