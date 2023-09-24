use std::cell::RefCell;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Mutex;

use domain_model::{InstrumentId, Subscriptions};
use interactor_persistence_api::{SubscriptionRepository};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemorySubscriptionRepository {
    storage: Arc<Mutex<RefCell<Vec<Subscriptions>>>>,
}

impl InMemorySubscriptionRepository {}

#[async_trait]
impl SubscriptionRepository for InMemorySubscriptionRepository {
    async fn get_all(&self) -> Vec<Subscriptions> {
       self.storage
           .lock()
           .await
           .borrow()
           .clone()
    }

    async fn get_by_deployment(&self, deployment_id: &Uuid) -> Vec<Subscriptions> {
        self.storage
            .lock()
            .await
            .borrow()
            .iter()
            .filter(|subscription| subscription.deployment_ids.contains(deployment_id))
            .cloned()
            .collect()
    }

    async fn get_be_instrument(&self, instrument_id: &InstrumentId) -> Option<Subscriptions> {
        self.storage
            .lock()
            .await
            .borrow()
            .iter()
            .find(|subscription| subscription.instrument_id.eq(instrument_id))
            .cloned()
    }


    async fn save(&self, subscription: &Subscriptions) -> Result<()> {
        let storage = self.storage
            .lock()
            .await;
        let mut storage = storage.borrow_mut();
        let existing_subscription = storage.iter_mut()
            .find(|subscription| subscription.instrument_id.eq(&subscription.instrument_id));
        if let Some(existing_subscription) = existing_subscription {
            existing_subscription.deployment_ids.extend(subscription.deployment_ids.clone());
        } else {
            storage.push(subscription.clone());
        }
        Ok(())
    }

    async fn delete(&self, instrument_id: &InstrumentId) -> Result<()> {
        self.storage
            .lock()
            .await
            .borrow_mut()
            .retain(|subscription| !subscription.instrument_id.eq(instrument_id));
        Ok(())
    }
}
