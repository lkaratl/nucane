use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain_model::{Candle, Currency, Exchange, InstrumentId, LP, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe};
use domain_model::drawing::{Line, Point};
use interactor_core_api::InteractorApi;
use storage_core_api::{StorageApi, SyncReport};
use storage_persistence_api::{
    CandleRepository, DrawingRepository, OrderRepository, PositionRepository,
};

use crate::services::candle::CandleService;
use crate::services::candle_sync::CandleSyncService;
use crate::services::drawing::DrawingService;
use crate::services::order::OrderService;
use crate::services::position::PositionService;

pub struct Storage<
    I: InteractorApi,
    O: OrderRepository,
    P: PositionRepository,
    C: CandleRepository,
    D: DrawingRepository,
> {
    order_service: OrderService<O>,
    position_service: PositionService<P>,
    candle_service: Arc<CandleService<C>>,
    candle_sync_service: CandleSyncService<I, C>,
    drawing_service: DrawingService<D>,
}

impl<
    I: InteractorApi,
    O: OrderRepository,
    P: PositionRepository,
    C: CandleRepository,
    D: DrawingRepository,
> Storage<I, O, P, C, D>
{
    pub fn new(
        interactor_client: I,
        order_repository: O,
        position_repository: P,
        candle_repository: C,
        drawing_repository: D,
    ) -> Self {
        let interactor_client = Arc::new(interactor_client);
        let order_service = OrderService::new(order_repository);
        let position_service = PositionService::new(position_repository);
        let candle_service = Arc::new(CandleService::new(candle_repository));
        let candle_sync_service =
            CandleSyncService::new(Arc::clone(&candle_service), interactor_client);
        let drawing_service = DrawingService::new(drawing_repository);
        Self {
            order_service,
            position_service,
            candle_service,
            candle_sync_service,
            drawing_service,
        }
    }
}

#[async_trait]
impl<I: InteractorApi, O: OrderRepository, P: PositionRepository, C: CandleRepository, D: DrawingRepository> StorageApi for Storage<I, O, P, C, D>
{
    async fn save_order(&self, order: Order) -> Result<()> {
        self.order_service.save(order).await;
        Ok(())
    }

    async fn save_lp(&self, lp: LP) -> Result<()> {
        self.order_service.save_lp(lp).await;
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
        let orders = self
            .order_service
            .get(
                id,
                simulation_id,
                exchange,
                market_type,
                target,
                source,
                status,
                side,
                order_type,
            )
            .await;
        Ok(orders)
    }

    async fn save_position(&self, position: Position) -> Result<()> {
        self.position_service.save(position).await;
        Ok(())
    }

    async fn get_positions(
        &self,
        exchange: Option<Exchange>,
        currency: Option<Currency>,
        side: Option<Side>,
    ) -> Result<Vec<Position>> {
        let positions = self.position_service.get(exchange, currency, side).await;
        Ok(positions)
    }

    async fn save_candle(&self, candle: Candle) -> Result<()> {
        self.candle_service.save(candle).await;
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
        let candles = self
            .candle_service
            .get(instrument_id, timeframe, from, to, limit)
            .await;
        Ok(candles)
    }

    async fn sync(
        &self,
        instrument_id: &InstrumentId,
        timeframes: &[Timeframe],
        from: DateTime<Utc>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<SyncReport>> {
        self.candle_sync_service
            .sync(instrument_id, timeframes, from, to)
            .await
    }

    async fn save_point(&self, point: Point) -> Result<()> {
        self.drawing_service.save_point(point).await;
        Ok(())
    }

    async fn get_points(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Point>> {
        let points = self
            .drawing_service
            .get_points(deployment_id, instrument_id)
            .await;
        Ok(points)
    }

    async fn save_line(&self, line: Line) -> Result<()> {
        self.drawing_service.save_line(line).await;
        Ok(())
    }

    async fn get_lines(
        &self,
        deployment_id: Uuid,
        instrument_id: &InstrumentId,
    ) -> Result<Vec<Line>> {
        let lines = self
            .drawing_service
            .get_lines(deployment_id, instrument_id)
            .await;
        Ok(lines)
    }
}
