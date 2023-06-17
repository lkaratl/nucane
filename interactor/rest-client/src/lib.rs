use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_urlencoded::to_string;
use tracing::trace;

use domain_model::{Candle, Currency, Exchange, InstrumentId, MarketType, Timeframe};
use interactor_rest_api::endpoints::GET_CANDLES_HISTORY;
use interactor_rest_api::path_query::CandlesHistoryQuery;

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
                                     timeframe: Timeframe,
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
            limit: limit.unwrap_or(100),
        };

        let endpoint = format!("{}{}", self.url, GET_CANDLES_HISTORY);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}
