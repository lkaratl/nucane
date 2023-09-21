use domain_model::{InstrumentId, Subscription};
use interactor_persistence_api::SubscriptionRepository;
use anyhow::Result;

pub struct EmbeddedSubscriptionRepository{

}

impl SubscriptionRepository for EmbeddedSubscriptionRepository {
    fn get(&self, instrument_id: &InstrumentId) -> Option<Subscription> {
        todo!()
    }

    fn insert_or_update(&self, subscription: &Subscription) -> Result<()> {
        todo!()
    }

    fn delete(&self) -> Result<()> {
        todo!()
    }
}
