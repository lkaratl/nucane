use async_trait::async_trait;
use chrono::{DateTime, Utc};

use domain_model::{InstrumentId, Side, Timeframe};
use indicators::api::{BollingerBand, IndicatorsApi};
use plugin_api::IndicatorsInternalApi;

pub struct DefaultIndicatorInternals<I: IndicatorsApi> {
    timestamp: DateTime<Utc>,
    indicators: I,
}

impl<I: IndicatorsApi> DefaultIndicatorInternals<I> {
    pub fn new(indicators: I, timestamp: DateTime<Utc>) -> Self {
        Self {
            indicators,
            timestamp,
        }
    }
}

#[async_trait]
impl<I: IndicatorsApi + Sync + Send> IndicatorsInternalApi for DefaultIndicatorInternals<I> {
    async fn sma(&self, instrument_id: &InstrumentId, timeframe: Timeframe, period: u64) -> f64 {
        self.indicators
            .simple_moving_average(instrument_id, timeframe, self.timestamp, period)
            .await
    }
    async fn ema(&self, instrument_id: &InstrumentId, timeframe: Timeframe, period: u64) -> f64 {
        self.indicators
            .exponential_moving_average(instrument_id, timeframe, self.timestamp, period)
            .await
    }
    async fn bb(&self, instrument_id: &InstrumentId, timeframe: Timeframe, period: u64, multiplier: f64) -> BollingerBand {
        self.indicators.bollinger_bands(instrument_id, timeframe, self.timestamp, period, multiplier).await
    }
    async fn psar(&self, instrument_id: &InstrumentId, timeframe: Timeframe) -> Option<Side> {
        self.indicators.parabolic_sar(instrument_id, timeframe, self.timestamp).await
    }
}
