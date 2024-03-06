use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Error};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod drawing;

#[derive(Debug, Deserialize, Serialize)]
pub struct Simulation {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub positions: Vec<SimulationPosition>,
    pub deployments: Vec<SimulationDeployment>,

    pub ticks_len: u32,
    pub actions_count: u32,
    pub active_orders: Vec<Order>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationDeployment {
    pub deployment_id: Option<Uuid>,
    // todo remove Option
    pub timeframe: Timeframe,
    pub plugin_id: PluginId,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
    pub indicators: Vec<Indicator>,
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
        let side = if value.end >= 0.0 {
            Side::Buy
        } else {
            Side::Sell
        };
        Position::new(
            Some(value.simulation_id),
            value.exchange,
            value.currency,
            side,
            value.end,
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl Candle {
    pub fn avg_price(&self) -> f64 {
        (self.open_price + self.highest_price + self.lowest_price + self.close_price) / 4.
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
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
            input => bail!("Unknown candle status: {input}"),
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

impl Timeframe {
    pub fn as_sec(&self) -> i64 {
        match self {
            Timeframe::OneS => 1,
            Timeframe::OneM => 60,
            Timeframe::FiveM => 300,
            Timeframe::FifteenM => 900,
            Timeframe::ThirtyM => 1800,
            Timeframe::OneH => 3600,
            Timeframe::TwoH => 7200,
            Timeframe::FourH => 14400,
            Timeframe::OneD => 345600,
        }
    }
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
            input => bail!("Unknown market type: {input}"),
        }
    }
}

impl From<Timeframe> for Duration {
    fn from(value: Timeframe) -> Self {
        match value {
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
    pub fn new(
        simulation_id: Option<Uuid>,
        exchange: Exchange,
        currency: Currency,
        side: Side,
        size: f64,
    ) -> Self {
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
    pub size: Size,
    pub fee: f64,
    pub avg_fill_price: f64,
    pub stop_loss: Option<Trigger>,
    pub avg_sl_price: f64,
    pub take_profit: Option<Trigger>,
    pub avg_tp_price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LP {
    pub id: String,
    pub price: f64,
    pub size: Size,
    pub fee: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Size {
    Target(f64),
    Source(f64),
}

impl Size {
    pub fn as_target(&self, quote: f64) -> Self {
        match self {
            Size::Target(_) => self.clone(),
            Size::Source(size) => Self::Target(size / quote)
        }
    }

    pub fn as_source(&self, quote: f64) -> Self {
        match self {
            Size::Target(size) => Self::Source(size * quote),
            Size::Source(_) => self.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: PluginId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PluginId {
    pub name: String,
    pub version: i64,
}

impl PluginId {
    pub fn new(name: &str, version: i64) -> Self {
        Self {
            name: name.to_string(),
            version,
        }
    }
}

impl From<PluginBinary> for PluginInfo {
    fn from(value: PluginBinary) -> Self {
        Self { id: value.id }
    }
}

impl From<&PluginBinary> for PluginInfo {
    fn from(value: &PluginBinary) -> Self {
        Self {
            id: value.id.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginBinary {
    pub id: PluginId,
    pub binary: Vec<u8>,
}

impl PluginBinary {
    pub fn new(id: PluginId, binary: &[u8]) -> Self {
        Self {
            id,
            binary: binary.to_vec(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeploymentInfo {
    pub id: Uuid,
    pub status: DeploymentStatus,
    pub simulation_id: Option<Uuid>,
    pub state_id: Option<Uuid>,
    pub plugin_id: PluginId,
    pub params: HashMap<String, String>,
    pub subscriptions: Vec<InstrumentId>,
    pub indicators: Vec<Indicator>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DeploymentStatus {
    Created,
    Deleted,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewDeployment {
    pub simulation_id: Option<Uuid>,
    pub plugin_id: PluginId,
    pub state_id: Option<Uuid>,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subscription {
    pub simulation_id: Option<Uuid>,
    pub deployment_id: Uuid,
    pub instruments: Vec<InstrumentId>,
}

impl From<&DeploymentInfo> for Subscription {
    fn from(value: &DeploymentInfo) -> Self {
        Self {
            simulation_id: value.simulation_id,
            deployment_id: value.id,
            instruments: value.subscriptions.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subscriptions {
    pub instrument_id: InstrumentId,
    pub deployments: HashSet<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct InstrumentId {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub pair: CurrencyPair,
}

impl InstrumentId {
    pub fn new(exchange: Exchange, market_type: MarketType, pair: CurrencyPair) -> Self {
        Self {
            exchange,
            market_type,
            pair,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Copy)]
pub struct CurrencyPair {
    pub target: Currency,
    pub source: Currency,
}

impl CurrencyPair {
    pub fn new(target: Currency, source: Currency) -> Self {
        Self { target, source }
    }
}

impl From<(Currency, Currency)> for CurrencyPair {
    fn from(value: (Currency, Currency)) -> Self {
        Self {
            target: value.0,
            source: value.1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tick {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub instrument_id: InstrumentId,
    pub price: f64,
}

impl Tick {
    pub fn new(simulation_id: Option<Uuid>, timestamp: DateTime<Utc>, instrument_id: InstrumentId, price: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            simulation_id,
            timestamp,
            instrument_id,
            price,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum Exchange {
    OKX,
    BYBIT
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
            "BYBIT" => Ok(Exchange::OKX),
            input => bail!("Unknown exchange: {input}"),
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
    XTZ,
    ATOM,
    ICP,
    APT,
    FLOW,
    ARB,
    SUI,
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
    DOGE,
    SOL,
    MATIC,
    AVAX,
    INJ,
    IMX,
    TIA,
    GALA,
    APE,
    FIL,
    MANA,
    SAND,
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
            "BTC" => Ok(Currency::BTC),
            "USDT" => Ok(Currency::USDT),
            "TON" => Ok(Currency::TON),
            "XRP" => Ok(Currency::XRP),
            "FTM" => Ok(Currency::FTM),
            "LINK" => Ok(Currency::LINK),
            "DOT" => Ok(Currency::DOT),
            "ETH" => Ok(Currency::ETH),
            "XTZ" => Ok(Currency::XTZ),
            "ATOM" => Ok(Currency::ATOM),
            "ICP" => Ok(Currency::ICP),
            "APT" => Ok(Currency::APT),
            "FLOW" => Ok(Currency::FLOW),
            "ARB" => Ok(Currency::ARB),
            "SUI" => Ok(Currency::SUI),
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
            "DOGE" => Ok(Currency::DOGE),
            "AVAX" => Ok(Currency::AVAX),
            "SOL" => Ok(Currency::SOL),
            "MATIC" => Ok(Currency::MATIC),
            "INJ" => Ok(Currency::INJ),
            "IMX" => Ok(Currency::IMX),
            "TIA" => Ok(Currency::TIA),
            "GALA" => Ok(Currency::GALA),
            "APE" => Ok(Currency::APE),
            "FIL" => Ok(Currency::FIL),
            "MANA" => Ok(Currency::MANA),
            "SAND" => Ok(Currency::SAND),
            input => bail!("Unknown currency: {input}"),
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
            input => bail!("Unknown market type: {input}"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
pub enum Action {
    OrderAction(OrderAction),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderAction {
    pub id: Uuid,
    pub simulation_id: Option<Uuid>,
    pub plugin_id: PluginId,
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

impl OrderStatus {
    pub fn is_finished(&self) -> bool {
        match self {
            OrderStatus::Created | OrderStatus::InProgress => false,
            OrderStatus::Failed(_) | OrderStatus::Completed | OrderStatus::Canceled => true,
        }
    }
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
    CancelOrder(CancelOrder),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOrder {
    pub id: String,
    pub pair: CurrencyPair,
    pub market_type: OrderMarketType,
    pub order_type: OrderType,
    pub side: Side,
    pub size: Size,
    pub stop_loss: Option<Trigger>,
    pub take_profit: Option<Trigger>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CancelOrder {
    pub id: String,
    pub pair: CurrencyPair,
    pub market_type: OrderMarketType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Trigger {
    pub trigger_px: f64,
    pub order_px: OrderType,
}

impl Trigger {
    pub fn new(trigger_px: f64, order_px: OrderType) -> Option<Self> {
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
            input => bail!("Unknown side: {input}"),
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
    Cross(Currency),
    Isolated,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSimulation {
    pub start: i64,
    pub end: i64,
    pub positions: Vec<CreateSimulationPosition>,
    pub strategies: Vec<CreateSimulationDeployment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSimulationDeployment {
    pub simulation_id: Option<Uuid>,
    pub timeframe: Timeframe,
    pub plugin_id: PluginId,
    pub params: HashMap<String, String>,
}

pub fn convert_to_simulation_deployment(value: CreateSimulationDeployment) -> SimulationDeployment {
    SimulationDeployment {
        deployment_id: None,
        timeframe: value.timeframe,
        plugin_id: value.plugin_id,
        params: value.params,
        subscriptions: Vec::new(),
        indicators: Vec::new(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSimulationPosition {
    pub exchange: Exchange,
    pub currency: Currency,
    pub side: Side,
    pub size: f64,
}

pub fn convert(value: CreateSimulationPosition, simulation_id: Uuid) -> SimulationPosition {
    SimulationPosition {
        simulation_id,
        exchange: value.exchange,
        currency: value.currency,
        start: value.size,
        end: value.size,
        diff: 0.0,
        fees: 0.0,
    }
}

impl From<CreateSimulation> for Simulation {
    fn from(value: CreateSimulation) -> Self {
        let simulation_id = Uuid::new_v4();
        let positions = value
            .positions
            .into_iter()
            .map(|position| convert(position, simulation_id))
            .collect();
        let deployments = value
            .strategies
            .into_iter()
            .map(convert_to_simulation_deployment)
            .collect();

        Self {
            id: simulation_id,
            timestamp: Utc::now(),
            start: Utc.timestamp_millis_opt(value.start).unwrap(),
            end: Utc.timestamp_millis_opt(value.end).unwrap(),
            positions,
            deployments,
            ticks_len: 0,
            actions_count: 0,
            active_orders: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Indicator {
    SMA(u64),
    EMA(u64),
    // period & multiplier
    BB(u64, f64),
    PSAR,
}

impl Indicator {
    pub fn as_multiplier(&self) -> u64 {
        match self {
            Indicator::SMA(period) => *period,
            Indicator::EMA(period) => *period,
            Indicator::BB(period, _) => *period,
            Indicator::PSAR => 100
        }
    }
}

impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
