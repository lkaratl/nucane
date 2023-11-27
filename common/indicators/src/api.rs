use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use ta::indicators::{BollingerBands, BollingerBandsOutput, ExponentialMovingAverage};
use ta::Next;
use yata::core::{Action, IndicatorConfig, IndicatorInstance};

use domain_model::{Candle, InstrumentId, Side, Timeframe};
use storage_core_api::StorageApi;

use crate::calculation::simple_moving_average;

pub struct Indicators<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> Indicators<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }

    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64) -> Vec<Candle> {
        let from = timestamp - Duration::seconds(timeframe.as_sec() * period as i64);
        self.storage_client
            .get_candles(
                instrument_id,
                Some(timeframe),
                Some(from),
                Some(timestamp),
                Some(period),
            )
            .await
            .unwrap()
    }

    async fn get_prices(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64) -> Vec<f64> {
        let candles = self.get_candles(instrument_id, timeframe, timestamp, period).await;
        candles.into_iter()
            .map(|candle| candle.close_price)
            .collect()
    }

    pub async fn simple_moving_average(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        timestamp: DateTime<Utc>,
        period: u64,
    ) -> f64 {
        let prices = self.get_prices(instrument_id, timeframe, timestamp, period).await;
        *simple_moving_average(&prices, period).unwrap().get(period as usize).unwrap()
    }

    pub async fn exponential_moving_average(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        timestamp: DateTime<Utc>,
        period: u64,
    ) -> f64 {
        let mut prices = self.get_prices(instrument_id, timeframe, timestamp, period * 2).await;
        prices.reverse();
        let mut ema = ExponentialMovingAverage::new(period as usize).unwrap();
        let mut result = 0.;
        for price in prices {
            result = ema.next(price);
        }
        result
    }

    pub async fn bollinger_bands(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>,
                                 period: u64, multiplier: f64) -> BollingerBand {
        let values = self.get_prices(instrument_id, timeframe, timestamp, period).await;
        let mut bb = BollingerBands::new(period as usize, multiplier).unwrap();
        let mut result = BollingerBand::default();
        for value in values {
            result = bb.next(value).into()
        }
        result
    }

    pub async fn parabolic_sar(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>) -> Option<Side> {
        let candles: Vec<_> = self.get_candles(instrument_id, timeframe, timestamp, 100).await
            .into_iter()
            .map(|candle| (candle.open_price, candle.highest_price, candle.lowest_price, candle.close_price, 0.))
            .collect();

        let psar = yata::indicators::ParabolicSAR::default();
        let mut instance = psar.init(candles.first().unwrap()).unwrap();
        let mut signals = Vec::new();
        for candle in candles.iter().skip(1) {
            signals = instance.next(candle).signals().to_vec();
        }
        let mut result = None;
        for signal in signals.iter() {
            match signal {
                Action::Buy(_) => {
                    result = Some(Side::Buy);
                    break;
                },
                Action::Sell(_) => {
                    result = Some(Side::Sell);
                    break;
                },
                Action::None => {}
            }
        }
        result
    }
}

#[derive(Default, Debug)]
pub struct BollingerBand {
    pub upper: f64,
    pub average: f64,
    pub lower: f64,
}

impl From<BollingerBandsOutput> for BollingerBand {
    fn from(value: BollingerBandsOutput) -> Self {
        Self {
            upper: value.upper,
            average: value.average,
            lower: value.lower,
        }
    }
}
