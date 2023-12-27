use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Duration, DurationRound, Utc};
use tracing::{debug, error, info, trace, warn};

use domain_model::{Candle, InstrumentId, Timeframe};
use interactor_core_api::InteractorApi;
use storage_core_api::SyncReport;
use storage_persistence_api::CandleRepository;

use crate::services::candle::CandleService;

pub struct CandleSyncService<I: InteractorApi, R: CandleRepository> {
    candle_service: Arc<CandleService<R>>,
    interactor_client: Arc<I>,
}

impl<I: InteractorApi, R: CandleRepository> CandleSyncService<I, R> {
    pub fn new(candle_service: Arc<CandleService<R>>,
               interactor_client: Arc<I>) -> Self {
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
        info!("Finish candles sync: {reports:?}");
        Ok(reports)
    }

    async fn sync_timeframe(&self,
                            instrument_id: &InstrumentId,
                            timeframe: &Timeframe,
                            from: DateTime<Utc>,
                            to: DateTime<Utc>) -> Result<SyncReport> {
        info!("Start sync for timeframe: {timeframe}");
        let duration: Duration = (*timeframe).into();
        let from = from.duration_round(duration)?;
        let to = to.duration_round(duration)?;
        let mut handles = Vec::new();
        let batch_step = Duration::days(5);
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
                    *timeframe,
                    batch_start,
                    batch_end)
            });
            handles.push(handle);

            batch_start = batch_end
        }
        let mut reports: Vec<_> = Vec::new();
        for report in futures::future::join_all(handles).await {
            let report = report.map_err(|err| {
                error!("Error sync batch for timeframe: {timeframe}, from: {from}, to: {to}, error: {err:?}");
                err
            })?.map_err(|err| {
                error!("Error sync batch for timeframe: {timeframe}, from: {from}, to: {to}, error: {err:?}");
                err
            })?;
            reports.push(report);
        }

        let mut result = SyncReport {
            timeframe: *timeframe,
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

async fn sync_batch<I: InteractorApi, R: CandleRepository>(candle_service: Arc<CandleService<R>>,
                                                           interactor_client: Arc<I>,
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
            candles_for_save.dedup_by_key(|candle| candle.id.clone());
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
                        trace!("Add new candle: {}", actual_candle.timestamp);
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
                    trace!("Add new candle: {}", actual_candle.timestamp);
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
        candles_for_save.dedup_by_key(|candle| candle.id.clone());
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

struct ActualCandlesLazyVec<I: InteractorApi> {
    max_size: u8,
    interactor_client: Arc<I>,
    instrument_id: InstrumentId,
    timeframe: Timeframe,
    candles: Vec<Candle>,
}

impl<I: InteractorApi> ActualCandlesLazyVec<I> {
    fn new(interactor_client: Arc<I>, instrument_id: InstrumentId, timeframe: Timeframe) -> Self {
        Self {
            max_size: 100,
            interactor_client,
            instrument_id,
            timeframe,
            candles: Vec::new(),
        }
    }

    async fn fetch(&mut self, timestamp: DateTime<Utc>) {
        trace!("Fetch candles for timestamp: {}", timestamp);
        let time_step: Duration = self.timeframe.into();
        let time_offset = time_step / 2;

        let timestamp = timestamp - time_offset;
        let to = timestamp + time_step * self.max_size as i32 - time_offset;

        self.candles = self.interactor_client.get_candles(
            &self.instrument_id,
            self.timeframe,
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
            if self.candles.is_empty() {
                None
            } else {
                Some(self.candles.remove(0))
            }
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
