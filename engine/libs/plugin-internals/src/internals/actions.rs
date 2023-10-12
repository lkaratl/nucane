use async_trait::async_trait;

use plugin_api::ActionsInternalApi;

#[derive(Default)]
pub struct DefaultActionInternals {}

#[async_trait]
impl ActionsInternalApi for DefaultActionInternals {}
