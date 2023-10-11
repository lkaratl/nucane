use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde_urlencoded::to_string;
use surf::{Client, Config};
use tracing::trace;
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
            client: Config::new().try_into().unwrap(),
        }
    }
}

#[async_trait]
impl StorageApi for StorageRestClient {
    async fn save_order(&self, order: Order) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_ORDERS);
        trace!("Request: POST '{endpoint}'");
        self.client
            .post(endpoint)
            .body_json(&order)
            .unwrap()
            .await
            .unwrap();
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
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: GET '{endpoint}'");
        let result = self
            .client
            .get(endpoint)
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
        Ok(result)
    }

    async fn save_position(&self, position: Position) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_POSITIONS);
        trace!("Request: POST '{endpoint}'");
        self.client
            .post(endpoint)
            .body_json(&position)
            .unwrap()
            .await
            .unwrap();
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
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: GET '{endpoint}'");
        let result = self
            .client
            .get(endpoint)
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
        Ok(result)
    }

    async fn save_candle(&self, candle: Candle) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_CANDLES);
        trace!("Request: POST '{endpoint}'");
        self.client
            .post(endpoint)
            .body_json(&candle)
            .unwrap()
            .await
            .unwrap();
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
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: GET '{endpoint}'");
        let result = self
            .client
            .get(endpoint)
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
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
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: POST '{endpoint}'");
        let result = self
            .client
            .post(endpoint)
            .body_json(instrument_id)
            .unwrap()
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
        Ok(result)
    }

    async fn save_point(&self, point: Point) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_POINT);
        let endpoint = Url::parse(&endpoint)?;
        trace!("Request: POST '{endpoint}'");
        self.client
            .post(endpoint)
            .body_json(&point)
            .unwrap()
            .await
            .unwrap();
        Ok(())
    }

    async fn get_points(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Point>> {
        let query = DrawingQuery {
            deployment_id,
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
        };
        let endpoint = format!("{}{}", self.url, GET_POINTS);
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: GET '{endpoint}'");
        let result = self
            .client
            .get(endpoint)
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
        Ok(result)
    }

    async fn save_line(&self, line: Line) -> Result<()> {
        let endpoint = format!("{}{}", self.url, POST_LINE);
        trace!("Request: POST '{endpoint}'");
        self.client
            .post(endpoint)
            .body_json(&line)
            .unwrap()
            .await
            .unwrap();
        Ok(())
    }

    async fn get_lines(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Line>> {
        let query = DrawingQuery {
            deployment_id,
            exchange: instrument_id.exchange,
            market_type: instrument_id.market_type,
            target: instrument_id.pair.target,
            source: instrument_id.pair.source,
        };
        let endpoint = format!("{}{}", self.url, GET_LINES);
        let mut endpoint = Url::parse(&endpoint)?;
        endpoint.set_query(Some(&to_string(&query)?));
        trace!("Request: GET '{endpoint}'");
        let result = self
            .client
            .get(endpoint)
            .await
            .unwrap()
            .body_json()
            .await
            .unwrap();
        Ok(result)
    }
}
