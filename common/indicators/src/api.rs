use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use ta::indicators::ExponentialMovingAverage;
use ta::Next;

use domain_model::{InstrumentId, Timeframe};
use storage_core_api::StorageApi;

use crate::calculation::simple_moving_average;

pub struct Indicators<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> Indicators<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }

    async fn get_prices(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64) -> Vec<f64> {
        let from = timestamp - Duration::seconds(timeframe.as_sec() * period as i64);
        let candles = self
            .storage_client
            .get_candles(
                instrument_id,
                Some(timeframe),
                Some(from),
                Some(timestamp),
                Some(period),
            )
            .await
            .unwrap();
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
}
