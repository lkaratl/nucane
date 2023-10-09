use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde_urlencoded::to_string;
use tracing::trace;
use uuid::Uuid;

use domain_model::{
    Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position,
    Side, Timeframe,
};
use storage_core_api::{StorageApi, SyncReport};
use storage_rest_api::endpoints::{
    GET_CANDLES, GET_ORDERS, GET_POSITIONS, POST_CANDLES, POST_ORDERS, POST_POSITIONS, POST_SYNC,
};
use storage_rest_api::path_queries::{CandleSyncQuery, CandlesQuery, OrdersQuery, PositionsQuery};

pub struct StorageRestClient {
    url: String,
    client: Client,
}

impl StorageRestClient {
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
impl StorageApi for StorageRestClient {
    async fn save_order(&self, order: Order) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_ORDERS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url).json(&order).send().await?;
        Ok(())
    }

    async fn get_orders(
        &self,
        id: Option<String>,
        simulation_id: Option<Uuid>,
        exchange: Option<Exchange>,
        market_type: Option<MarketType>,
        target: Option<Currency>,
        source: Option<Currency>,
        status: Option<OrderStatus>,
        side: Option<Side>,
        order_type: Option<OrderType>,
    ) -> Result<Vec<Order>> {
        let query = OrdersQuery {
            id,
            simulation_id,
            exchange,
            market_type,
            target,
            source,
            status,
            side,
            order_type,
        };

        let endpoint = format!("{}{}", self.url, GET_ORDERS);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url).send().await?.json().await?;
        Ok(result)
    }

    async fn save_position(&self, position: Position) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_POSITIONS);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url).json(&position).send().await?;
        Ok(())
    }

    async fn get_positions(
        &self,
        exchange: Option<Exchange>,
        currency: Option<Currency>,
        side: Option<Side>,
    ) -> Result<Vec<Position>> {
        let query = PositionsQuery {
            exchange,
            currency,
            side,
        };

        let endpoint = format!("{}{}", self.url, GET_POSITIONS);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url).send().await?.json().await?;
        Ok(result)
    }

    async fn save_candle(&self, candle: Candle) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_CANDLES);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url).json(&candle).send().await?;
        Ok(())
    }

    async fn get_candles(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Option<Timeframe>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Candle>> {
        let query = CandlesQuery {
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
            timeframe,
            from: from.map(|timestamp| timestamp.timestamp_millis()),
            to: to.map(|timestamp| timestamp.timestamp_millis()),
            limit,
        };

        let endpoint = format!("{}{}", self.url, GET_CANDLES);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url).send().await?.json().await?;
        Ok(result)
    }

    async fn sync(
        &self,
        instrument_id: &InstrumentId,
        timeframes: &[Timeframe],
        from: DateTime<Utc>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SyncReport>> {
        let timeframes = timeframes
            .iter()
            .map(|timeframe| timeframe.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = CandleSyncQuery {
            timeframes,
            from: from.timestamp_millis(),
            to: to.map(|timestamp| timestamp.timestamp_millis()),
        };

        let endpoint = format!("{}{}", self.url, POST_SYNC);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self
            .client
            .post(url)
            .json(instrument_id)
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }
}
