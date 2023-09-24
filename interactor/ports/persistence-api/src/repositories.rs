use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use domain_model::{InstrumentId, Subscriptions};

#[async_trait]
pub trait SubscriptionRepository: Send + Sync + 'static {
    async fn get_all(&self) -> Vec<Subscriptions>;
    async fn get_by_deployment(&self, deployment_id: &Uuid) -> Vec<Subscriptions>;
    async fn get_be_instrument(&self, instrument_id: &InstrumentId) -> Option<Subscriptions>;
    async fn save(&self, subscription: &Subscriptions) -> Result<()>;
    async fn save_many(&self, subscriptions: &[Subscriptions]) -> Result<()> {
        for subscription in subscriptions {
            self.save(subscription).await?;
        }
        Ok(())
    }
    async fn delete(&self, instrument_id: &InstrumentId) -> Result<()>;
}
