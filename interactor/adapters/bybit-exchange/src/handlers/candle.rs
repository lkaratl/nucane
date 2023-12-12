use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use serde_json::{from_value, Value};
use tracing::trace;

use domain_model::{
    Candle, CandleStatus, CurrencyPair, Exchange, InstrumentId, MarketType, Timeframe,
};
use eac::bybit::websocket::{CandleResponse, WsMessageHandler};
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

    async fn convert_data(&mut self, _topic: String, data: Value) -> Option<Self::Type> {
        trace!("Retrieved massage with raw payload: {:?}", &data);
        let mut candles = Vec::new();
        let candle_messages: Vec<CandleResponse> = from_value(data).unwrap();
        for item in candle_messages {
            let timeframe = match item.interval.as_str() {
                "1" => Timeframe::OneM,
                "5" => Timeframe::FiveM,
                "15" => Timeframe::FifteenM,
                "30" => Timeframe::ThirtyM,
                "60" => Timeframe::OneH,
                "120" => Timeframe::TwoH,
                "240" => Timeframe::FourH,
                "D" => Timeframe::OneD,
                channel => panic!(
                    "Error during timeframe parsing for candle, unexpected channel: {channel:?}"
                ),
            };
            let id = format!(
                "{}_{}_{}_{}_{}_{}",
                Exchange::BYBIT,
                self.market_type,
                self.currency_pair.target,
                self.currency_pair.source,
                timeframe,
                item.start
            );
            let status = if item.confirm {
                CandleStatus::Close
            } else {
                CandleStatus::Open
            };
            let instrument_id = InstrumentId {
                exchange: Exchange::BYBIT,
                market_type: self.market_type,
                pair: self.currency_pair,
            };
            let timestamp = Utc.timestamp_millis_opt(item.start as i64).unwrap();
            let candle = Candle {
                id,
                status,
                instrument_id,
                timestamp,
                timeframe,
                open_price: item.open,
                highest_price: item.high,
                lowest_price: item.low,
                close_price: item.close,
                target_volume: item.turnover,
                source_volume: item.volume,
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
