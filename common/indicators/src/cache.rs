use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use moka::future::Cache;

use domain_model::{InstrumentId, Side, Timeframe};

use crate::api::{BollingerBand, IndicatorsApi};

pub struct IndicatorCache<I: IndicatorsApi> {
    indicators: Arc<I>,
    sma_cache: Cache<String, f64>,
    ema_cache: Cache<String, f64>,
    bb_cache: Cache<String, BollingerBand>,
    psar_cache: Cache<String, Option<Side>>,
}

impl<I: IndicatorsApi> IndicatorCache<I> {
    pub fn new(indicators: Arc<I>) -> Self {
        Self {
            indicators,
            sma_cache: Cache::builder()
                .max_capacity(u64::MAX)
                .time_to_idle(Duration::from_secs(21600))
                .build(),
            ema_cache: Cache::builder()
                .max_capacity(u64::MAX)
                .time_to_idle(Duration::from_secs(21600))
                .build(),
            bb_cache: Cache::builder()
                .max_capacity(u64::MAX)
                .time_to_idle(Duration::from_secs(21600))
                .build(),
            psar_cache: Cache::builder()
                .max_capacity(u64::MAX)
                .time_to_idle(Duration::from_secs(21600))
                .build(),
        }
    }
}

#[async_trait]
impl<I: IndicatorsApi + Sync + Send> IndicatorsApi for IndicatorCache<I> {
    async fn simple_moving_average(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64) -> f64 {
        let key = format!("{instrument_id:?}-{timeframe:?}-{timestamp:?}-{period:?}");
        if let Some(value) = self.sma_cache.get(&key).await {
            value
        } else {
            let value = self.indicators.simple_moving_average(instrument_id, timeframe, timestamp, period).await;
            self.sma_cache.insert(key, value).await;
            value
        }
    }

    async fn exponential_moving_average(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64) -> f64 {
        let key = format!("{instrument_id:?}-{timeframe:?}-{timestamp:?}-{period:?}");
        if let Some(value) = self.ema_cache.get(&key).await {
            value
        } else {
            let value = self.indicators.exponential_moving_average(instrument_id, timeframe, timestamp, period).await;
            self.ema_cache.insert(key, value).await;
            value
        }
    }

    async fn bollinger_bands(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>, period: u64, multiplier: f64) -> BollingerBand {
        let key = format!("{instrument_id:?}-{timeframe:?}-{timestamp:?}-{period:?}-{multiplier:?}");
        if let Some(value) = self.bb_cache.get(&key).await {
            value
        } else {
            let value = self.indicators.bollinger_bands(instrument_id, timeframe, timestamp, period, multiplier).await;
            self.bb_cache.insert(key, value.clone()).await;
            value
        }
    }

    async fn parabolic_sar(&self, instrument_id: &InstrumentId, timeframe: Timeframe, timestamp: DateTime<Utc>) -> Option<Side> {
        let key = format!("{instrument_id:?}-{timeframe:?}-{timestamp:?}");
        if let Some(value) = self.psar_cache.get(&key).await {
            value
        } else {
            let value = self.indicators.parabolic_sar(instrument_id, timeframe, timestamp).await;
            self.psar_cache.insert(key, value).await;
            value
        }
    }
}
