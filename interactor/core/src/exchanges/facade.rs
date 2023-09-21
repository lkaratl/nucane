use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};
use tracing::trace;

use domain_model::{Candle, CreateOrder, Exchange, InstrumentId, Timeframe};

use crate::exchanges::okx::OkxService;

#[derive(Default)]
pub struct ServiceFacade {
    okx: OkxService,
}

impl ServiceFacade {
    pub fn new() -> Self {
        Self {
            okx: Default::default(),
        }
    }

    pub async fn subscribe_ticks(&mut self, instrument_id: &InstrumentId) {
        debug!("Subscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.subscribe_ticks(&instrument_id.pair, &instrument_id.market_type, |tick| {
            trace!("Send tick to synapse: {tick:?}");
            debug!("{}={:?}-{:?}: {}",
                tick.instrument_id.exchange,
                tick.instrument_id.pair.target,
                tick.instrument_id.pair.source,
                tick.price);
            self.synapse_sender.send_message(ExchangeTickSubject, &tick).await.expect("Error during tick sending to synapse");
        }).await;
    }

    pub fn unsubscribe_ticks(&mut self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe ticks for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.unsubscribe_ticks(&instrument_id.pair, &instrument_id.market_type);
    }


    pub async fn subscribe_candles(&mut self, instrument_id: &InstrumentId) {
        debug!("Subscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.subscribe_candles(&instrument_id.pair, &instrument_id.market_type, |candle| async {
            trace!("Send candle to synapse: {candle:?}");
            self.synapse_sender.send_message(ExchangeCandleSubject, &candle).await.expect("Error during candle sending to synapse");
        }).await;
    }

    pub fn unsubscribe_candles(&mut self, instrument_id: &InstrumentId) {
        debug!("Unsubscribe candles for instrument: '{}-{}-{}', exchange: '{}'",
            instrument_id.pair.target, instrument_id.pair.source, instrument_id.market_type, instrument_id.exchange);
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.unsubscribe_candles(&instrument_id.pair, &instrument_id.market_type);
    }

    pub async fn listen_orders(&mut self, exchange: Exchange) {
        debug!("Start listening order events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.listen_orders(|order| async {
            debug!("Retrieved new order with id: '{}' from exchange: '{}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            order.id, order.exchange, order.market_type, order.pair.target, order.pair.source, order.order_type, order.stop_loss, order.take_profit);
            trace!("Send order to synapse: {order:?}");
            self.synapse_sender.send_message(ExchangeOrderSubject, &order).await.expect("Error during order sending to synapse");
        }).await;
    }

    pub async fn listen_position(&mut self, exchange: Exchange) {
        debug!("Start listening account events for exchange: '{exchange}'");
        let service = match exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.listen_positions(|position| async {
            debug!("Retrieved account position update from exchange: '{}', currency: '{}', size: '{}'",
                  position.exchange, position.currency, position.size);
            trace!("Send position to synapse: {position:?}");
            self.synapse_sender.send_message(ExchangePositionSubject, &position).await.expect("Error during position sending to synapse");
        }).await;
    }

    pub async fn place_order(&mut self, exchange: Exchange, create_order: CreateOrder) {
        info!("Placing new order with id: '{}' for exchange: '{exchange}', market type: '{:?}', pair: '{}-{}', order type: '{:?}', stop-loss: '{:?}', take-profit: '{:?}'",
            create_order.id, create_order.market_type, create_order.pair.target, create_order.pair.source, create_order.order_type, create_order.stop_loss, create_order.take_profit);
        let service = match exchange {
            Exchange::OKX => &mut self.okx,
        };
        let order = service.place_order(&create_order).await;
        self.synapse_sender.send_message(ExchangeOrderSubject, &order).await.expect("Error during placed order sending to synapse");
    }

    pub async fn candles_history(&mut self,
                                 instrument_id: &InstrumentId,
                                 timeframe: Timeframe,
                                 from_timestamp: Option<DateTime<Utc>>,
                                 to_timestamp: Option<DateTime<Utc>>,
                                 limit: Option<u8>) -> Vec<Candle> {
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
        };
        service.candles_history(&instrument_id.pair, &instrument_id.market_type, timeframe, from_timestamp, to_timestamp, limit).await
    }

    pub async fn price(&mut self, instrument_id: &InstrumentId, timestamp: DateTime<Utc>) -> f64 {
        let service = match instrument_id.exchange {
            Exchange::OKX => &mut self.okx,
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
