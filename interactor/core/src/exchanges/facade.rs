use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};

use domain_model::{Candle, CreateOrder, Exchange, InstrumentId, Timeframe};
use storage_core_api::StorageApi;

use crate::exchanges::exchange::ExchangeApi;
use crate::exchanges::okx::handlers::{CandleHandler, OrderHandler, PositionHandler};
use crate::exchanges::okx::OkxService;

#[derive(Default)]
pub struct ServiceFacade<S: StorageApi> {
    storage_client: Arc<S>,
    okx: OkxService,
}

impl<S: StorageApi> ServiceFacade<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self {
            storage_client,
            okx: Default::default(),
        }
    }

    pub async fn subscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!("Subscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        // service.subscribe_ticks(&instrument_id.pair, &instrument_id.market_type, |tick| {
        //     trace!("Send tick to synapse: {tick:?}");
        //     debug!("{}={:?}-{:?}: {}",
        //         tick.instrument_id.exchange,
        //         tick.instrument_id.pair.target,
        //         tick.instrument_id.pair.source,
        //         tick.price);
        //     // self.synapse_sender.send_message(ExchangeTickSubject, &tick).await.expect("Error during tick sending to synapse"); // todo use rest client
        // }).await;
    }

    pub async fn unsubscribe_ticks(&self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        service.unsubscribe_ticks(&instrument_id.pair, &instrument_id.market_type).await;
    }


    pub async fn subscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!("Subscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        let currency_pair = instrument_id.pair;
        let market_type = instrument_id.market_type;
        let storage_client = Arc::clone(&self.storage_client);
        let handler = CandleHandler::new(currency_pair, market_type, storage_client);
        service.subscribe_candles(&currency_pair, &market_type, handler).await;
    }

    pub async fn unsubscribe_candles(&self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        service.unsubscribe_candles(&instrument_id.pair, &instrument_id.market_type).await;
    }

    pub async fn listen_orders(&self, exchange: Exchange) {
        debug!("Start listening order events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &self.okx,
        };
        let storage_client = Arc::clone(&self.storage_client);
        let handler = OrderHandler::new(storage_client);
        service.listen_orders(handler).await;
    }

    pub async fn listen_position(&self, exchange: Exchange) {
        debug!("Start listening account events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &self.okx,
        };
        let storage_client = Arc::clone(&self.storage_client);
        let handler = PositionHandler::new(storage_client);
        service.listen_positions(handler).await;
    }

    pub async fn place_order(&self, exchange: Exchange, create_order: CreateOrder) {
        info!("Placing new order with id: '{}' for exchange: '{exchange}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            create_order.id, create_order.market_type, create_order.pair.target, create_order.pair.source, create_order.order_type, create_order.stop_loss, create_order.take_profit);
        let service = match exchange {
            Exchange::OKX => &self.okx,
        };
        let order = service.place_order(&create_order).await;
        self.storage_client.save_order(order).await;
    }

    pub async fn candles_history(&self,
                                 instrument_id: &InstrumentId,
                                 timeframe: Timeframe,
                                 from_timestamp: Option<DateTime<Utc>>,
                                 to_timestamp: Option<DateTime<Utc>>,
                                 limit: Option<u8>) -> Vec<Candle> {
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        service.candles_history(&instrument_id.pair, &instrument_id.market_type, timeframe, from_timestamp, to_timestamp, limit).await
    }

    pub async fn price(&self, instrument_id: &InstrumentId, timestamp: Option<DateTime<Utc>>) -> f64 {
        let service = match instrument_id.exchange {
            Exchange::OKX => &self.okx,
        };
        let timestamp = timestamp.unwrap_or(Utc::now());
        let from = timestamp - Duration::seconds(1);
        let to = timestamp + Duration::seconds(1);
        service.candles_history(&instrument_id.pair, &instrument_id.market_type, Timeframe::OneS, Some(from), Some(to), Some(1))
            .await
            .first()
            .unwrap()
            .open_price
    }
}
