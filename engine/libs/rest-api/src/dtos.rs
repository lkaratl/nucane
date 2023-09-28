use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateDeploymentDto {
    pub simulation_id: Option<Uuid>,
    pub name: String,
    pub version: i64,
    pub params: HashMap<String, String>,
}
