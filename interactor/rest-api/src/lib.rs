pub mod endpoints {
    pub const GET_CANDLES_HISTORY: &str = "/api/v1/interactor/candles/history";
}

pub mod path_query {
    use serde::{Serialize, Deserialize};
    use domain_model::{Currency, Exchange, MarketType, Timeframe};
    use serde_inline_default::serde_inline_default;

    #[serde_inline_default]
    #[derive(Debug, Deserialize, Serialize)]
    #[allow(unused)]
    pub struct CandlesHistoryQuery {
        pub exchange: Exchange,
        pub market_type: MarketType,
        pub target: Currency,
        pub source: Currency,
        pub timeframe: Timeframe,
        pub from_timestamp: Option<i64>,
        pub to_timestamp: Option<i64>,
        #[serde_inline_default(100)]
        pub limit: u8,
    }
}
