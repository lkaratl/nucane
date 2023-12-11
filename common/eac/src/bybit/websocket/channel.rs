use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "channel")]
pub enum Channel {
    Ticker(String),
    Orders,
    Positions,
    Candles(CandleTopic),
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Ticker(inst_id) => {
                write!(f, "tickers.{inst_id}")
            }
            Channel::Orders => {
                write!(f, "order")
            }
            Channel::Positions => {
                write!(f, "wallet")
            }
            Channel::Candles(topic) => {
                write!(f, "kline.{}.{}", topic.timeframe.as_topic(), topic.inst_id)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandleTopic {
    timeframe: CandleTimeframe,
    inst_id: String,
}

impl From<(CandleTimeframe, &str)> for CandleTopic {
    fn from(value: (CandleTimeframe, &str)) -> Self {
        CandleTopic {
            timeframe: value.0,
            inst_id: value.1.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CandleTimeframe {
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

impl CandleTimeframe {
    pub fn as_topic(&self) -> String {
        match self {
            CandleTimeframe::Min1 => "1",
            CandleTimeframe::Min3 => "3",
            CandleTimeframe::Min5 => "5",
            CandleTimeframe::Min15 => "15",
            CandleTimeframe::Min30 => "30",
            CandleTimeframe::H1 => "60",
            CandleTimeframe::H2 => "120",
            CandleTimeframe::H4 => "240",
            CandleTimeframe::H6 => "360",
            CandleTimeframe::H12 => "720",
            CandleTimeframe::D => "D",
            CandleTimeframe::W => "W",
            CandleTimeframe::M => "M",
        }.into()
    }
}

