use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain_model::{Candle, Currency, Exchange, InstrumentId, LP, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe};
use domain_model::drawing::{Line, Point};

#[async_trait]
pub trait StorageApi: Send + Sync + 'static {
    async fn save_order(&self, order: Order) -> Result<()>;

    async fn save_lp(&self, lp: LP) -> Result<()>;

    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Vec<Order>>;
    async fn save_position(&self, position: Position) -> Result<()>;
    async fn get_positions(
        &self,
        exchange: Option<Exchange>,
        currency: Option<Currency>,
        side: Option<Side>,
    ) -> Result<Vec<Position>>;
    async fn save_candle(&self, candle: Candle) -> Result<()>;
    async fn get_candles(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Option<Timeframe>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Candle>>;
    async fn sync(
        &self,
        instrument_id: &InstrumentId,
        timeframes: &[Timeframe],
        from: DateTime<Utc>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SyncReport>>;
    async fn save_point(&self, point: Point) -> Result<()>;
    async fn get_points(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Point>>;
    async fn save_line(&self, line: Line) -> Result<()>;
    async fn get_lines(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Line>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncReport {
    pub timeframe: Timeframe,
    pub total: u64,
    pub exists: u64,
    pub synced: u64,
}
