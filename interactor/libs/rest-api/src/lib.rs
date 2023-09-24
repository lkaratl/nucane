pub mod endpoints {
    pub const GET_SUBSCRIPTIONS: &str = "/api/v1/interactor/subscriptions";
    pub const POST_SUBSCRIBE: &str = "/api/v1/interactor/subscriptions";
    pub const DELETE_UNSUBSCRIBE: &str = "/api/v1/interactor/subscriptions";
    pub const POST_EXECUTE_ACTIONS: &str = "/api/v1/interactor/actions";
    pub const GET_CANDLES: &str = "/api/v1/interactor/candles";
    pub const GET_PRICE: &str = "/api/v1/interactor/price";
}

pub mod path_query {
    use serde::{Serialize, Deserialize};
    use domain_model::{Currency, Exchange, MarketType, Timeframe};
    use serde_inline_default::serde_inline_default;

    #[serde_inline_default]
    #[derive(Debug, Deserialize, Serialize)]
    #[allow(unused)]
    pub struct CandlesQuery {
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

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PriceQuery {
        pub timestamp: Option<i64>,
        pub exchange: Exchange,
        pub market_type: MarketType,
        pub target: Currency,
        pub source: Currency,
    }
}
