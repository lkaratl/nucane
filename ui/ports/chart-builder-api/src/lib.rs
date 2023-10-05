use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait ChartBuilderApi: Send + Sync + 'static {
    // todo use builder pattern
    async fn build(&self, timestamps: Vec<DateTime<Utc>>, series: Vec<Series>) -> String;
}

pub struct Series {
    pub label: String,
    pub data: Vec<Vec<f64>>,
    // todo add type f.e candlestick, line ...
}
