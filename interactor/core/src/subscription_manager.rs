use tracing::debug;
use uuid::Uuid;

use domain_model::{Deployment, InstrumentId};
use crate::service::ServiceFacade;

pub struct SubscriptionManager {
    subscriptions: Vec<Subscriptions>,
    service_facade: ServiceFacade,
}

impl SubscriptionManager {
    pub fn new(service_facade: ServiceFacade) -> Self {
        Self {
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
                self.service_facade.listen_position(new_instrument.exchange).await;
                self.service_facade.listen_orders(new_instrument.exchange).await;
                self.service_facade.subscribe_candles(&new_instrument).await;
                self.service_facade.subscribe_ticks(&new_instrument).await;

                self.subscriptions.push(Subscriptions {
                    instrument_id: new_instrument,
                    deployment_ids: vec![new_subscription.deployment_id],
                })
            }
        }
    }

    pub fn unsubscribe(&mut self, deployment_id: Uuid) {
        debug!("Unsubscribe: {}", deployment_id);
        self.subscriptions.iter_mut()
            .for_each(|subscription| {
                subscription.deployment_ids.retain(|id| !id.eq(&deployment_id));
            });
        self.subscriptions.retain(|subscription| {
            if subscription.deployment_ids.is_empty() {
                self.service_facade.unsubscribe_candles(&subscription.instrument_id);
                self.service_facade.unsubscribe_ticks(&subscription.instrument_id);
                false
            } else {
                true
            }
        });
    }
}

#[derive(Debug)]
pub struct Subscription {
    pub deployment_id: Uuid,
    pub instruments: Vec<InstrumentId>,
}

impl From<Deployment> for Subscription {
    fn from(value: Deployment) -> Self {
        Self {
            deployment_id: value.id,
            instruments: value.subscriptions,
        }
    }
}

#[derive(Debug)]
pub struct Subscriptions {
    pub instrument_id: InstrumentId,
    pub deployment_ids: Vec<Uuid>,
}
