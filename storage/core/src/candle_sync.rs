use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use sea_orm::DatabaseConnection;
use tracing::{debug, error, info, trace, warn};

use domain_model::{Candle, InstrumentId, Timeframe};
use interactor_rest_client::InteractorClient;

use crate::candle::CandleService;
use anyhow::Result;

pub struct CandleSyncService {
    candle_service: Arc<CandleService<DatabaseConnection>>,
    interactor_client: Arc<InteractorClient>,
}

impl CandleSyncService {
    pub fn new(candle_service: Arc<CandleService<DatabaseConnection>>,
               interactor_client: Arc<InteractorClient>) -> Self {
        Self {
            candle_service,
            interactor_client,
        }
    }

    pub async fn sync(&self,
                      instrument_id: &InstrumentId,
                      timeframes: &[Timeframe],
                      from: DateTime<Utc>,
                      to: Option<DateTime<Utc>>) -> Result<Vec<SyncReport>> {
        let to = to.unwrap_or(Utc::now());
        info!("Sync for {} '{}-{}-{}' from: '{from}', to '{to}'",
            instrument_id.exchange,
            instrument_id.pair.target,
            instrument_id.pair.source,
            instrument_id.market_type);

        let mut reports = Vec::new();
        for timeframe in timeframes {
            let report = self.sync_timeframe(instrument_id, timeframe, from, to).await?;
            reports.push(report);
        }
        Ok(reports)
    }

    async fn sync_timeframe(&self,
                            instrument_id: &InstrumentId,
                            timeframe: &Timeframe,
                            from: DateTime<Utc>,
                            to: DateTime<Utc>) -> Result<SyncReport> {
        info!("Start sync for timeframe: {timeframe}");
        let mut handles = Vec::new();
        let batch_step = Duration::days(7);
        let mut batch_start = from;
        while batch_start < to {
            let mut batch_end = batch_start + batch_step;
            if batch_end > to { batch_end = to }

            let handle = tokio::spawn({
                let candle_service = Arc::clone(&self.candle_service);
                let interactor_client = Arc::clone(&self.interactor_client);
                sync_batch(
                    candle_service,
                    interactor_client,
                    instrument_id.clone(),
                    timeframe.clone(),
                    batch_start,
                    batch_end)
            });
            handles.push(handle);

            batch_start = batch_end
        }
        let reports: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap().unwrap())
            .collect();

        let mut result = SyncReport {
            timeframe: timeframe.clone(),
            total: 0,
            exists: 0,
            synced: 0,
        };
        for report in &reports {
            result.total += report.total;
            result.exists += report.exists;
            result.synced += report.synced;
        }
        Ok(result)
    }
}

async fn sync_batch(candle_service: Arc<CandleService<DatabaseConnection>>,
                    interactor_client: Arc<InteractorClient>,
                    instrument_id: InstrumentId,
                    timeframe: Timeframe,
                    from: DateTime<Utc>,
                    to: DateTime<Utc>) -> Result<SyncReport> {
    info!("Sync batch from: '{from}', to '{to}'");
    let saved_candles = candle_service.get(
        &instrument_id,
        Some(timeframe),
        Some(from),
        Some(to - Duration::milliseconds(500)),
        None).await;
    let saved_candles_count = saved_candles.len();
    let mut saved_candles = saved_candles.into_iter().rev();

    let mut actual_candles = ActualCandlesLazyVec::new(interactor_client, instrument_id, timeframe);

    let mut synced_candles_count = 0;
    let mut candles_for_save: Vec<Candle> = Vec::new();

    let time_step: Duration = timeframe.into();
    let mut timestamp = from;

    let mut saved_candle_opt = saved_candles.next();
    while timestamp < to {
        if candles_for_save.len() > 5000 {
            debug!("Save batch of candles for time range: {} - {}",
                            candles_for_save.first().unwrap().timestamp, candles_for_save.last().unwrap().timestamp );
            synced_candles_count += candles_for_save.len();
            candle_service.save_many(candles_for_save).await;
            candles_for_save = Vec::new();
        }

        match &saved_candle_opt {
            Some(saved_candle) => {
                trace!("Saved candle: {}", saved_candle.timestamp);
                if saved_candle.timestamp != timestamp {
                    warn!("Inconsistent candle: {}, for timestamp: {}", saved_candle.timestamp, timestamp);
                    if let Some(actual_candle) = actual_candles.take(timestamp).await {
                        candles_for_save.push(actual_candle);
                    } else {
                        error!("Can't load required actual candle for timestamp: {}", timestamp);
                    }
                } else {
                    saved_candle_opt = saved_candles.next();
                }
            }
            None => {
                if let Some(actual_candle) = actual_candles.take(timestamp).await {
                    candles_for_save.push(actual_candle);
                } else {
                    error!("Can't load required actual candle for timestamp: {}", timestamp);
                }
            }
        }
        timestamp += time_step;
    }
    if !candles_for_save.is_empty() {
        debug!("Save last batch candles for time range: {} - {}",
                            candles_for_save.first().unwrap().timestamp, candles_for_save.last().unwrap().timestamp );
        synced_candles_count += candles_for_save.len();
        candle_service.save_many(candles_for_save).await;
    }

    let total_candles = (to - from).num_seconds() / time_step.num_seconds();
    Ok(SyncReport {
        timeframe,
        total: total_candles as u64,
        exists: saved_candles_count as u64,
        synced: synced_candles_count as u64,
    })
}

struct ActualCandlesLazyVec {
    max_size: u8,
    interactor_client: Arc<InteractorClient>,
    instrument_id: InstrumentId,
    timeframe: Timeframe,
    candles: Vec<Candle>,
}

impl ActualCandlesLazyVec {
    fn new(interactor_client: Arc<InteractorClient>, instrument_id: InstrumentId, timeframe: Timeframe) -> Self {
        Self {
            max_size: 100,
            interactor_client,
            instrument_id,
            timeframe,
            candles: Vec::new(),
        }
    }

    async fn fetch(&mut self, timestamp: DateTime<Utc>) {
        debug!("Fetch candles for timestamp: {}", timestamp);
        let time_step: Duration = self.timeframe.into();
        let time_offset = time_step / 2;

        let timestamp = timestamp - time_offset;
        let to = timestamp + time_step * self.max_size as i32 - time_offset;

        self.candles = self.interactor_client.get_candles_history(
            &self.instrument_id,
            self.timeframe.clone(),
            Some(timestamp),
            Some(to),
            Some(self.max_size))
            .await
            .expect("Can't load candles");
        self.candles.sort_by_key(|c| c.timestamp);
    }

    async fn take(&mut self, timestamp: DateTime<Utc>) -> Option<Candle> {
        let candle = self.find_position(timestamp)
            .map(|index| self.candles.remove(index));
        if candle.is_none() {
            self.fetch(timestamp).await;
            Some(self.candles.remove(0))
        } else { candle }
    }

    fn find_position(&self, timestamp: DateTime<Utc>) -> Option<usize> {
        if self.candles.is_empty() || timestamp < self.candles.first().unwrap().timestamp || self.candles.last().unwrap().timestamp < timestamp {
            None
        } else {
            self.candles.iter().position(|c| c.timestamp == timestamp)
        }
    }
}

pub struct SyncReport {
    pub timeframe: Timeframe,
    pub total: u64,
    pub exists: u64,
    pub synced: u64,
}
