use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;

use domain_model::{InstrumentId, Subscription};
use interactor_persistence_api::SubscriptionRepository;

use crate::exchanges::ServiceFacade;

pub struct SubscriptionManager<S: SubscriptionRepository> {
    subscriptions: Vec<Subscriptions>,
    subscription_repository: S,
    service_facade: Arc<Mutex<ServiceFacade>>,
}

impl<S: SubscriptionRepository> SubscriptionManager<S> {
    pub fn new(service_facade: Arc<Mutex<ServiceFacade>>, subscription_repository: impl SubscriptionRepository) -> Self {
        Self {
            subscription_repository,
            subscriptions: Vec::new(),
            service_facade,
        }
    }

    pub async fn subscribe(&mut self, new_subscription: Subscription) {
        debug!("Subscribe: {}", new_subscription.deployment_id);
        for new_instrument in new_subscription.instruments {
            let subscription = self.subscriptions.iter_mut()
                .find(|subscription| subscription.instrument_id.eq(&new_instrument));
            if let Some(subscription) = subscription {
                subscription.deployment_ids.push(new_subscription.deployment_id);
            } else {
                let mut service_facade = self.service_facade.lock().await;
                service_facade.listen_orders(new_instrument.exchange).await;
                service_facade.listen_position(new_instrument.exchange).await;
                service_facade.subscribe_candles(&new_instrument).await;
                service_facade.subscribe_ticks(&new_instrument).await;

                self.subscriptions.push(Subscriptions {
                    instrument_id: new_instrument,
                    deployment_ids: vec![new_subscription.deployment_id],
                })
            }
        }
    }

    pub async fn unsubscribe(&mut self, deployment_id: Uuid) {
        debug!("Unsubscribe: {}", deployment_id);
        self.subscriptions.iter_mut()
            .for_each(|subscription| {
                subscription.deployment_ids.retain(|id| !id.eq(&deployment_id));
            });
        let mut service_facade = self.service_facade.lock().await;
        self.subscriptions.retain(|subscription| {
            if subscription.deployment_ids.is_empty() {
                service_facade.unsubscribe_candles(&subscription.instrument_id);
                service_facade.unsubscribe_ticks(&subscription.instrument_id);
                false
            } else {
                true
            }
        });
    }
}

#[derive(Debug)]
struct Subscriptions {
    pub instrument_id: InstrumentId,
    pub deployment_ids: Vec<Uuid>,
}
