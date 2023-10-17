use std::sync::Arc;

use async_trait::async_trait;

use domain_model::Order;
use plugin_api::OrdersInternalApi;
use storage_core_api::StorageApi;

pub struct DefaultOrderInternals<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> DefaultOrderInternals<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }
}

#[async_trait]
impl<S: StorageApi> OrdersInternalApi for DefaultOrderInternals<S> {
    async fn get_order_by_id(&self, id: &str) -> Option<Order> {
        self.storage_client
            .get_orders(
                Some(id.to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap()
            .first()
            .cloned()
    }
}
