use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};
use tracing::trace;

use domain_model::{Candle, CreateOrder, CurrencyPair, Exchange, InstrumentId, MarketType, Order, Position, Tick, Timeframe};
use interactor_config::CONFIG;

use crate::service::okx::OKXService;

mod okx;

#[async_trait]
pub trait Service {
    async fn subscribe_ticks<T: Fn(Tick) + Send + 'static>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    fn unsubscribe_ticks(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn subscribe_candles<T: Fn(Candle) + Send + 'static>(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, callback: T);
    fn unsubscribe_candles(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType);
    async fn listen_orders<T: Fn(Order) + Send + 'static>(&mut self, callback: T);
    async fn listen_positions<T: Fn(Position) + Send + 'static>(&mut self, callback: T);
    async fn place_order(&mut self, create_order: &CreateOrder) -> Order;
    async fn candles_history(&mut self, currency_pair: &CurrencyPair, market_type: &MarketType, timeframe: Timeframe, before: Option<DateTime<Utc>>, after: Option<DateTime<Utc>>, limit: Option<u8>) -> Vec<Candle>;
}

#[derive(Default)]
pub struct ServiceFacade {
    okx_service: OKXService,
}

impl ServiceFacade {
    pub async fn subscribe_ticks(&mut self, instrument_id: &InstrumentId) {
        debug!("Subscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.subscribe_ticks(&instrument_id.pair, &instrument_id.market_type, |tick| {
            trace!("Send tick to synapse: {tick:?}");
            debug!("{}={:?}-{:?}: {}",
                tick.instrument_id.exchange,
                tick.instrument_id.pair.target,
                tick.instrument_id.pair.source,
                tick.price);
            synapse::writer(&CONFIG.broker.url).send(&tick);
        }).await;
    }

    pub fn unsubscribe_ticks(&mut self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.unsubscribe_ticks(&instrument_id.pair, &instrument_id.market_type);
    }


    pub async fn subscribe_candles(&mut self, instrument_id: &InstrumentId) {
        debug!("Subscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.subscribe_candles(&instrument_id.pair, &instrument_id.market_type, |candle| {
            trace!("Send candle to synapse: {candle:?}");
            synapse::writer(&CONFIG.broker.url).send(&candle);
        }).await;
    }

    pub fn unsubscribe_candles(&mut self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.unsubscribe_candles(&instrument_id.pair, &instrument_id.market_type);
    }

    pub async fn listen_orders(&mut self, exchange: Exchange) {
        debug!("Start listening order events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.listen_orders(|order| {
            debug!("Retrieved new order with id: '{}' from exchange: '{}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            order.id, order.exchange, order.market_type, order.pair.target, order.pair.source, order.order_type, order.stop_loss, order.take_profit);
            trace!("Send order to synapse: {order:?}");
            synapse::writer(&CONFIG.broker.url).send(&order);
        }).await;
    }

    pub async fn listen_position(&mut self, exchange: Exchange) {
        debug!("Start listening account events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.listen_positions(|position| {
            debug!("Retrieved account position update from exchange: '{}', currency: '{}', size: '{}'",
                  position.exchange, position.currency, position.size);
            trace!("Send position to synapse: {position:?}");
            synapse::writer(&CONFIG.broker.url).send(&position);
        }).await;
    }

    pub async fn place_order(&mut self, exchange: Exchange, create_order: CreateOrder) {
        info!("Placing new order with id: '{}' for exchange: '{exchange}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            create_order.id, create_order.market_type, create_order.pair.target, create_order.pair.source, create_order.order_type, create_order.stop_loss, create_order.take_profit);
        let service = match exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        let order = service.place_order(&create_order).await;
        synapse::writer(&CONFIG.broker.url).send(&order);
    }

    pub async fn candles_history(&mut self,
                                 instrument_id: &InstrumentId,
                                 timeframe: Timeframe,
                                 from_timestamp: Option<DateTime<Utc>>,
                                 to_timestamp: Option<DateTime<Utc>>,
                                 limit: Option<u8>) -> Vec<Candle> {
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        service.candles_history(&instrument_id.pair, &instrument_id.market_type, timeframe, from_timestamp, to_timestamp, limit).await
    }

    pub async fn price(&mut self, instrument_id: &InstrumentId, timestamp: DateTime<Utc>) -> f64 {
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx_service,
        };
        let from = timestamp - Duration::seconds(1);
        let to = timestamp + Duration::seconds(1);
        service.candles_history(&instrument_id.pair, &instrument_id.market_type, Timeframe::OneS, Some(from), Some(to), Some(1))
            .await
            .first()
            .unwrap()
            .open_price
    }
}
