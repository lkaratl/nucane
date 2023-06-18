use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_urlencoded::to_string as to_ustring;
use tracing::trace;

use domain_model::{Candle, Currency, Exchange, InstrumentId, MarketType, Timeframe};

pub struct InteractorClient {
    url: String,
    client: Client,
}

impl InteractorClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_candles_history(&self,
                                     instrument_id: &InstrumentId,
                                     timeframe: Option<Timeframe>,
                                     from_timestamp: Option<DateTime<Utc>>,
                                     to_timestamp: Option<DateTime<Utc>>,
                                     limit: Option<u8>) -> Result<Vec<Candle>, Error> {
        let query = CandlesHistoryQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timeframe,
            from_timestamp: from_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            to_timestamp: to_timestamp.map(|timestamp| timestamp.timestamp_millis()),
            limit,
        };

        let endpoint = format!("{}{}", self.url, "/api/v1/interactor/candles/history");
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_ustring(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}

#[derive(Serialize)]
struct CandlesHistoryQuery {
    exchange: Exchange,
    market_type: MarketType,
    target: Currency,
    source: Currency,
    timeframe: Option<Timeframe>,
    from_timestamp: Option<i64>,
    to_timestamp: Option<i64>,
    limit: Option<u8>,
}
