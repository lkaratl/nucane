mod calculation;

use std::sync::Arc;
use domain_model::{InstrumentId, Timeframe};
use storage_core_api::StorageApi;
use crate::calculation::moving_average;

pub struct Indicators {
    storage_client: Arc<dyn StorageApi>,
}

impl Indicators {
    pub fn new(storage_client: Arc<dyn StorageApi>) -> Self {
        Self {
            storage_client
        }
    }

    pub async fn moving_average(&self, instrument_id: &InstrumentId, timeframe: Timeframe, length: usize) -> f64 {
        let candles = self.storage_client.get_candles(instrument_id,
                                                      Some(timeframe),
                                                      None,
                                                      None,
                                                      Some(length as u64))
            .await
            .unwrap();
        let values: Vec<_> = candles.into_iter()
            .map(|candle| candle.close_price)
            .collect();
        *moving_average(&values, length)
            .unwrap()
            .first()
            .unwrap()
    }
}
