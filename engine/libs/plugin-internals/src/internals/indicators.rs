use std::sync::Arc;

use async_trait::async_trait;

use domain_model::{InstrumentId, Timeframe};
use indicators_api::Indicators;
use plugin_api::IndicatorsInternalApi;
use storage_core_api::StorageApi;

pub struct DefaultIndicatorInternals<S: StorageApi> {
    indicators: Indicators<S>,
}

impl<S: StorageApi> DefaultIndicatorInternals<S> {
    pub fn new(storage_client: Arc<S>) -> Self {
        let indicators = Indicators::new(storage_client);
        Self { indicators }
    }
}

#[async_trait]
impl<S: StorageApi> IndicatorsInternalApi for DefaultIndicatorInternals<S> {
    async fn moving_avg(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        length: u16,
    ) -> f64 {
        self.indicators
            .moving_average(instrument_id, timeframe, length)
            .await
    }
}
