pub use error::{BybitError, Result};

mod credential;
pub mod enums;
mod error;
mod parser;
pub mod rest;
pub mod websocket;

#[cfg(test)]
mod tests {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::SubscriberBuilder;

    #[allow(unused)]
    const LOGGING_LEVEL: &str = "DEBUG";

    #[allow(unused)]
    pub fn init_logger(directives: &str) {
        let subscriber = SubscriberBuilder::default()
            .with_env_filter(EnvFilter::new(directives))
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");
    }

    mod rest {
        use chrono::Utc;
        use uuid::Uuid;

        use crate::bybit::enums::{Category, Side, Timeframe};
        use crate::bybit::rest::{BybitRest, CandlesRequest, OrderDetailsRequest, PlaceOrderRequest, RateLimitedRestClient};
        use crate::bybit::rest::Size::{Source, Target};
        use crate::bybit::tests::{init_logger, LOGGING_LEVEL};

        pub fn build_private_rest_client() -> BybitRest {
            BybitRest::with_credential(
                "https://api-testnet.bybit.com",
                "",
                "",
            )
        }

        pub fn build_public_rest_rate_limited_client() -> RateLimitedRestClient {
            RateLimitedRestClient::new(BybitRest::new("https://api-testnet.bybit.com"))
        }

        #[tokio::test]
        async fn test_get_candles_history() {
            init_logger(LOGGING_LEVEL);
            let rest_client = build_public_rest_rate_limited_client();
            let request = CandlesRequest {
                symbol: "BTCUSDT".to_string(),
                interval: Timeframe::Min5.as_topic(),
                category: Category::Spot,
                start: Utc::now().timestamp_millis().into(),
                end: None,
                limit: 1000.into(),
            };

            if let Ok(response) = rest_client.request(request).await {
                dbg!(response);
            } else {
                panic!("Error during candles retrieving");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_get_orders() {
            init_logger(LOGGING_LEVEL);
            let rest_client = build_private_rest_client();
            let request = OrderDetailsRequest {
                symbol: "BTCUSDT".to_string().into(),
                category: Category::Spot,
                base_coin: None,
                settle_coin: None,
                order_id: None,
                order_link_id: None,
                order_filter: None,
                order_status: None,
                start_time: None,
                end_time: None,
                limit: None,
                cursor: None,
            };

            if let Ok(response) = rest_client.request(request).await {
                dbg!(response);
            } else {
                panic!("Error during orders retrieving");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_margin_market_buy_order() {
            init_logger(LOGGING_LEVEL);
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                Some(Uuid::new_v4().to_string()),
                "BTCUSDT",
                Category::Spot,
                Side::Buy,
                Source(100.0),
                true,
            );
            let response = rest_client.request(request).await;
            if let Ok(response) = response {
                dbg!(response);
            } else {
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_margin_market_sell_order() {
            init_logger(LOGGING_LEVEL);
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                Some(Uuid::new_v4().to_string()),
                "BTCUSDT",
                Category::Spot,
                Side::Sell,
                Target(0.0034),
                true,
            );
            let response = rest_client.request(request).await;
            if let Ok(response) = response {
                dbg!(response);
            } else {
                panic!("Error during order creation");
            }
        }
    }

    mod websocket {
        use std::time::Duration;

        use async_trait::async_trait;
        use serde_json::{from_value, Value};
        use tracing::error;

        use crate::bybit::enums::Timeframe;
        use crate::bybit::tests::{init_logger, LOGGING_LEVEL};
        use crate::bybit::websocket::{CandleResponse, OrderDetailsResponse, TickerResponse};
        use crate::bybit::websocket::*;

        struct TickerLtHandler;

        #[async_trait]
        impl WsMessageHandler for TickerLtHandler {
            type Type = TickerResponse;

            async fn convert_data(&mut self, _topic: String, data: Value,
            ) -> Option<Self::Type> {
                from_value(data).map_err(|err| error!("Error during message conversation: {err}")).ok()
            }

            async fn handle(&mut self, message: Self::Type) {
                dbg!(&message);
            }
        }

        struct OrderHandler;

        #[async_trait]
        impl WsMessageHandler for OrderHandler {
            type Type = Vec<OrderDetailsResponse>;

            async fn convert_data(&mut self, _topic: String, data: Value) -> Option<Self::Type> {
                from_value(data).map_err(|err| error!("Error during message conversation: {err}")).ok()
            }

            async fn handle(&mut self, message: Self::Type) {
                dbg!(&message);
            }
        }

        struct CandlesHandler;

        #[async_trait]
        impl WsMessageHandler for CandlesHandler {
            type Type = Vec<CandleResponse>;

            async fn convert_data(&mut self, _topic: String, data: Value) -> Option<Self::Type> {
                from_value(data).map_err(|err| error!("Error during message conversation: {err}")).ok()
            }

            async fn handle(&mut self, message: Self::Type) {
                dbg!(&message);
            }
        }

        async fn build_private_ws_client<H: WsMessageHandler>(handler: H) -> BybitWsClient {
            BybitWsClient::private(
                "wss://stream-testnet.bybit.com",
                "",
                "",
                handler,
            ).await
        }

        async fn build_public_ws_client<H: WsMessageHandler>(handler: H) -> BybitWsClient {
            BybitWsClient::public("wss://stream-testnet.bybit.com", handler).await
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_handle_mark_price() {
            init_logger(LOGGING_LEVEL);
            let client = build_public_ws_client(TickerLtHandler).await;
            client
                .send(Command::subscribe(vec![Channel::Ticker("BTCUSDT".into())]))
                .await;

            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_handle_order() {
            init_logger(LOGGING_LEVEL);
            let client = build_private_ws_client(OrderHandler).await;
            client
                .send(Command::subscribe(vec![Channel::Orders]))
                .await;

            tokio::time::sleep(Duration::from_secs(30)).await;

            // let rest_client = build_private_rest_client();
            // let mut request = PlaceOrderRequest::market(
            //     "BTC-USDT",
            //     TdMode::Cash,
            //     None,
            //     Side::Buy,
            //     Source(100.0),
            //     None,
            //     None,
            // );
            // let order_id = "test";
            // request.cl_ord_id = Some(order_id.to_string());
            // let [response] = rest_client.request(request).await.unwrap();
            // if response.s_code != 0 {
            //     dbg!(response);
            //     panic!("Error during order creation");
            // }
            // tokio::time::sleep(Duration::from_secs(1)).await;
        }

        #[tokio::test]
        async fn test_handle_candles() {
            init_logger(LOGGING_LEVEL);
            let client = build_public_ws_client(CandlesHandler).await;
            client
                .send(Command::subscribe(vec![Channel::Candles((Timeframe::Min1, "BTCUSDT").into())]))
                .await;

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}
