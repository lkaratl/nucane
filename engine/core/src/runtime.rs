use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use domain_model::{Action, DeploymentInfo, Tick};
use engine_core_api::api::Deployment;
use plugin_api::{PluginApi, PluginInternalApi};
use storage_core_api::StorageApi;

pub struct Runtime {
    deployments: Arc<RwLock<Vec<Deployment>>>,
    api: PluginInternalApi,
}

impl Runtime {
    pub fn new<S: StorageApi>(storage_client: Arc<S>) -> Self {
        Self {
            deployments: Default::default(),
            api: PluginInternalApi::new(storage_client),
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
                let mut actions = plugin.on_tick_sync(tick, &self.api);
                actions.iter_mut().for_each(|action| match action {
                    Action::OrderAction(order_action) => {
                        order_action.simulation_id = deployment.simulation_id
                    }
                });
                result.append(&mut actions);
            }
        }
        result
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
