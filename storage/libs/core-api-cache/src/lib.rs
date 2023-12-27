use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use uuid::Uuid;

use domain_model::{Candle, Currency, Exchange, InstrumentId, LP, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe};
use domain_model::drawing::{Line, Point};
use storage_core_api::{StorageApi, SyncReport};

pub struct StorageCoreApiCache<S: StorageApi> {
    client: Arc<S>,
    candles_cache: Cache<String, Vec<Candle>>,
}

impl<S: StorageApi> StorageCoreApiCache<S> {
    pub async fn new(client: Arc<S>) -> Self {
        Self {
            client,
            candles_cache: Cache::builder()
                .max_capacity(u64::MAX)
                .time_to_idle(Duration::from_secs(21600))
                .build(),
        }
    }
}

#[async_trait]
impl<S: StorageApi> StorageApi for StorageCoreApiCache<S> {
    async fn save_order(&self, order: Order) -> Result<()> {
        self.client.save_order(order).await
    }

    async fn save_lp(&self, lp: LP) -> Result<()> {
        self.client.save_lp(lp).await
    }

    async fn get_orders(&self, id: Option<String>,
                        simulation_id: Option<Uuid>,
                        exchange: Option<Exchange>,
                        market_type: Option<MarketType>,
                        target: Option<Currency>,
                        source: Option<Currency>,
                        status: Option<OrderStatus>,
                        side: Option<Side>,
                        order_type: Option<OrderType>) -> Result<Vec<Order>> {
        self.client.get_orders(id, simulation_id, exchange, market_type, target, source, status, side, order_type).await
    }

    async fn save_position(&self, position: Position) -> Result<()> {
        self.client.save_position(position).await
    }

    async fn get_positions(&self, exchange: Option<Exchange>, currency: Option<Currency>, side: Option<Side>) -> Result<Vec<Position>> {
        self.client.get_positions(exchange, currency, side).await
    }

    async fn save_candle(&self, candle: Candle) -> Result<()> {
        self.client.save_candle(candle).await
    }

    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Option<Timeframe>, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u64>) -> Result<Vec<Candle>> {
        let key = format!("{instrument_id:?}-{timeframe:?}-{from:?}-{to:?}-{limit:?}");
        if let Some(value) = self.candles_cache.get(&key).await {
            Ok(value)
        } else {
            let result = self.client.get_candles(instrument_id, timeframe, from, to, limit).await;
            if let Ok(candles) = &result {
                self.candles_cache.insert(key, candles.clone()).await;
            }
            result
        }
    }

    async fn sync(&self, instrument_id: &InstrumentId, timeframes: &[Timeframe], from: DateTime<Utc>, to: Option<DateTime<Utc>>) -> Result<Vec<SyncReport>> {
        self.client.sync(instrument_id, timeframes, from, to).await
    }

    async fn save_point(&self, point: Point) -> Result<()> {
        self.client.save_point(point).await
    }

    async fn get_points(&self, deployment_id: Uuid, instrument_id: &InstrumentId) -> Result<Vec<Point>> {
        self.client.get_points(deployment_id, instrument_id).await
    }

    async fn save_line(&self, line: Line) -> Result<()> {
        self.client.save_line(line).await
    }

    async fn get_lines(&self, deployment_id: Uuid, instrument_id: &InstrumentId) -> Result<Vec<Line>> {
        self.client.get_lines(deployment_id, instrument_id).await
    }
}
