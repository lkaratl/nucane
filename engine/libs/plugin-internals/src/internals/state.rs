use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use engine_fs_plugin_state::FsStateManager;
use plugin_api::StateInternalApi;

pub struct DefaultStateInternals {
    deployment_id: Uuid,
    state_manager: Arc<FsStateManager>,
}

impl DefaultStateInternals {
    pub fn new(deployment_id: Uuid, state_manager: Arc<FsStateManager>) -> Self {
        Self {
            deployment_id,
            state_manager,
        }
    }
}

#[async_trait]
impl StateInternalApi for DefaultStateInternals {
    async fn set(&self, key: &str, state: Value) {
        let key = format!("{}-{}", self.deployment_id, key);
        self.state_manager.set(&key, state).await;
    }

    async fn get(&self, key: &str) -> Option<Value> {
        let key = format!("{}-{}", self.deployment_id, key);
        self.state_manager.get(&key).await
    }
}
