use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Error};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use synapse::{Synapse, Topic};

#[derive(Debug, Deserialize, Serialize)]
pub struct Simulation {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub positions: Vec<SimulationPosition>,
    pub deployments: Vec<SimulationDeployment>,

    pub ticks_len: usize,
    pub actions_count: u16,
    pub active_orders: Vec<Order>,
}

impl AuditTags for Simulation {
    fn audit_tags(&self) -> Vec<String> {
        vec![
            CommonAuditTags::Simulation.to_string(),
        ]
    }
}

impl Synapse for Simulation {
    fn topic(&self) -> Topic {
        Topic::Simulation
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationDeployment {
    pub deployment_id: Option<Uuid>,
    pub timeframe: Timeframe,
    pub strategy_name: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationPosition {
    pub simulation_id: Uuid,
    pub exchange: Exchange,
    pub currency: Currency,
    pub start: f64,
    pub end: f64,
    pub diff: f64,
    pub fees: f64,
}

impl From<SimulationPosition> for Position {
    fn from(value: SimulationPosition) -> Self {
        let side = if value.end >= 0.0 { Side::Buy } else { Side::Sell };
        Position::new(Some(value.simulation_id), value.exchange, value.currency, side, value.end)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
    pub event: AuditDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AuditDetails {
    Deployment(Deployment),
    Action(Action),
    Order(Order),
    Position(Position),
    Simulation(Simulation),
}

pub trait AuditTags {
    fn audit_tags(&self) -> Vec<String>;
}

#[derive(Debug)]
pub enum CommonAuditTags {
    Deployment,
    Action,
    Order,
    Position,
    Created,
    Deleted,
    OrderAction,
    InProgress,
    Failed,
    Completed,
    Canceled,
    Simulation,
}

impl fmt::Display for CommonAuditTags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Candle {
    pub id: String,
    pub status: CandleStatus,
    pub instrument_id: InstrumentId,
    pub timestamp: DateTime<Utc>,
    pub timeframe: Timeframe,
    pub open_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub close_price: f64,
    pub target_volume: f64,
    pub source_volume: f64,
}

impl Synapse for Candle {
    fn topic(&self) -> Topic {
        Topic::Candle
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CandleStatus {
    Open,
    Close,
}

impl FromStr for CandleStatus {
    type Err = Error;
    fn from_str(input: &str) -> Result<CandleStatus, Self::Err> {
        match input {
            "Open" => Ok(CandleStatus::Open),
            "Close" => Ok(CandleStatus::Close),
            input => bail!("Unknown candle status: {input}")
        }
    }
}

impl fmt::Display for CandleStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum Timeframe {
    OneS,
    OneM,
    FiveM,
    FifteenM,
    ThirtyM,
    OneH,
    TwoH,
    FourH,
    OneD,
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Timeframe {
    type Err = Error;
    fn from_str(input: &str) -> Result<Timeframe, Self::Err> {
        match input {
            "OneS" => Ok(Timeframe::OneS),
            "OneM" => Ok(Timeframe::OneM),
            "FiveM" => Ok(Timeframe::FiveM),
            "FifteenM" => Ok(Timeframe::FifteenM),
            "ThirtyM" => Ok(Timeframe::ThirtyM),
            "OneH" => Ok(Timeframe::OneH),
            "TwoH" => Ok(Timeframe::TwoH),
            "FourH" => Ok(Timeframe::FourH),
            "OneD" => Ok(Timeframe::OneD),
            input => bail!("Unknown market type: {input}")
        }
    }
}

impl Into<Duration> for Timeframe {
    fn into(self) -> Duration {
        match self {
            Timeframe::OneS => Duration::seconds(1),
            Timeframe::OneM => Duration::minutes(1),
            Timeframe::FiveM => Duration::minutes(5),
            Timeframe::FifteenM => Duration::minutes(15),
            Timeframe::ThirtyM => Duration::minutes(30),
            Timeframe::OneH => Duration::hours(1),
            Timeframe::TwoH => Duration::hours(2),
            Timeframe::FourH => Duration::hours(4),
            Timeframe::OneD => Duration::days(1),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position {
    pub id: String,
    pub simulation_id: Option<Uuid>,
    pub exchange: Exchange,
    pub currency: Currency,
    pub side: Side,
    pub size: f64,
}

impl Position {
    pub fn new(simulation_id: Option<Uuid>,
               exchange: Exchange,
               currency: Currency,
               side: Side,
               size: f64) -> Self {
        let id = {
            let mut id = format!("{exchange}_{currency}");
            if let Some(simulation_id) = simulation_id {
                id = format!("{id}-{}", simulation_id);
            }
            id
        };
        Self {
            id,
            simulation_id,
            exchange,
            currency,
            side,
            size,
        }
    }
}

impl AuditTags for Position {
    fn audit_tags(&self) -> Vec<String> {
        vec![
            CommonAuditTags::Position.to_string(),
            self.exchange.to_string(),
            self.currency.to_string(),
        ]
    }
}

impl Synapse for Position {
    fn topic(&self) -> Topic {
        Topic::Position
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub simulation_id: Option<Uuid>,
    pub status: OrderStatus,
    pub exchange: Exchange,
    pub pair: CurrencyPair,
    pub market_type: OrderMarketType,
    pub order_type: OrderType,
    pub side: Side,
    pub size: f64,
    pub avg_price: f64,
}

impl Synapse for Order {
    fn topic(&self) -> Topic {
        Topic::Order
    }
}

impl AuditTags for Order {
    fn audit_tags(&self) -> Vec<String> {
        let mut tags = vec![CommonAuditTags::Order.to_string()];
        tags.push(self.exchange.to_string());
        tags.push(self.pair.target.to_string());
        tags.push(self.pair.source.to_string());
        match self.status {
            OrderStatus::Created => tags.push(CommonAuditTags::Created.to_string()),
            OrderStatus::InProgress => tags.push(CommonAuditTags::InProgress.to_string()),
            OrderStatus::Failed(_) => tags.push(CommonAuditTags::Failed.to_string()),
            OrderStatus::Completed => tags.push(CommonAuditTags::Completed.to_string()),
            OrderStatus::Canceled => tags.push(CommonAuditTags::Canceled.to_string()),
        }
        tags
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginEvent {
    pub id: Uuid,
    pub event: PluginEventType,
    pub strategy_name: String,
    pub strategy_version: String,
}

impl Synapse for PluginEvent {
    fn topic(&self) -> Topic {
        Topic::Plugin
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
pub enum PluginEventType {
    Updated
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Deployment {
    pub id: Uuid,
    pub event: DeploymentEvent,
    pub simulation_id: Option<Uuid>,
    pub strategy_name: String,
    pub strategy_version: String,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
}

impl Synapse for Deployment {
    fn topic(&self) -> Topic {
        Topic::Deployment
    }
}

impl AuditTags for Deployment {
    fn audit_tags(&self) -> Vec<String> {
        let mut tags = vec![CommonAuditTags::Deployment.to_string()];
        match self.event {
            DeploymentEvent::Created => tags.push(CommonAuditTags::Created.to_string()),
            DeploymentEvent::Deleted => tags.push(CommonAuditTags::Deleted.to_string())
        }
        tags
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DeploymentEvent {
    Created,
    Deleted,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct InstrumentId {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub pair: CurrencyPair,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Copy)]
pub struct CurrencyPair {
    pub target: Currency,
    pub source: Currency,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tick {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub instrument_id: InstrumentId,
    pub price: f64,
}

impl Synapse for Tick {
    fn topic(&self) -> Topic {
        Topic::Tick
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum Exchange {
    OKX
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Exchange {
    type Err = Error;
    fn from_str(input: &str) -> Result<Exchange, Self::Err> {
        match input {
            "OKX" => Ok(Exchange::OKX),
            input => bail!("Unknown exchange: {input}")
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum Currency {
    BTC,
    USDT,
    TON,
    XRP,
    FTM,
    LINK,
    DOT,
    ETH,
    USDC,
    TUSD,
    USDK,
    ADA,
    TRX,
    LTC,
    UNI,
    PAX,
    JFI,
    OKB,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Currency {
    type Err = Error;
    fn from_str(input: &str) -> Result<Currency, Self::Err> {
        match input {
            "BTC" | "Btc" => Ok(Currency::BTC),
            "USDT" | "Usdt" => Ok(Currency::USDT),
            "TON" => Ok(Currency::TON),
            "XRP" => Ok(Currency::XRP),
            "FTM" => Ok(Currency::FTM),
            "LINK" => Ok(Currency::LINK),
            "DOT" => Ok(Currency::DOT),
            "ETH" => Ok(Currency::ETH),
            "USDC" => Ok(Currency::USDC),
            "TUSD" => Ok(Currency::TUSD),
            "USDK" => Ok(Currency::USDK),
            "ADA" => Ok(Currency::ADA),
            "TRX" => Ok(Currency::TRX),
            "LTC" => Ok(Currency::LTC),
            "UNI" => Ok(Currency::UNI),
            "PAX" => Ok(Currency::PAX),
            "JFI" => Ok(Currency::JFI),
            "OKB" => Ok(Currency::OKB),
            input => bail!("Unknown currency: {input}")
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum MarketType {
    Spot,
    Margin,
}

impl fmt::Display for MarketType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_uppercase())
    }
}

impl FromStr for MarketType {
    type Err = Error;
    fn from_str(input: &str) -> Result<MarketType, Self::Err> {
        match input {
            "SPOT" => Ok(MarketType::Spot),
            "MARGIN" => Ok(MarketType::Margin),
            input => bail!("Unknown market type: {input}")
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
pub enum Action {
    OrderAction(OrderAction)
}

impl Synapse for Action {
    fn topic(&self) -> Topic {
        Topic::Action
    }
}

impl AuditTags for Action {
    fn audit_tags(&self) -> Vec<String> {
        let mut tags = vec![CommonAuditTags::Action.to_string()];
        match self {
            Action::OrderAction(order_action) => {
                tags.push(CommonAuditTags::OrderAction.to_string());
                tags.push(order_action.exchange.to_string());
                match order_action.status {
                    OrderStatus::Created => tags.push(CommonAuditTags::Created.to_string()),
                    OrderStatus::InProgress => tags.push(CommonAuditTags::InProgress.to_string()),
                    OrderStatus::Failed(_) => tags.push(CommonAuditTags::Failed.to_string()),
                    OrderStatus::Completed => tags.push(CommonAuditTags::Completed.to_string()),
                    OrderStatus::Canceled => tags.push(CommonAuditTags::Canceled.to_string()),
                }

                match &order_action.order {
                    OrderActionType::CreateOrder(create_order) => {
                        tags.push(create_order.pair.target.to_string());
                        tags.push(create_order.pair.source.to_string());
                    }
                    OrderActionType::PatchOrder => {}
                    OrderActionType::CancelOrder => {}
                }
            }
        }
        tags
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderAction {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub strategy_name: String,
    pub strategy_version: String,
    pub timestamp: DateTime<Utc>,
    pub exchange: Exchange,
    pub status: OrderStatus,
    pub order: OrderActionType,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum OrderStatus {
    Created,
    InProgress,
    Failed(String),
    Completed,
    Canceled,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct OrderInProgressStatus {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "operation")]
pub enum OrderActionType {
    CreateOrder(CreateOrder),
    PatchOrder,
    CancelOrder,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOrder {
    pub id: String,
    pub pair: CurrencyPair,
    pub market_type: OrderMarketType,
    pub order_type: OrderType,
    pub side: Side,
    pub size: f64,
    pub stop_lose: Option<Trigger>,
    pub take_profit: Option<Trigger>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Trigger {
    pub trigger_px: f64,
    pub order_px: f64,
}

impl Trigger {
    pub fn new(trigger_px: f64, order_px: f64) -> Option<Self> {
        Some(Self {
            trigger_px,
            order_px,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_uppercase())
    }
}

impl FromStr for Side {
    type Err = Error;
    fn from_str(input: &str) -> Result<Side, Self::Err> {
        match input {
            "Buy" | "BUY" => Ok(Side::Buy),
            "Sell" | "SELL" => Ok(Side::Sell),
            input => bail!("Unknown side: {input}")
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum OrderType {
    Limit(f64),
    Market,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum OrderMarketType {
    Spot,
    Margin(MarginMode),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum MarginMode {
    Cross,
    Isolated,
}
