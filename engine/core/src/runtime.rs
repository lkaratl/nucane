use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, PluginId, Tick};
use engine_core_api::api::Deployment;
use engine_plugin_internals::api::DefaultPluginInternals;
use plugin_api::PluginApi;
use storage_core_api::StorageApi;

use crate::fs_state_manager::FsStateManager;

pub struct Runtime<S: StorageApi> {
    deployments: Arc<RwLock<Vec<Deployment>>>,
    storage_client: Arc<S>,
    state_manager: Arc<FsStateManager>,
}

impl<S: StorageApi> Runtime<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self {
            deployments: Default::default(),
            storage_client,
            state_manager: Default::default(),
        }
    }

    pub async fn get_deployments_info(&self) -> Vec<DeploymentInfo> {
        self.deployments
            .read()
            .await
            .iter()
            .map(|deployment| deployment.into())
            .collect()
    }

    pub async fn deploy(&self, deployment: Deployment) {
        self.deployments.write().await.push(deployment);
    }

    pub async fn delete_deployment(&self, id: Uuid) -> Option<DeploymentInfo> {
        let index = self
            .deployments
            .read()
            .await
            .iter()
            .position(|deployment| deployment.id == id);
        if let Some(index) = index {
            let removed_deployment = self.deployments.write().await.remove(index);
            Some((&removed_deployment).into())
        } else {
            None
        }
    }

    // todo refactoring
    pub async fn get_actions(&self, tick: &Tick) -> Vec<Action> {
        let mut result = Vec::new();
        for deployment in self.deployments.write().await.iter_mut() {
            let is_simulation = deployment.simulation_id == tick.simulation_id;
            let plugin = &mut deployment.plugin.api;
            if is_subscribed(plugin.as_ref(), tick) && is_simulation {
                debug!(
                    "Processing tick: '{} {}-{}={}' for plugin: '{}:{}'",
                    tick.instrument_id.exchange,
                    tick.instrument_id.pair.target,
                    tick.instrument_id.pair.source,
                    tick.price,
                    plugin.id().name,
                    plugin.id().version
                );

                let state = self.state_manager.get(&deployment.id.to_string()).await;
                if let Some(state) = state {
                    plugin.set_state(state);
                }

                let plugin_internal_api = self.build_plugin_internal_api(
                    deployment.id,
                    plugin.id(),
                    deployment.simulation_id,
                );
                let mut actions = plugin.on_tick_sync(tick, plugin_internal_api);
                result.append(&mut actions);

                let state = plugin.get_state();
                self.state_manager.set(&deployment.id.to_string(), state).await;
            }
        }
        result
    }

    fn build_plugin_internal_api(
        &self,
        deployment_id: Uuid,
        plugin_id: PluginId,
        simulation_id: Option<Uuid>,
    ) -> Arc<DefaultPluginInternals<S>> {
        let storage_client = Arc::clone(&self.storage_client);
        Arc::new(DefaultPluginInternals::new(
            deployment_id,
            plugin_id,
            simulation_id,
            storage_client,
        ))
    }
}

fn is_subscribed(plugin: &(dyn PluginApi + Send), tick: &Tick) -> bool {
    let instrument_id = &tick.instrument_id;
    plugin.instruments().iter().any(|subscription| {
        subscription.exchange.eq(&instrument_id.exchange)
            && subscription.market_type.eq(&instrument_id.market_type)
            && subscription.pair.target.eq(&instrument_id.pair.target)
            && subscription.pair.source.eq(&instrument_id.pair.source)
    })
}
