use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{InstrumentId, Timeframe};
use indicators::Indicators;
use plugin_api::IndicatorsInternalApi;
use storage_core_api::StorageApi;

pub struct DefaultIndicatorInternals<S: StorageApi> {
    timestamp: DateTime<Utc>,
    indicators: Indicators<S>,
}

impl<S: StorageApi> DefaultIndicatorInternals<S> {
    pub fn new(storage_client: Arc<S>, timestamp: DateTime<Utc>) -> Self {
        let indicators = Indicators::new(storage_client);
        Self {
            indicators,
            timestamp,
        }
    }
}

#[async_trait]
impl<S: StorageApi> IndicatorsInternalApi for DefaultIndicatorInternals<S> {
    async fn moving_avg(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        length: u64,
    ) -> f64 {
        self.indicators
            .moving_average(instrument_id, timeframe, self.timestamp, length)
            .await
    }
}
