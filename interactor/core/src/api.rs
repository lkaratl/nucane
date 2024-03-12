use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{debug, trace, warn};

use domain_model::{Action, Candle, Exchange, InstrumentId, Order, OrderAction, OrderActionType, Subscription, Subscriptions, Timeframe};
use interactor_core_api::InteractorApi;
use interactor_exchange_api::ExchangeApi;
use interactor_persistence_api::SubscriptionRepository;
use storage_core_api::StorageApi;

use crate::exchanges::ServiceFacade;
use crate::services::SubscriptionManager;

pub struct Interactor<S: StorageApi, R: SubscriptionRepository> {
    service_facade: Arc<ServiceFacade<S>>,
    subscription_manager: SubscriptionManager<S, R>,
}

impl<S: StorageApi, R: SubscriptionRepository> Interactor<S, R> {
    pub fn new(
        storage_client: Arc<S>,
        subscription_repository: R,
        exchanges: Vec<Box<dyn ExchangeApi>>,
    ) -> Self {
        let service_facade = Arc::new(ServiceFacade::new(storage_client, exchanges));
        let subscription_manager =
            SubscriptionManager::new(Arc::clone(&service_facade), subscription_repository);
        Self {
            service_facade,
            subscription_manager,
        }
    }
}

#[async_trait]
impl<S: StorageApi, R: SubscriptionRepository> InteractorApi for Interactor<S, R> {
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
            let simulation_id = match &action {
                Action::OrderAction(order_action) => order_action.simulation_id,
            };
            if simulation_id.is_none() {
                debug!("Retrieved new action event");
                trace!("Action event: {action:?}");
                match action {
                    Action::OrderAction(OrderAction {
                                            order: OrderActionType::CreateOrder(create_order),
                                            exchange,
                                            ..
                                        }) => {
                        self.service_facade
                            .place_order(exchange, create_order)
                            .await
                    }
                    Action::OrderAction(OrderAction {
                                            order: OrderActionType::CancelOrder(cancel_order),
                                            exchange,
                                            ..
                                        }) => {
                        self.service_facade
                            .cancel_order(exchange, cancel_order)
                            .await
                    }
                    action => warn!("Temporary unsupported action: {action:?}"),
                }
            }
        }
        Ok(())
    }

    async fn get_candles(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u8>,
    ) -> Result<Vec<Candle>> {
        let candles = self
            .service_facade
            .candles_history(instrument_id, timeframe, from, to, limit)
            .await;
        Ok(candles)
    }

    async fn get_price(
        &self,
        instrument_id: &InstrumentId,
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<f64> {
        let price = self.service_facade.price(instrument_id, timestamp).await;
        Ok(price)
    }

    async fn get_order(&self, exchange: Exchange, order_id: &str) -> Result<Option<Order>> {
        let order = self.service_facade.order(exchange, order_id).await;
        Ok(order)
    }

    async fn get_total_balance(&self, exchange: Exchange) -> Result<f64> {
        let total_balance = self.service_facade.total_balance(exchange).await;
        Ok(total_balance)
    }
}
