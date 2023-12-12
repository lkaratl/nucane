use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::bybit::enums::Timeframe;

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
    timeframe: Timeframe,
    inst_id: String,
}

impl From<(Timeframe, &str)> for CandleTopic {
    fn from(value: (Timeframe, &str)) -> Self {
        CandleTopic {
            timeframe: value.0,
            inst_id: value.1.to_string(),
        }
    }
}
