use domain_model::{Action, Candle, InstrumentId, Subscription, Timeframe};
use anyhow::Result;
use chrono::{DateTime, Utc};

pub trait InteractorApi: Clone + Send + Sync + 'static {
    fn subscribe(&self, subscription: Subscription) -> Result<()>;
    fn unsubscribe(&self, subscription: Subscription) -> Result<()>;
    fn execute_actions(&self, actions: Vec<Action>) -> Result<()>;
    fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u8>) -> Result<Vec<Candle>>;
    fn get_price(&self, instrument_id: &InstrumentId, timestamp: DateTime<Utc>) -> Result<f64>; // todo maybe same bidirectional abstraction + timestamp maybe optional
}
