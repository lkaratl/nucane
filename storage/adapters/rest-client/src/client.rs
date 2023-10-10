use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde_urlencoded::to_string;
use tracing::{info, trace};
use uuid::Uuid;

use domain_model::drawing::{Line, Point};
use domain_model::{
    Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position,
    Side, Timeframe,
};
use storage_core_api::{StorageApi, SyncReport};
use storage_rest_api::endpoints::{
    GET_CANDLES, GET_LINES, GET_ORDERS, GET_POINTS, GET_POSITIONS, POST_CANDLES, POST_LINE,
    POST_ORDERS, POST_POINT, POST_POSITIONS, POST_SYNC,
};
use storage_rest_api::path_queries::{
    CandleSyncQuery, CandlesQuery, DrawingQuery, OrdersQuery, PositionsQuery,
};

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

    async fn save_point(&self, point: Point) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_POINT);
        let url = Url::parse(&endpoint)?;
        info!("Request url: {url:?}");
        self.client.post(url).json(&point).send().await?;
        info!("after point save");
        Ok(())
    }

    async fn get_points(
        &self,
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Result<Vec<Point>> {
        let query = DrawingQuery {
            simulation_id,
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
        };

        let endpoint = format!("{}{}", self.url, GET_POINTS);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url).send().await?.json().await?;
        Ok(result)
    }

    async fn save_line(&self, line: Line) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_LINE);
        let url = Url::parse(&endpoint)?;
        trace!("Request url: {url:?}");
        self.client.post(url).json(&line).send().await?;
        Ok(())
    }

    async fn get_lines(
        &self,
        instrument_id: &InstrumentId,
        simulation_id: Option<Uuid>,
    ) -> Result<Vec<Line>> {
        let query = DrawingQuery {
            simulation_id,
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
        };

        let endpoint = format!("{}{}", self.url, GET_LINES);
        let mut url = Url::parse(&endpoint)?;
        url.set_query(Some(&to_string(&query)?));
        trace!("Request url: {url:?}");
        let result = self.client.get(url).send().await?.json().await?;
        Ok(result)
    }
}
