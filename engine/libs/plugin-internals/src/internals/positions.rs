use std::sync::Arc;

use async_trait::async_trait;

use domain_model::{Currency, Exchange, Position};
use plugin_api::PositionsInternalApi;
use storage_core_api::StorageApi;

pub struct DefaultPositionInternals<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> DefaultPositionInternals<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }
}

#[async_trait]
impl<S: StorageApi> PositionsInternalApi for DefaultPositionInternals<S> {
    async fn get_position(&self, exchange: Exchange, currency: Currency) -> Option<Position> {
        self.storage_client
            .get_positions(Some(exchange), Some(currency), None)
            .await
            .unwrap()
            .first()
            .cloned()
    }
}
