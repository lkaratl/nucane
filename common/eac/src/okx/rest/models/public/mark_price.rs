use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::okx::enums::InstType;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkPriceResponse {
    pub inst_type: InstType,
    pub inst_id: String,
    #[serde(deserialize_with = "crate::okx::parser::from_str")]
    pub mark_px: f64,
    #[serde(deserialize_with = "crate::okx::parser::ts_milliseconds")]
    pub ts: DateTime<Utc>,
}
