use uuid::Uuid;
use domain_model::{PluginEvent, PluginEventType};

#[derive(Clone)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub binary: Vec<u8>,
}

impl Plugin {
    pub fn new(name: &str, version: &str, binary: &[u8]) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            version: version.to_string(),
            binary: binary.to_vec(),
        }
    }

    pub fn as_event(&self, event_type: PluginEventType) -> PluginEvent {
        PluginEvent {
            id: self.id,
            strategy_name: self.name.clone(),
            strategy_version: self.version.clone(),
            event: event_type,
        }
    }
}
