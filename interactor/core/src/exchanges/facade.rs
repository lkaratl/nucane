use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};

use domain_model::{Candle, CreateOrder, Exchange, InstrumentId, Timeframe};
use engine_core_api::api::EngineApi;
use interactor_exchange_api::ExchangeApi;
use storage_core_api::StorageApi;

#[derive(Default)]
pub struct ServiceFacade<E: EngineApi, S: StorageApi> {
    engine_client: Arc<E>,
    storage_client: Arc<S>,
    exchanges: Vec<Box<dyn ExchangeApi>>,
}

impl<E: EngineApi, S: StorageApi> ServiceFacade<E, S> {
    pub fn new(engine_client: Arc<E>, storage_client: Arc<S>, exchanges: Vec<Box<dyn ExchangeApi>>) -> Self {
        Self {
            engine_client,
            storage_client,
            exchanges,
        }
    }

    fn get_exchange(&self, id: Exchange) -> &Box<dyn ExchangeApi> {
        self.exchanges.iter().find(|exchange| exchange.id() == id).unwrap()
    }

    pub async fn subscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!("Subscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let exchange = self.get_exchange(instrument_id.exchange);
        let currency_pair = instrument_id.pair;
        let market_type = instrument_id.market_type;
        exchange.subscribe_ticks(&currency_pair, &market_type).await;
    }

    pub async fn unsubscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange.unsubscribe_ticks(&instrument_id.pair, &instrument_id.market_type).await;
    }


    pub async fn subscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!("Subscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let exchange = self.get_exchange(instrument_id.exchange);
        let currency_pair = instrument_id.pair;
        let market_type = instrument_id.market_type;
        exchange.subscribe_candles(&currency_pair, &market_type).await;
    }

    pub async fn unsubscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange.unsubscribe_candles(&instrument_id.pair, &instrument_id.market_type).await;
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
        let _ = self.storage_client.save_order(order).await;
    }

    pub async fn candles_history(&self,
                                 instrument_id: &InstrumentId,
                                 timeframe: Timeframe,
                                 from_timestamp: Option<DateTime<Utc>>,
                                 to_timestamp: Option<DateTime<Utc>>,
                                 limit: Option<u8>) -> Vec<Candle> {
        let exchange = self.get_exchange(instrument_id.exchange);
        exchange.candles_history(&instrument_id.pair, &instrument_id.market_type, timeframe, from_timestamp, to_timestamp, limit).await
    }

    pub async fn price(&self, instrument_id: &InstrumentId, timestamp: Option<DateTime<Utc>>) -> f64 {
        let exchange = self.get_exchange(instrument_id.exchange);
        let timestamp = timestamp.unwrap_or(Utc::now());
        let from = timestamp - Duration::seconds(1);
        let to = timestamp + Duration::seconds(1);
        exchange.candles_history(&instrument_id.pair, &instrument_id.market_type, Timeframe::OneS, Some(from), Some(to), Some(1))
            .await
            .first()
            .unwrap()
            .open_price
    }
}
