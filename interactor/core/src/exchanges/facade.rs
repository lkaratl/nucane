use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};

use domain_model::{CancelOrder, Candle, CreateOrder, Exchange, InstrumentId, Order, Timeframe};
use interactor_exchange_api::ExchangeApi;
use storage_core_api::StorageApi;

#[derive(Default)]
pub struct ServiceFacade<S: StorageApi> {
    storage_client: Arc<S>,
    exchanges: Vec<Box<dyn ExchangeApi>>,
}

impl<S: StorageApi> ServiceFacade<S> {
    pub fn new(storage_client: Arc<S>, exchanges: Vec<Box<dyn ExchangeApi>>) -> Self {
        Self {
            storage_client,
            exchanges,
        }
    }

    fn get_exchange(&self, id: Exchange) -> &dyn ExchangeApi {
        self.exchanges
            .iter()
            .find(|exchange| exchange.id() == id)
            .unwrap()
            .as_ref()
    }

    pub async fn subscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!(
            "Subscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target,
            instrument_id.pair.source,
            instrument_id.market_type,
            instrument_id.exchange
        );
        let exchange = self.get_exchange(instrument_id.exchange);
        let currency_pair = instrument_id.pair;
        let market_type = instrument_id.market_type;
        exchange.subscribe_ticks(&currency_pair, &market_type).await;
    }

    pub async fn unsubscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!(
            "Unsubscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target,
            instrument_id.pair.source,
            instrument_id.market_type,
            instrument_id.exchange
        );
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange
            .unsubscribe_ticks(&instrument_id.pair, &instrument_id.market_type)
            .await;
    }

    pub async fn subscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!(
            "Subscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target,
            instrument_id.pair.source,
            instrument_id.market_type,
            instrument_id.exchange
        );
        let exchange = self.get_exchange(instrument_id.exchange);
        let currency_pair = instrument_id.pair;
        let market_type = instrument_id.market_type;
        exchange
            .subscribe_candles(&currency_pair, &market_type)
            .await;
    }

    pub async fn unsubscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!(
            "Unsubscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target,
            instrument_id.pair.source,
            instrument_id.market_type,
            instrument_id.exchange
        );
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange
            .unsubscribe_candles(&instrument_id.pair, &instrument_id.market_type)
            .await;
    }

    pub async fn listen_orders(&self, exchange: Exchange) {
        debug!("Start listening order events for exchange: '{exchange}'");
        let exchange = self.get_exchange(exchange);
        exchange.listen_orders().await;
    }

    pub async fn listen_position(&self, exchange: Exchange) {
        debug!("Start listening account events for exchange: '{exchange}'");
        let exchange = self.get_exchange(exchange);
        exchange.listen_positions().await;
    }

    pub async fn place_order(&self, exchange: Exchange, create_order: CreateOrder) {
        info!("Placing new order with id: '{}' for exchange: '{exchange}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            create_order.id, create_order.market_type, create_order.pair.target, create_order.pair.source, create_order.order_type, create_order.stop_loss, create_order.take_profit);
        let exchange = self.get_exchange(exchange);
        let order = exchange.place_order(&create_order).await;
        self.storage_client.save_order(order).await.unwrap();
    }

    pub async fn cancel_order(&self, exchange: Exchange, cancel_order: CancelOrder) {
        info!("Cancel order with id: '{}' for exchange: '{exchange}', pair: '{}-{}'",
            cancel_order.id, cancel_order.pair.target, cancel_order.pair.source);
        let exchange = self.get_exchange(exchange);
        exchange.cancel_oder(cancel_order).await;
    }

    pub async fn candles_history(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: Option<u8>,
    ) -> Vec<Candle> {
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange
            .candles_history(
                &instrument_id.pair,
                &instrument_id.market_type,
                timeframe,
                from_timestamp,
                to_timestamp,
                limit,
            )
            .await
    }

    pub async fn price(
        &self,
        instrument_id: &InstrumentId,
        timestamp: Option<DateTime<Utc>>,
    ) -> f64 {
        let exchange = self.get_exchange(instrument_id.exchange);
        let timestamp = timestamp.unwrap_or(Utc::now());
        let from = timestamp - Duration::minutes(1);
        let to = timestamp + Duration::minutes(1);
        exchange
            .candles_history(
                &instrument_id.pair,
                &instrument_id.market_type,
                Timeframe::OneM,
                Some(from),
                Some(to),
                Some(1),
            )
            .await
            .first()
            .unwrap()
            .open_price
    }

    pub async fn order(&self, exchange: Exchange, order_id: &str) -> Option<Order> {
        let exchange = self.get_exchange(exchange);
        exchange.get_order(order_id).await
    }

    pub async fn total_balance(&self, exchange: Exchange) -> f64 {
        let exchange = self.get_exchange(exchange);
        exchange.get_total_balance().await
    }
}
