use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{from_value, Value};
use tracing::trace;
use uuid::Uuid;

use domain_model::{CurrencyPair, Exchange, InstrumentId, MarketType, Tick};
use eac::rest::MarkPriceResponse;
use eac::websocket::{Action, Channel, WsMessageHandler};
use engine_core_api::api::EngineApi;

const TICK_PRICE_DEVIATION_MULTIPLIER: f64 = 1000.0;
const TICK_PRICE_THRESHOLD: f64 = 2.0;

pub struct TickHandler<E: EngineApi> {
    deviation_percent: f64,
    currency_pair: CurrencyPair,
    market_type: MarketType,
    engine_client: Arc<E>,
}

impl<E: EngineApi> TickHandler<E> {
    pub fn new(engine_client: Arc<E>, currency_pair: CurrencyPair, market_type: MarketType) -> Self {
        Self {
            deviation_percent: 1f64,
            currency_pair,
            market_type,
            engine_client,
        }
    }
}

#[async_trait]
impl<E: EngineApi> WsMessageHandler for TickHandler<E> {
    type Type = Tick;

    async fn convert_data(&mut self, _arg: Channel, _action: Option<Action>, mut data: Vec<Value>) -> Option<Self::Type> {
        trace!("Retrieved massage with raw payload: {:?}", &data);
        let data = data.pop().unwrap();
        let mark_price: MarkPriceResponse = from_value(data).unwrap();

        let price = mark_price.mark_px;
        let deviation = price / self.deviation_percent - TICK_PRICE_DEVIATION_MULTIPLIER;
        if !(TICK_PRICE_THRESHOLD * -1.0..=TICK_PRICE_THRESHOLD).contains(&deviation) {
            self.deviation_percent = price / TICK_PRICE_DEVIATION_MULTIPLIER;
            let tick = Tick {
                id: Uuid::new_v4(),
                simulation_id: None,
                timestamp: mark_price.ts,
                instrument_id: InstrumentId {
                    exchange: Exchange::OKX,
                    market_type: self.market_type,
                    pair: self.currency_pair,
                },
                price,
            };
            Some(tick)
        } else {
            None
        }
    }

    async fn handle(&mut self, message: Self::Type) {
        let _ = self.engine_client.get_actions(&message).await;
    }
}
