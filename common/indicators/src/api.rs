use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use domain_model::{InstrumentId, Timeframe};
use storage_core_api::StorageApi;

use crate::calculation::moving_average;

pub struct Indicators<S: StorageApi> {
    storage_client: Arc<S>,
}

impl<S: StorageApi> Indicators<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        Self { storage_client }
    }

    pub async fn moving_average(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        timestamp: DateTime<Utc>,
        length: u64,
    ) -> f64 {
        let from = timestamp - Duration::seconds(timeframe.as_sec() * length as i64);
        let candles = self
            .storage_client
            .get_candles(
                instrument_id,
                Some(timeframe),
                Some(from),
                Some(timestamp),
                Some(length),
            )
            .await
            .unwrap();
        let values: Vec<_> = candles
            .into_iter()
            .map(|candle| candle.close_price)
            .collect();
        *moving_average(&values, length).unwrap().get(length as usize).unwrap()
    }
}
