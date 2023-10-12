use async_trait::async_trait;

use plugin_api::StateInternalApi;

#[derive(Default)]
pub struct DefaultStateInternals {}

#[async_trait]
impl StateInternalApi for DefaultStateInternals {}
