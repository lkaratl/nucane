use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Spot,
    Linear,
    Inverse,
    Option,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum OrderStatus {
    Created,
    New,
    Rejected,
    PartiallyFilled,
    PartiallyFilledCanceled,
    Filled,
    Cancelled,
    Untriggered,
    Triggered,
    Deactivated,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum OrderCancelType {
    CancelByUser,
    CancelByReduceOnly,
    CancelByPrepareLiq,
    CancelAllBeforeLiq,
    CancelByPrepareAdl,
    CancelAllBeforeAdl,
    CancelByAdmin,
    CancelByTpSlTsClear,
    CancelByPzSideCh,
    CancelBySmp,
    UNKNOWN,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum OrderTimeInForce {
    GTC,
    IOC,
    FOK,
    PostOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum OrderType {
    Market,
    Limit,
    UNKNOWN,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum OrderFilter {
    Order,
    TpslOrder,
    StopOrder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Timeframe {
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    H1,
    H2,
    H4,
    H6,
    H12,
    D,
    W,
    M,
}

impl Timeframe {
    pub fn as_topic(&self) -> String {
        match self {
            Timeframe::Min1 => "1",
            Timeframe::Min3 => "3",
            Timeframe::Min5 => "5",
            Timeframe::Min15 => "15",
            Timeframe::Min30 => "30",
            Timeframe::H1 => "60",
            Timeframe::H2 => "120",
            Timeframe::H4 => "240",
            Timeframe::H6 => "360",
            Timeframe::H12 => "720",
            Timeframe::D => "D",
            Timeframe::W => "W",
            Timeframe::M => "M",
        }.into()
    }
}
