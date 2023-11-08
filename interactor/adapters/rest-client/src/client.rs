use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde_urlencoded::to_string;
use tracing::trace;

use domain_model::{Action, Candle, Exchange, InstrumentId, Order, Subscription, Subscriptions, Timeframe};
use interactor_core_api::InteractorApi;
use interactor_rest_api::endpoints::{DELETE_UNSUBSCRIBE, GET_CANDLES, GET_ORDER, GET_PRICE, GET_SUBSCRIPTIONS, POST_EXECUTE_ACTIONS, POST_SUBSCRIBE};
use interactor_rest_api::path_queries::{CandlesQuery, OrderQuery, PriceQuery};

pub struct InteractorRestClient {
    url: String,
    client: Client,
}

impl InteractorRestClient {
    pub fn new(url: &str) -> Self {
        let mut url = String::from(url);
        if !url.starts_with("http") {
            url = format!("http://{url}");
        }
        Self {
            url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl InteractorApi for InteractorRestClient {
    async fn subscriptions(&self) -> Result<Vec<Subscriptions>> {
        let endpoint = format!("{}{}", self.url, GET_SUBSCRIPTIONS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    async fn subscribe(&self, subscription: Subscription) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_SUBSCRIBE);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url)
            .json(&subscription)
            .send()
            .await?;
        Ok(())
    }

    async fn unsubscribe(&self, subscription: Subscription) -> Result<()> {
        let endpoint = format!("{}{}", self.url, DELETE_UNSUBSCRIBE);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.delete(url)
            .json(&subscription)
            .send()
            .await?;
        Ok(())
    }

    async fn execute_actions(&self, actions: Vec<Action>) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_EXECUTE_ACTIONS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url)
            .json(&actions)
            .send()
            .await?;
        Ok(())
    }

    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Timeframe, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u8>) -> Result<Vec<Candle>> {
        let query = CandlesQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timeframe,
            from: from.map(|timestamp| timestamp.timestamp_millis()),
            to: to.map(|timestamp| timestamp.timestamp_millis()),
            limit: limit.unwrap_or(100),
        };

        let endpoint = format!("{}{}", self.url, GET_CANDLES);
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

    async fn get_price(&self, instrument_id: &InstrumentId, timestamp: Option<DateTime<Utc>>) -> Result<f64> {
        let query = PriceQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timestamp: timestamp.map(|timestamp| timestamp.timestamp_millis()),
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

    async fn get_order(&self, exchange: Exchange, order_id: &str) -> Result<Option<Order>> {
        let query = OrderQuery {
            order_id: order_id.to_string(),
            exchange,
        };

        let endpoint = format!("{}{}", self.url, GET_ORDER);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}
