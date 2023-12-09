use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{from_value, Value};
use tracing::trace;

use domain_model::{
    Candle, CandleStatus, CurrencyPair, Exchange, InstrumentId, MarketType, Timeframe,
};
use eac::okx::rest::CandleResponse;
use eac::okx::websocket::{Action, Channel, WsMessageHandler};
use storage_core_api::StorageApi;

pub struct CandleHandler<S: StorageApi> {
    currency_pair: CurrencyPair,
    market_type: MarketType,
    storage_client: Arc<S>,
}

impl<S: StorageApi> CandleHandler<S> {
    pub fn new(
        currency_pair: CurrencyPair,
        market_type: MarketType,
        storage_client: Arc<S>,
    ) -> Self {
        Self {
            currency_pair,
            market_type,
            storage_client,
        }
    }
}

#[async_trait]
impl<S: StorageApi> WsMessageHandler for CandleHandler<S> {
    type Type = Vec<Candle>;

    async fn convert_data(
        &mut self,
        arg: Channel,
        _action: Option<Action>,
        data: Vec<Value>,
    ) -> Option<Self::Type> {
        trace!("Retrieved massage with raw payload: {:?}", &data);
        let mut candles = Vec::new();
        for item in data {
            let candle_message: CandleResponse = from_value(item).unwrap();
            let timeframe = match arg {
                Channel::Candle1M { .. } => Timeframe::OneM,
                Channel::Candle5M { .. } => Timeframe::FiveM,
                Channel::Candle15M { .. } => Timeframe::FifteenM,
                Channel::Candle30M { .. } => Timeframe::ThirtyM,
                Channel::Candle1H { .. } => Timeframe::OneH,
                Channel::Candle2H { .. } => Timeframe::TwoH,
                Channel::Candle4H { .. } => Timeframe::FourH,
                Channel::Candle1D { .. } => Timeframe::OneD,
                channel => panic!(
                    "Error during timeframe parsing for candle, unexpected channel: {channel:?}"
                ),
            };
            let id = format!(
                "{}_{}_{}_{}_{}_{}",
                Exchange::OKX,
                self.market_type,
                self.currency_pair.target,
                self.currency_pair.source,
                timeframe,
                candle_message.0.timestamp_millis()
            );
            let status = match candle_message.8.as_str() {
                "0" => CandleStatus::Open,
                "1" => CandleStatus::Close,
                status => panic!("Error during candle status parsing, unexpected status: {status}"),
            };
            let instrument_id = InstrumentId {
                exchange: Exchange::OKX,
                market_type: self.market_type,
                pair: self.currency_pair,
            };
            let candle = Candle {
                id,
                status,
                instrument_id,
                timestamp: candle_message.0,
                timeframe,
                open_price: candle_message.1,
                highest_price: candle_message.2,
                lowest_price: candle_message.3,
                close_price: candle_message.4,
                target_volume: candle_message.6,
                source_volume: candle_message.7,
            };
            candles.push(candle);
        }
        Some(candles)
    }

    async fn handle(&mut self, message: Self::Type) {
        for candle in message {
            self.storage_client.save_candle(candle).await.unwrap();
        }
    }
}
