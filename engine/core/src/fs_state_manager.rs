use dashmap::DashMap;
use serde_json::Value;

#[derive(Default)]
pub struct FsStateManager {
    state: DashMap<String, Value>, // todo really value?
}

impl FsStateManager {
    pub async fn set(&self, key: &str, state: Value) {
        // todo remove await if not needed
        self.state.insert(key.to_string(), state);
    }
    pub async fn get(&self, key: &str) -> Option<Value> {
        // todo remove await if not needed
        self.state.get(key).map(|value| value.value().clone())
    }
}
