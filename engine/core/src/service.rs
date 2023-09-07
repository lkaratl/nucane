use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use registry_rest_client::RegistryClient;
use crate::registry;
use crate::registry::Deployment;
use anyhow::{Result};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error};
use domain_model::DeploymentEvent;
use engine_config::CONFIG;
use plugin_loader::Plugin;
use synapse::SynapseSend;
use crate::service::EngineError::{PluginLoadingError, PluginNotFound};

pub struct EngineService {
    registry_client: Arc<RegistryClient>,
}

impl EngineService {
    pub fn new(registry_client: Arc<RegistryClient>) -> Self {
        Self {
            registry_client
        }
    }

    pub async fn get_deployments(&self) -> Vec<Arc<Mutex<Deployment>>> {
        registry::get_deployments().await
    }

    pub async fn get_deployment_by_name_and_version(&self, strategy_name: &str, strategy_version: &str) -> Vec<Arc<Mutex<Deployment>>> {
        registry::find(|deployment| deployment.plugin.strategy.name() == strategy_name &&
            deployment.plugin.strategy.version() == strategy_version)
            .await
    }

    pub async fn add_deployment(&self, simulation_id: Option<Uuid>, strategy_name: &str, strategy_version: &str, params: &HashMap<String, String>) -> Result<Arc<Mutex<Deployment>>, EngineError> {
        debug!("Create deployment for strategy with name: '{strategy_name}' and version: '{strategy_version}' and params: '{params:?}'");
        let plugin = self.load_plugin(strategy_name, strategy_version, params).await?;
        // let existing_deployment = self.remove_deployment_by_name_and_version(strategy_name, strategy_version).await;
        let deployment = Arc::new(Mutex::new(Deployment {
                id: Uuid::new_v4(),
                simulation_id,
                params: params.clone(),
                plugin,
            }));

        registry::add_deployment(deployment.clone()).await;
        let event = deployment_to_event(deployment.clone(), DeploymentEvent::Created).await;
        synapse::writer(&CONFIG.broker.url,).send(&event);
        Ok(deployment)
    }

    pub async fn update_plugin(&self, strategy_name: &str, strategy_version: &str) -> Result<(), EngineError> {
        let deployments = self.get_deployment_by_name_and_version(strategy_name, strategy_version).await;
        for deployment in deployments {
            let _deployment = deployment.lock().await;
            // self.add_or_update_deployment(deployment.simulation_id, strategy_name, strategy_version, &deployment.params).await?; // todo make update method
        }
        Ok(())
    }

    async fn load_plugin(&self, strategy_name: &str, strategy_version: &str, params: &HashMap<String, String>) -> Result<Plugin, EngineError> {
        let plugin = self.registry_client.get_binary(None,
                                                     Some(strategy_name.to_string()),
                                                     Some(strategy_version.to_string()),
        ).await.map_err(|err| {
            error!("Error during plugin loading: {err}");
            PluginLoadingError
        })?;
        let plugin = plugin.first().ok_or(PluginNotFound)?;
        let mut plugin = plugin_loader::load(&plugin.binary).map_err(|err| {
            error!("Error during plugin loading: {err}");
            PluginLoadingError
        })?;
        plugin.strategy.tune(params);
        Ok(plugin)
    }

    pub async fn remove_deployment_by_id(&self, id: Uuid) -> Option<Arc<Mutex<Deployment>>> {
        let deployment = registry::delete_if(|deployment| deployment.id == id).await
            .first()
            .cloned();
        if let Some(deployment) = deployment.clone() {
            let event = &deployment_to_event(deployment, DeploymentEvent::Deleted).await;
            synapse::writer(&CONFIG.broker.url,).send(event);
        }
        deployment
    }

    pub async fn remove_deployment_by_name_and_version(&self, name: &str, version: &str) -> Option<Arc<Mutex<Deployment>>> {
        let deployment = registry::delete_if(|deployment| deployment.plugin.strategy.name() == name && deployment.plugin.strategy.version() == version).await
            .first()
            .cloned();
        if let Some(deployment) = deployment.clone() {
            let event = &deployment_to_event(deployment, DeploymentEvent::Deleted).await;
            synapse::writer(&CONFIG.broker.url,).send(event);
        }
        deployment
    }
}


async fn deployment_to_event(deployment: Arc<Mutex<Deployment>>, event_type: DeploymentEvent) -> domain_model::Deployment {
    let deployment = deployment.lock().await;
    domain_model::Deployment {
        id: deployment.id,
        simulation_id: deployment.simulation_id,
        event: event_type,
        strategy_name: deployment.plugin.strategy.name(),
        strategy_version: deployment.plugin.strategy.version(),
        params: deployment.params.clone(),
        subscriptions: deployment.plugin.strategy.subscriptions(),
    }
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Plugin not found in registry")]
    PluginNotFound,
    #[error("Failed to load plugin")]
    PluginLoadingError,
}
