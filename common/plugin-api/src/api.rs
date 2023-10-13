use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::time::error::Elapsed;
use tracing::Level;
use tracing::{error, span};

use domain_model::drawing::{Color, Coord, Icon, LineStyle};
use domain_model::{
    Action, Currency, CurrencyPair, Exchange, InstrumentId, Order, OrderType, PluginId, Position,
    Side, Size, Tick, Timeframe, Trigger,
};

#[async_trait]
pub trait PluginApi: Send + Sync {
    fn id(&self) -> PluginId;
    fn configure(&mut self, _config: &HashMap<String, String>) {}
    fn instruments(&self) -> Vec<InstrumentId>;
    fn indicators(&self) -> Vec<()> {
        Vec::new()
    }
    // todo create common indicators enum
    fn on_tick_sync(&mut self, tick: &Tick, api: Arc<dyn PluginInternalApi>) -> Vec<Action> {
        let tick_id = format!(
            "{} '{}' {}-{}='{}'",
            tick.instrument_id.exchange,
            tick.timestamp,
            tick.instrument_id.pair.target,
            tick.instrument_id.pair.source,
            tick.price
        );
        let _span = span!(
            Level::INFO,
            "strategy",
            name = self.id().name,
            version = self.id().version,
            tick_id
        )
        .entered();
        let runtime = with_tokio_runtime(self.on_tick(tick, api));
        match runtime {
            Ok(actions) => actions,
            Err(error) => {
                error!(
                    "Timeout during tick processing, strategy: '{}:{}'. Error: '{error}'",
                    self.id().name,
                    self.id().version
                );
                Vec::new()
            }
        }
    }

    async fn on_tick(&mut self, tick: &Tick, api: Arc<dyn PluginInternalApi>) -> Vec<Action>;
}

#[tokio::main]
async fn with_tokio_runtime<T: Default>(future: impl Future<Output = T>) -> Result<T, Elapsed> {
    tokio::time::timeout(Duration::from_secs(10), future).await
}

pub trait PluginInternalApi: Send + Sync {
    fn state(&self) -> Arc<dyn StateInternalApi>;
    fn actions(&self) -> Arc<dyn ActionsInternalApi>;
    fn orders(&self) -> Arc<dyn OrdersInternalApi>;
    fn positions(&self) -> Arc<dyn PositionsInternalApi>;
    fn indicators(&self) -> Arc<dyn IndicatorsInternalApi>;
    fn drawings(&self) -> Arc<dyn DrawingsInternalApi>;
}

#[async_trait]
pub trait StateInternalApi: Send + Sync {
    async fn set(&self, key: &str, state: Value);
    async fn get(&self, key: &str) -> Option<Value>;
}

#[async_trait]
pub trait ActionsInternalApi: Send + Sync {
    fn create_order_action(
        &self,
        pair: CurrencyPair,
        order_type: OrderType,
        size: Size,
        side: Side,
        sl: Option<Trigger>,
        tp: Option<Trigger>,
    ) -> Action;
}

#[async_trait]
pub trait OrdersInternalApi: Send + Sync {
    async fn get_order_by_id(&self, id: &str) -> Option<Order>;
}

#[async_trait]
pub trait PositionsInternalApi: Send + Sync {
    async fn get_position(&self, exchange: Exchange, currency: Currency) -> Option<Position>;
}

#[async_trait]
pub trait IndicatorsInternalApi: Send + Sync {
    async fn moving_avg(
        &self,
        instrument_id: &InstrumentId,
        timeframe: Timeframe,
        length: u16,
    ) -> f64;
}

#[async_trait]
pub trait DrawingsInternalApi: Send + Sync {
    async fn save_point(&self, point: Point);
    async fn save_line(&self, line: Line);
}

#[derive(Debug)]
pub struct Point {
    pub instrument_id: InstrumentId,
    pub label: String,
    pub icon: Option<Icon>,
    pub color: Option<Color>,
    pub text: Option<String>,
    pub coord: Coord,
}

impl Point {
    pub fn new(
        instrument_id: InstrumentId,
        label: &str,
        icon: Option<Icon>,
        color: Option<Color>,
        text: Option<String>,
        coord: Coord,
    ) -> Self {
        Self {
            instrument_id,
            label: label.to_string(),
            icon,
            color,
            text,
            coord,
        }
    }
}

#[derive(Debug)]
pub struct Line {
    pub instrument_id: InstrumentId,
    pub label: String,
    pub style: Option<LineStyle>,
    pub color: Option<Color>,
    pub start: Coord,
    pub end: Coord,
}

impl Line {
    pub fn new(
        instrument_id: InstrumentId,
        label: &str,
        style: Option<LineStyle>,
        color: Option<Color>,
        start: Coord,
        end: Coord,
    ) -> Self {
        Self {
            instrument_id,
            label: label.to_string(),
            style,
            color,
            start,
            end,
        }
    }
}
