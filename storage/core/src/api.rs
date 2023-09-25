use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_model::{Candle, Currency, Exchange, InstrumentId, MarketType, Order, OrderStatus, OrderType, Position, Side, Timeframe};
use storage_core_api::{StorageApi, SyncReport};
use anyhow::Result;
use interactor_core_api::InteractorApi;
use storage_persistence_api::{CandleRepository, OrderRepository, PositionRepository};
use crate::services::candle::CandleService;
use crate::services::candle_sync::CandleSyncService;
use crate::services::order::OrderService;
use crate::services::position::PositionService;

pub struct Storage<I: InteractorApi, O: OrderRepository, P: PositionRepository, C: CandleRepository> {
    order_service: OrderService<O>,
    position_service: PositionService<P>,
    candle_service: Arc<CandleService<C>>,
    candle_sync_service: CandleSyncService<I, C>,
}

impl<I: InteractorApi, O: OrderRepository, P: PositionRepository, C: CandleRepository> Storage<I, O, P, C> {
    pub fn new(interactor_client: I, order_repository: O, position_repository: P, candle_repository: C) -> Self {
        let interactor_client = Arc::new(interactor_client);
        let order_service = OrderService::new(order_repository);
        let position_service = PositionService::new(position_repository);
        let candle_service = Arc::new(CandleService::new(candle_repository));
        let candle_sync_service = CandleSyncService::new(Arc::clone(&candle_service), interactor_client);
        Self {
            order_service,
            position_service,
            candle_service,
            candle_sync_service,
        }
    }
}

#[async_trait]
impl <I: InteractorApi, O: OrderRepository, P: PositionRepository, C: CandleRepository> StorageApi for Storage<I, O, P, C> {
    async fn save_order(&self, order: Order) -> Result<()> {
        self.order_service.save(order).await;
        Ok(())
    }

    async fn get_orders(&self, id: Option<String>,
                       exchange: Option<Exchange>,
                       market_type: Option<MarketType>,
                       target: Option<Currency>,
                       source: Option<Currency>,
                       status: Option<OrderStatus>,
                       side: Option<Side>,
                       order_type: Option<OrderType>) -> Result<Vec<Order>> {
        let orders = self.order_service.get(id, exchange, market_type, target, source, status, side, order_type).await;
        Ok(orders)
    }

    async fn save_position(&self, position: Position) -> Result<()> {
        self.position_service.save(position).await;
        Ok(())
    }

    async fn get_positions(&self, exchange: Option<Exchange>, currency: Option<Currency>, side: Option<Side>) -> Result<Vec<Position>> {
        let positions = self.position_service.get(exchange, currency, side).await;
        Ok(positions)
    }

    async fn save_candle(&self, candle: Candle) -> Result<()> {
        self.candle_service.save(candle).await;
        Ok(())
    }

    async fn get_candles(&self, instrument_id: &InstrumentId, timeframe: Option<Timeframe>, from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>, limit: Option<u64>) -> Result<Vec<Candle>> {
        let candles = self.candle_service.get(instrument_id, timeframe, from, to, limit).await;
        Ok(candles)
    }

    async fn sync(&self, instrument_id: &InstrumentId, timeframes: &[Timeframe], from: DateTime<Utc>, to: Option<DateTime<Utc>>) -> Result<Vec<SyncReport>> {
        self.candle_sync_service.sync(instrument_id, timeframes, from, to).await
    }
}
