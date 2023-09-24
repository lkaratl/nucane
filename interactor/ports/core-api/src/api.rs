use domain_model::{Action, Candle, InstrumentId, Subscription, Subscriptions, Timeframe};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait InteractorApi: Send + Sync + 'static {
    async fn subscriptions(&self) -> Result<Vec<Subscriptions>>;
    async fn subscribe(&self, subscription: Subscription) -> Result<()>; // todo add return type
    async fn unsubscribe(&self, subscription: Subscription) -> Result<()>; // todo add return type
    async fn execute_actions(&self, actions: Vec<Action>) -> Result<()>; // todo add return type ????
    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u8>) -> Result<Vec<Candle>>;
    async fn get_price(&self, instrument_id: &InstrumentId, timestamp: Option<DateTime<Utc>>) -> Result<f64>; // todo maybe same bidirectional abstraction
}
