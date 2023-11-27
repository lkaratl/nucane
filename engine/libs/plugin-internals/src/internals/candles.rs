use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{Candle, InstrumentId, Timeframe};
use plugin_api::CandlesInternalApi;
use storage_core_api::StorageApi;

pub struct DefaultCandleInternals<S: StorageApi> {
    timestamp: DateTime<Utc>,
    storage_client: Arc<S>,
}

impl<S: StorageApi> DefaultCandleInternals<S> {
    pub fn new(storage_client: Arc<S>, timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            storage_client,
        }
    }
}

#[async_trait]
impl<S: StorageApi> CandlesInternalApi for DefaultCandleInternals<S> {
    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, limit: u64) -> Vec<Candle> {
        self.storage_client.get_candles(instrument_id, Some(timeframe), None, Some(self.timestamp), Some(limit))
            .await
            .unwrap()
    }
}
