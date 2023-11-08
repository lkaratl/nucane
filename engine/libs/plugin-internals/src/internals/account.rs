use std::sync::Arc;

use async_trait::async_trait;

use domain_model::Exchange;
use interactor_core_api::InteractorApi;
use plugin_api::AccountInternalApi;

pub struct DefaultAccountInternals<I: InteractorApi> {
    interactor_client: Arc<I>,
}

impl<I: InteractorApi> DefaultAccountInternals<I> {
    pub fn new(interactor_client: Arc<I>) -> Self {
        Self { interactor_client }
    }
}

#[async_trait]
impl<I: InteractorApi> AccountInternalApi for DefaultAccountInternals<I> {
    async fn total_balance(&self, exchange: Exchange) -> f64 {
        self.interactor_client.get_total_balance(exchange).await.unwrap()
    }
}
