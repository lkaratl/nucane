use anyhow::Error;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde_urlencoded::to_string;
use tracing::{trace};

use domain_model::{Candle, InstrumentId, Timeframe};
use interactor_rest_api::endpoints::{GET_CANDLES_HISTORY, GET_PRICE};
use interactor_rest_api::path_query::{CandlesQuery, PriceQuery};

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
        let query = CandlesQuery {
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

    pub async fn get_price(&self,
                           instrument_id: &InstrumentId,
                           timestamp: Option<DateTime<Utc>>) -> Result<f64, Error> {
        let query = PriceQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timestamp: timestamp.map(|timestamp| timestamp.timestamp_millis())
        };

        let endpoint = format!("{}{}", self.url, GET_PRICE);
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use tracing::debug;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::SubscriberBuilder;
    use domain_model::{Currency, CurrencyPair, Exchange, InstrumentId, MarketType, Timeframe};
    use crate::InteractorClient;

    pub fn init_logger(directives: &str) {
        let subscriber = SubscriberBuilder::default()
            .with_env_filter(EnvFilter::new(directives))
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");
    }

    #[tokio::test]
    async fn test_get_candles_history() {
        init_logger("DEBUG");
        let instrument_id = InstrumentId {
            exchange: Exchange::OKX,
            market_type: MarketType::Spot,
            pair: CurrencyPair {
                target: Currency::FTM,
                source: Currency::USDT,
            },
        };

        let response = InteractorClient::new("http://localhost:8083")
            .get_candles_history(
                &instrument_id,
                Timeframe::OneH,
                Some(Utc.timestamp_millis_opt(1682899200000).unwrap()),
                Some(Utc.timestamp_millis_opt(1683259200000).unwrap()),
                Some(100),
            ).await
            .unwrap();
        debug!("{response:?}");
    }
}
