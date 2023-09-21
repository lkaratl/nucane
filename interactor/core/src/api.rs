use domain_model::{Action, Candle, InstrumentId, OrderAction, OrderActionType, Subscription, Timeframe};
use interactor_api::InteractorApi;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{debug, trace, warn};
use crate::exchanges::ServiceFacade;
use crate::services::SubscriptionManager;

#[derive(Clone)]
pub struct Interactor {
    service_facade: ServiceFacade,
    subscription_manager: SubscriptionManager,
}

impl Interactor {
    pub fn new(service_facade: ServiceFacade,
               subscription_manager: SubscriptionManager, ) -> Self {
        Self {
            service_facade,
            subscription_manager,
        }
    }
}

impl InteractorApi for Interactor {
    fn subscribe(&self, subscription: Subscription) -> Result<()> {
        self.subscription_manager.subscribe(subscription);
        Ok(())
    }

    fn unsubscribe(&self, subscription: Subscription) -> Result<()> {
        self.subscription_manager.unsubscribe(subscription);
        Ok(())
    }

    fn execute_actions(&self, actions: Vec<Action>) -> Result<()> {
        for action in actions {
            let simulation_id = match &action { Action::OrderAction(order_action) => order_action.simulation_id };
            if simulation_id.is_none() {
                debug!("Retrieved new action event");
                trace!("Action event: {action:?}");
                match action {
                    Action::OrderAction(OrderAction { order: OrderActionType::CreateOrder(create_order), exchange, .. }) =>
                        self.service_facade
                            .lock()
                            .await
                            .place_order(exchange, create_order)
                            .await,
                    action => warn!("Temporary unsupported action: {action:?}")
                }
            }
        }
        Ok(())
    }

    fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u8>) -> Result<Vec<Candle>> {
        let candles = self.service_facade.candles_history(instrument_id, timeframe, from, to, limit);
        Ok(candles)
    }

    fn get_price(&self, instrument_id: &InstrumentId, timestamp: DateTime<Utc>) -> Result<f64> {
        let price = self.service_facade.price(instrument_id, timestamp);
        Ok(price)
    }
}
