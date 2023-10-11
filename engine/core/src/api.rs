use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use axum::async_trait;
use chrono::{Duration, Utc};
use tracing::{debug, error, info};
use uuid::Uuid;

use domain_model::{
    Action, DeploymentInfo, InstrumentId, NewDeployment, PluginId, Tick, Timeframe,
};
use engine_core_api::api::{Deployment, EngineApi, EngineError};
use interactor_core_api::InteractorApi;
use plugin_loader::Plugin;
use registry_core_api::RegistryApi;
use storage_core_api::StorageApi;

use crate::api::EngineError::{PluginLoadingError, PluginNotFound};
use crate::runtime::Runtime;

pub struct Engine<I: InteractorApi, R: RegistryApi, S: StorageApi> {
    interactor_client: Arc<I>,
    registry_client: Arc<R>,
    storage_client: Arc<S>,
    runtime: Runtime,
}

impl<I: InteractorApi, R: RegistryApi, S: StorageApi> Engine<I, R, S> {
    pub fn new(interactor_client: Arc<I>, registry_client: Arc<R>, storage_client: Arc<S>) -> Self {
        Self {
            interactor_client,
            registry_client,
            storage_client: Arc::clone(&storage_client),
            runtime: Runtime::new(storage_client),
        }
    }

    async fn load_plugin(
        &self,
        name: &str,
        version: i64,
        params: &HashMap<String, String>,
    ) -> Result<Plugin, EngineError> {
        let plugin_id = PluginId::new(name, version);
        let plugin = self.registry_client.get_plugin_binary(plugin_id)
            .await
            .ok_or_else(|| {
                error!("Error required plugin not found in registry, name: '{name}', versions: '{version}'");
                PluginNotFound
            })?;
        let mut plugin = plugin_loader::load(&plugin.binary).map_err(|err| {
            error!("Error during plugin loading: {err}");
            PluginLoadingError
        })?;
        plugin.strategy.tune(params);
        Ok(plugin)
    }

    async fn sync_data(&self, subscriptions: &[InstrumentId]) {
        let timeframes = [
            Timeframe::FiveM,
            Timeframe::FifteenM,
            Timeframe::ThirtyM,
            Timeframe::OneD,
            Timeframe::FourH,
            Timeframe::OneD,
        ];
        let from = Utc::now() - Duration::days(30);
        for subscription in subscriptions {
            let sync_result = self
                .storage_client
                .sync(subscription, &timeframes, from, None)
                .await
                .unwrap();
            info!("Sync data reports: '{sync_result:?}' for instrument: '{subscription:?}'")
        }
    }

    async fn deploy_single(
        &self,
        deployment: &NewDeployment,
    ) -> Result<DeploymentInfo, EngineError> {
        let strategy_name = deployment.plugin_id.name.clone();
        let strategy_version = deployment.plugin_id.version;
        let params = deployment.params.clone();
        debug!("Create deployment for strategy with name: '{strategy_name}' and version: '{strategy_version}' and params: '{params:?}'");

        let plugin = self
            .load_plugin(&strategy_name, strategy_version, &params)
            .await?;
        let deployment = Deployment {
            id: Uuid::new_v4(),
            simulation_id: deployment.simulation_id,
            params: params.clone(),
            plugin,
        };
        let deployment_info: DeploymentInfo = (&deployment).into();
        self.runtime.deploy(deployment).await;
        self.sync_data(&deployment_info.subscriptions).await;
        self.interactor_client
            .subscribe((&deployment_info).into())
            .await
            .unwrap();
        Ok(deployment_info)
    }
}

#[async_trait]
impl<I: InteractorApi, R: RegistryApi, S: StorageApi> EngineApi for Engine<I, R, S> {
    async fn get_deployments_info(&self) -> Vec<DeploymentInfo> {
        self.runtime.get_deployments_info().await
    }

    async fn deploy(
        &self,
        deployments: &[NewDeployment],
    ) -> Result<Vec<DeploymentInfo>, EngineError> {
        let mut result = Vec::new();
        for deployment in deployments {
            let deployment_info = self.deploy_single(deployment).await?;
            result.push(deployment_info);
        }
        Ok(result)
    }

    async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let actions = self.runtime.get_actions(tick).await;
        if !actions.is_empty() {
            self.interactor_client
                .execute_actions(actions.clone())
                .await
                .unwrap();
        }
        actions
    }

    async fn delete_deployment(&self, id: Uuid) -> Option<DeploymentInfo> {
        self.runtime.delete_deployment(id).await
    }

    async fn update_plugin(&self, plugin_id: PluginId) {
        let all_deployments = self.get_deployments_info().await;
        let outdated_deployments = all_deployments
            .iter()
            .filter(|deployment| deployment.plugin_id == plugin_id);

        for deployment in outdated_deployments {
            self.delete_deployment(deployment.id).await;
            let new_deployment = NewDeployment {
                simulation_id: deployment.simulation_id,
                plugin_id: plugin_id.clone(),
                params: deployment.params.clone(),
            };
            self.deploy_single(&new_deployment).await.unwrap();
        }
    }
}
