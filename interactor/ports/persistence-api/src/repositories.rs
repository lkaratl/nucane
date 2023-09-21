use domain_model::{InstrumentId, Subscription};
use anyhow::Result;

pub trait SubscriptionRepository {
    fn get(&self, instrument_id: &InstrumentId) -> Option<Subscription>;
    fn insert_or_update(&self, subscription: &Subscription) -> Result<()>;
    fn delete(&self) -> Result<()>;
}