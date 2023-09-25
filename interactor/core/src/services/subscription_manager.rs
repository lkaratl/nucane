use std::collections::HashSet;
use std::sync::Arc;

use tracing::debug;

use domain_model::{Subscription, Subscriptions};
use interactor_persistence_api::SubscriptionRepository;

use crate::exchanges::ServiceFacade;

pub struct SubscriptionManager<S: SubscriptionRepository> {
    subscription_repository: S,
    service_facade: Arc<ServiceFacade>,
}

impl<S: SubscriptionRepository> SubscriptionManager<S> {
    pub fn new(service_facade: Arc<ServiceFacade>, subscription_repository: S) -> Self {
        Self {
            subscription_repository,
            service_facade,
        }
    }

    pub async fn subscriptions(&self) -> Vec<Subscriptions> {
        self.subscription_repository.get_all().await
    }

    pub async fn subscribe(&self, new_subscription: Subscription) {
        if new_subscription.simulation_id.is_none() {
            debug!("Subscribe: {}", new_subscription.deployment_id);
            for new_instrument in new_subscription.instruments {
                let subscription = self.subscription_repository.get_be_instrument(&new_instrument).await;
                if let Some(mut subscription) = subscription {
                    subscription.deployment_ids.insert(new_subscription.deployment_id);
                } else {
                    self.service_facade.listen_orders(new_instrument.exchange).await;
                    self.service_facade.listen_position(new_instrument.exchange).await;
                    self.service_facade.subscribe_candles(&new_instrument).await;
                    self.service_facade.subscribe_ticks(&new_instrument).await;

                    let new_subscription = Subscriptions {
                        instrument_id: new_instrument,
                        deployment_ids: HashSet::from([new_subscription.deployment_id]),
                    };
                    self.subscription_repository.save(&new_subscription).await;
                }
            }
        }
    }

    pub async fn unsubscribe(&self, subscription: Subscription) {
        if subscription.simulation_id.is_none() {
            let deployment_id = subscription.deployment_id;
            debug!("Unsubscribe: {}", deployment_id);
            let updated_subscriptions: Vec<_> = self.subscription_repository.get_by_deployment(&deployment_id).await
                .into_iter()
                .map(|mut subscription| {
                    subscription.deployment_ids.retain(|id| !id.eq(&deployment_id));
                    subscription
                }).collect();
            self.subscription_repository.save_many(&updated_subscriptions).await;

            let service_facade = &self.service_facade;
            for subscription in updated_subscriptions {
                if subscription.deployment_ids.is_empty() {
                    service_facade.unsubscribe_candles(&subscription.instrument_id).await;
                    service_facade.unsubscribe_ticks(&subscription.instrument_id).await;
                    self.subscription_repository.delete(&subscription.instrument_id).await;
                }
            }
        }
    }
}