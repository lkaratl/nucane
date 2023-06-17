use std::sync::Arc;
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use tracing::{debug, info, trace, warn};

use domain_model::{Candle, CandleStatus, InstrumentId, Timeframe};
use interactor_rest_client::InteractorClient;

use crate::candle::CandleService;

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

    #[tokio::main]
    pub async fn sync(&self, instrument_id: &InstrumentId, duration: Option<Duration>) {
        // todo uncomment for indicators calculation
        // self.sync_timeframe(instrument_id, Timeframe::OneD, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::FourH, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::TwoH, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::OneH, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::ThirtyM, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::FifteenM, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::FiveM, duration).await;
        // self.sync_timeframe(instrument_id, Timeframe::OneM, duration).await;
        self.sync_timeframe(instrument_id, Timeframe::OneS, duration).await;
    }

    // todo refactor this !!!
    async fn sync_timeframe(&self, instrument_id: &InstrumentId, timeframe: Timeframe, duration: Option<Duration>) {
        info!("Start sync for timeframe: {timeframe}");
        let to_timestamp = Utc::now();
        let mut from_timestamp = to_timestamp - duration.unwrap_or(Duration::days(7));
        let time_step = match timeframe {
            Timeframe::OneS => Duration::seconds(1),
            Timeframe::OneM => Duration::minutes(1),
            Timeframe::FiveM => Duration::minutes(5),
            Timeframe::FifteenM => Duration::minutes(15),
            Timeframe::ThirtyM => Duration::minutes(30),
            Timeframe::OneH => Duration::hours(1),
            Timeframe::TwoH => Duration::hours(2),
            Timeframe::FourH => Duration::hours(4),
            Timeframe::OneD => Duration::days(1),
        };
        let mut candles = self.candle_service.get(
            instrument_id,
            Some(timeframe),
            Some(from_timestamp),
            Some(to_timestamp),
            None).into_iter().rev();
        let mut candle = candles.next();

        let mut actual_candles = Vec::new().into_iter();
        let mut candles_for_save: Vec<Candle> = Vec::new();
        trace!("Start sync for timeframe: {timeframe}, from timestamp: {from_timestamp}, to timestamp: {to_timestamp}");
        while from_timestamp <= to_timestamp {
            trace!("Check candle before timestamp step: {}", from_timestamp + time_step);
            if candles_for_save.len() > 5000 {
                debug!("Save batch of candles for time range: {} - {}",
                    candles_for_save.first().unwrap().timestamp, candles_for_save.last().unwrap().timestamp );
                self.candle_service.save_many(candles_for_save);
                candles_for_save = Vec::new();
            }
            match &candle {
                None => {
                    let mut actual_candle = actual_candles.next();
                    trace!("Actual candle: {actual_candle:?}");
                    if actual_candle.is_none() {
                        let batch_end_timestamp = from_timestamp + time_step * 100;
                        actual_candles = self.interactor_client.get_candles_history(instrument_id,
                                                                                    timeframe,
                                                                                    Some(from_timestamp),
                                                                                    Some(batch_end_timestamp),
                                                                                    Some(100))
                            .await
                            .unwrap()
                            .into_iter();
                        actual_candle = actual_candles.next();
                        trace!("Actual candle: {actual_candle:?}");
                    }
                    match actual_candle {
                        None => warn!("WARN can't add candle for timestamp: {from_timestamp}"),
                        Some(c) => {
                            if c.timestamp < from_timestamp + time_step {
                                trace!("- Save new candle: {c:?}");
                                candles_for_save.push(c);
                            } else {
                                warn!("WARN can't get actual candle before timestamp: {}", from_timestamp + time_step)
                            }
                        }
                    }
                }
                Some(c) => {
                    if c.timestamp > from_timestamp + time_step {
                        let mut actual_candle = actual_candles.next();
                        trace!("Actual candle: {actual_candle:?}");
                        if actual_candle.is_none() {
                            let batch_end_timestamp = from_timestamp + time_step * 100;
                            actual_candles = self.interactor_client.get_candles_history(instrument_id,
                                                                                        timeframe,
                                                                                        Some(from_timestamp),
                                                                                        Some(batch_end_timestamp),
                                                                                        Some(100))
                                .await
                                .unwrap()
                                .into_iter();
                            actual_candle = actual_candles.next();
                            trace!("Actual candle: {actual_candle:?}");
                        }
                        match actual_candle {
                            None => warn!("WARN can't add candle for timestamp: {from_timestamp}"),
                            Some(c) => {
                                if c.timestamp < from_timestamp + time_step {
                                    trace!("- Save new candle: {c:?}");
                                    candles_for_save.push(c);
                                } else {
                                    warn!("WARN can't get actual candle before timestamp: {}", from_timestamp + time_step)
                                }
                            }
                        }
                    } else if c.status == CandleStatus::Open {
                        trace!("Candle already exists, but have not actual state, candle: {c:?}");
                        let mut actual_candle = actual_candles.next();
                        trace!("Actual candle: {actual_candle:?}");
                        if actual_candle.is_none() {
                            let batch_end_timestamp = from_timestamp + time_step * 100;
                            actual_candles = self.interactor_client.get_candles_history(instrument_id,
                                                                                        timeframe,
                                                                                        Some(from_timestamp),
                                                                                        Some(batch_end_timestamp),
                                                                                        Some(100))
                                .await
                                .unwrap()
                                .into_iter();
                            actual_candle = actual_candles.next();
                            trace!("Actual candle: {actual_candle:?}");
                        }
                        match actual_candle {
                            None => warn!("WARN can't add candle for timestamp: {from_timestamp}"),
                            Some(c) => {
                                if c.timestamp < from_timestamp + time_step {
                                    trace!("- Save new candle: {c:?}");
                                    candles_for_save.push(c);
                                } else {
                                    warn!("WARN can't get actual candle before timestamp: {}", from_timestamp + time_step)
                                }
                            }
                        }
                        candle = candles.next();
                        trace!("Candle: {candle:?}");
                    } else {
                        trace!("Candle already exists, candle: {c:?}");
                        candle = candles.next();
                        trace!("Candle: {candle:?}");
                        actual_candles.next();
                    }
                }
            }
            from_timestamp += time_step;
        }
        info!("Finish sync for timeframe: {timeframe}");
    }
}
