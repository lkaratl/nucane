use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginQuery {
    pub name: Option<String>,
    pub version: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPluginQuery {
    pub force: bool,
}
