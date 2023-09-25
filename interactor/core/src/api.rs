use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{debug, trace, warn};

use domain_model::{Action, Candle, InstrumentId, OrderAction, OrderActionType, Subscription, Subscriptions, Timeframe};
use interactor_core_api::InteractorApi;
use interactor_persistence_api::SubscriptionRepository;

use crate::exchanges::ServiceFacade;
use crate::services::SubscriptionManager;

pub struct Interactor<S: SubscriptionRepository> {
    service_facade: Arc<ServiceFacade>,
    subscription_manager: SubscriptionManager<S>,
}

impl<S: SubscriptionRepository> Interactor<S> {
    pub fn new(subscription_repository: S) -> Self {
        let service_facade = Arc::new(ServiceFacade::new());
        let subscription_manager = SubscriptionManager::new(Arc::clone(&service_facade), subscription_repository);
        Self {
            service_facade,
            subscription_manager,
        }
    }
}

#[async_trait]
impl<S: SubscriptionRepository> InteractorApi for Interactor<S> {
    async fn subscriptions(&self) -> Result<Vec<Subscriptions>> {
        Ok(self.subscription_manager.subscriptions().await)
    }

    async fn subscribe(&self, subscription: Subscription) -> Result<()> {
        self.subscription_manager.subscribe(subscription).await;
        Ok(())
    }

    async fn unsubscribe(&self, subscription: Subscription) -> Result<()> {
        self.subscription_manager.unsubscribe(subscription).await;
        Ok(())
    }

    async fn execute_actions(&self, actions: Vec<Action>) -> Result<()> {
        for action in actions {
            let simulation_id = match &action { Action::OrderAction(order_action) => order_action.simulation_id };
            if simulation_id.is_none() {
                debug!("Retrieved new action event");
                trace!("Action event: {action:?}");
                match action {
                    Action::OrderAction(OrderAction { order: OrderActionType::CreateOrder(create_order), exchange, .. }) =>
                        self.service_facade
                            .place_order(exchange, create_order)
                            .await,
                    action => warn!("Temporary unsupported action: {action:?}")
                }
            }
        }
        Ok(())
    }

    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u8>) -> Result<Vec<Candle>> {
        let candles = self.service_facade.candles_history(instrument_id, timeframe, from, to, limit).await;
        Ok(candles)
    }

    async fn get_price(&self, instrument_id: &InstrumentId, timestamp: Option<DateTime<Utc>>) -> Result<f64> {
        let price = self.service_facade.price(instrument_id, timestamp).await;
        Ok(price)
    }
}