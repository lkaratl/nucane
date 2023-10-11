use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain_model::drawing::{Line, Point};
use domain_model::{
    Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position,
    Side, Timeframe,
};

#[async_trait]
pub trait OrderRepository: Send + Sync + 'static {
    async fn save(&self, order: Order) -> Result<()>;

    #[allow(clippy::too_many_arguments)]
    async fn get(
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
}

#[async_trait]
pub trait PositionRepository: Send + Sync + 'static {
    async fn save(&self, position: Position) -> Result<()>;
    async fn get(
        &self,
        exchange: Option<Exchange>,
        currency: Option<Currency>,
        side: Option<Side>,
    ) -> Result<Vec<Position>>;
}

#[async_trait]
pub trait CandleRepository: Send + Sync + 'static {
    async fn save(&self, candle: Candle) -> Result<()>;
    async fn save_many(&self, candles: Vec<Candle>) -> Result<()>;
    async fn get(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Option<Timeframe>,
        from_timestamp: Option<DateTime<Utc>>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Candle>>;
}

#[async_trait]
pub trait DrawingRepository: Send + Sync + 'static {
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
