use chrono::{DateTime, Utc};

use domain_model::{InstrumentId, Timeframe};
use storage_persistence_api::CandleRepository;

pub struct CandleService<R: CandleRepository> {
    repository: R,
}

impl<R: CandleRepository> CandleService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn save(&self, candle: domain_model::Candle) {
        self.repository
            .save(candle)
            .await
            .expect("Error during order saving");
    }

    pub async fn save_many(&self, candles: Vec<domain_model::Candle>) {
        self.repository
            .save_many(candles)
            .await
            .expect("Error during order saving");
    }

    pub async fn get(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Option<Timeframe>,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Vec<domain_model::Candle> {
        self.repository
            .get(
                instrument_id,
                timeframe,
                from_timestamp,
                to_timestamp,
                limit,
            )
            .await
            .unwrap()
    }
}
