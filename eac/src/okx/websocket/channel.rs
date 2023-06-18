use serde::{Deserialize, Serialize};

use crate::enums::InstType;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "channel")]
pub enum Channel {
    #[serde(rename = "books")]
    Books {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "books5")]
    Books5 {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "books50-l2-tbt")]
    Books50L2Tbt {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "books-l2-tbt")]
    BooksL2Tbt {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "instruments")]
    Instruments {
        #[serde(rename = "instType")]
        inst_type: InstType,
    },
    #[serde(rename = "orders")]
    Orders {
        #[serde(rename = "instType")]
        inst_type: InstType,
        uly: Option<String>,
        #[serde(rename = "instId")]
        inst_id: Option<String>,
    },
    #[serde(rename = "price-limit")]
    PriceLimit {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "tickers")]
    Tickers {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "mark-price")]
    MarkPrice {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "account")]
    Account {
        ccy: Option<String>,
    },
    #[serde(rename = "candle1m")]
    Candle1M {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle5m")]
    Candle5M {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle15m")]
    Candle15M {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle30m")]
    Candle30M {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle1H")]
    Candle1H {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle2H")]
    Candle2H {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle4H")]
    Candle4H {
        #[serde(rename = "instId")]
        inst_id: String,
    },
    #[serde(rename = "candle1D")]
    Candle1D {
        #[serde(rename = "instId")]
        inst_id: String,
    },
}

impl Channel {
    pub fn books(inst_id: &str) -> Self {
        Self::Books {
            inst_id: inst_id.into(),
        }
    }
    pub fn books5(inst_id: &str) -> Self {
        Self::Books5 {
            inst_id: inst_id.into(),
        }
    }

    pub fn books50_l2_tbt(inst_id: &str) -> Self {
        Self::Books50L2Tbt {
            inst_id: inst_id.into(),
        }
    }

    pub fn books_l2_tbt(inst_id: &str) -> Self {
        Self::BooksL2Tbt {
            inst_id: inst_id.into(),
        }
    }

    pub fn instruments(inst_type: InstType) -> Self {
        Self::Instruments { inst_type }
    }

    pub fn tickers(inst_id: &str) -> Self {
        Self::Tickers {
            inst_id: inst_id.into(),
        }
    }

    pub fn price_limit(inst_id: &str) -> Self {
        Self::PriceLimit {
            inst_id: inst_id.into(),
        }
    }

    pub fn mark_price(inst_id: &str) -> Self {
        Self::MarkPrice {
            inst_id: inst_id.into(),
        }
    }

    pub fn account(ccy: Option<String>) -> Self {
        Self::Account {
            ccy,
        }
    }

    pub fn candle_1m(inst_id: &str) -> Self {
        Self::Candle1M {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_5m(inst_id: &str) -> Self {
        Self::Candle5M {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_15m(inst_id: &str) -> Self {
        Self::Candle15M {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_30m(inst_id: &str) -> Self {
        Self::Candle30M {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_1h(inst_id: &str) -> Self {
        Self::Candle1H {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_2h(inst_id: &str) -> Self {
        Self::Candle2H {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_4h(inst_id: &str) -> Self {
        Self::Candle4H {
            inst_id: inst_id.into(),
        }
    }

    pub fn candle_1d(inst_id: &str) -> Self {
        Self::Candle1D {
            inst_id: inst_id.into(),
        }
    }
}
