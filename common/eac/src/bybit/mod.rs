pub use error::{OkExError, Result};

mod credential;
pub mod enums;
mod error;
mod parser;
pub mod rest;
pub mod websocket;

// todo created order verification
// account settings:
// - single-currency margin
// - isolated margin auto transfer
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
        use std::env;

        use crate::bybit::enums::{Side, TdMode};
        use crate::bybit::rest::{
            OkExRest, PlaceOrderRequest, RateLimitedRestClient, Trigger,
        };
        use crate::bybit::rest::Size::{Source, Target};

        pub fn build_private_rest_client() -> OkExRest {
            OkExRest::with_credential(
                "https://www.okx.com",
                true,
                &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
            )
        }

        #[allow(unused)]
        pub fn build_public_rest_rate_limited_client() -> RateLimitedRestClient {
            RateLimitedRestClient::new(OkExRest::new("https://www.okx.com", true))
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_market_buy_order() {
            // init_logger(LOGGING_LEVEL);
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Buy,
                Source(100.0),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_market_sell_order() {
            // init_logger(LOGGING_LEVEL);
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Sell,
                Target(0.0038),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_limit_buy_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Buy,
                26000.0,
                Target(0.0038),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_limit_sell_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Sell,
                26000.0,
                Target(0.0038),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[allow(unused)]
        // todo fix isolated orders
        // #[tokio::test]
        async fn test_place_margin_market_buy_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Isolated,
                None,
                Side::Buy,
                Source(100.0),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[allow(unused)]
        // todo fix isolated orders
        // #[tokio::test]
        async fn test_place_margin_market_sell_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Isolated,
                None,
                Side::Sell,
                Target(0.0038),
                None,
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_limit_buy_order_with_sl() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Buy,
                26_000.0,
                Target(0.0038),
                Trigger::new(10_000.0, 9_900.0),
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_limit_buy_order_with_tp() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Buy,
                26_000.0,
                Target(0.0038),
                None,
                Trigger::new(100_000.0, 100_100.0),
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[ignore = "failed ci"]
        #[tokio::test]
        async fn test_place_spot_limit_sell_order_with_sl_and_tp() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit(
                "BTC-USDT",
                TdMode::Cash,
                None,
                Side::Sell,
                26_000.0,
                Target(0.0038),
                Trigger::new(100_000.0, 100_100.0),
                Trigger::new(10_000.0, 9_900.0),
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[allow(unused)]
        // todo fix isolated orders
        // #[tokio::test]
        async fn test_place_margin_market_buy_order_with_sl() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Isolated,
                None,
                Side::Buy,
                Source(100.0),
                Trigger::new(10_000.0, 9_900.0),
                None,
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[allow(unused)]
        // todo fix isolated orders
        // #[tokio::test]
        async fn test_place_margin_market_buy_order_with_tp() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Isolated,
                None,
                Side::Buy,
                Source(100.0),
                None,
                Trigger::new(100_000.0, 100_100.0),
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[allow(unused)]
        // todo fix isolated orders
        // #[tokio::test]
        async fn test_place_margin_market_sell_order_with_sl_and_tp() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market(
                "BTC-USDT",
                TdMode::Isolated,
                None,
                Side::Sell,
                Target(0.0038),
                Trigger::new(100_000.0, 100_100.0),
                Trigger::new(10_000.0, 9_900.0),
            );
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        // #[allow(unused)]
        // // todo try to break this test
        // // #[tokio::test]
        // async fn test_request_rate_limit() {
        //     init_logger(LOGGING_LEVEL);
        //     let rest_client = Arc::new(tokio::sync::Mutex::new(
        //         build_public_rest_rate_limited_client(),
        //     ));
        //     let request = CandlesHistoryRequest {
        //         inst_id: "BTC-USDT".to_string(),
        //         bar: Some("1H".to_string()),
        //         before: Some(1682899200000u64.to_string()),
        //         after: Some(1682899200000u64.to_string()),
        //         limit: Some(100),
        //     };
        //     let mut handles = Vec::new();
        //     for _ in 0..=100 {
        //         let handle = tokio::spawn({
        //             let rest_client = Arc::clone(&rest_client);
        //             let request = request.clone();
        //             async move {
        //                 let response = rest_client.lock().await.request(request).await.unwrap();
        //                 debug!("{:?}", response)
        //             }
        //         });
        //         handles.push(handle);
        //     }
        //     futures::future::join_all(handles).await;
        // }
    }

    mod websocket {
        use std::time::Duration;

        use async_trait::async_trait;
        use serde_json::{from_value, Value};

        use crate::bybit::rest::TickerLtResponse;
        use crate::bybit::tests::{init_logger, LOGGING_LEVEL};
        use crate::bybit::websocket::{BybitWsClient, Channel, Command, WsMessageHandler};

        struct MarkPriceHandler;

        #[async_trait]
        impl WsMessageHandler for MarkPriceHandler {
            type Type = TickerLtResponse;

            async fn convert_data(&mut self, topic: String, data: Value,
            ) -> Option<Self::Type> {
                from_value(data).ok()
            }

            async fn handle(&mut self, message: Self::Type) {
                dbg!(&message);
            }
        }

        struct OrderHandler;

        // #[async_trait]
        // impl WsMessageHandler for OrderHandler {
        //     type Type = OrderDetailsResponse;
        //
        //     async fn convert_data(
        //         &mut self,
        //         arg: Channel,
        //         _action: Option<Action>,
        //         mut data: Vec<Value>,
        //     ) -> Option<Self::Type> {
        //         assert!(matches!(arg, Channel::Orders { .. }));
        //         let data = data.pop().unwrap();
        //         from_value(data).ok()
        //     }
        //
        //     async fn handle(&mut self, message: Self::Type) {
        //         dbg!(&message);
        //     }
        // }

        async fn build_private_ws_client<H: WsMessageHandler>(handler: H) -> BybitWsClient {
            BybitWsClient::private(
                "wss://stream-testnet.bybit.com",
                "vQU4uWbU3V4VROyVqq",
                "gACukoe74WBZBApClNsSOjNhEPrWhRMkFVGu",
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
            let client = build_public_ws_client(MarkPriceHandler).await;
            client
                .send(Command::subscribe(vec![Channel::TickerLt("BTC3SUSDT".into())]))
                .await;

            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        // #[ignore = "failed ci"]
        // #[tokio::test]
        // async fn test_handle_order() {
        //     // init_logger(LOGGING_LEVEL);
        //     let client = build_private_ws_client(OrderHandler).await;
        //     client
        //         .send(Command::subscribe(vec![Channel::Orders {
        //             inst_type: InstType::Any,
        //             inst_id: None,
        //             uly: None,
        //         }]))
        //         .await;
        //
        //     tokio::time::sleep(Duration::from_secs(30)).await;
        //
        //     let rest_client = build_private_rest_client();
        //     let mut request = PlaceOrderRequest::market(
        //         "BTC-USDT",
        //         TdMode::Cash,
        //         None,
        //         Side::Buy,
        //         Source(100.0),
        //         None,
        //         None,
        //     );
        //     let order_id = "test";
        //     request.cl_ord_id = Some(order_id.to_string());
        //     let [response] = rest_client.request(request).await.unwrap();
        //     if response.s_code != 0 {
        //         dbg!(response);
        //         panic!("Error during order creation");
        //     }
        //     tokio::time::sleep(Duration::from_secs(1)).await;
        // }

        // #[allow(unused, unused_assignments)]
        // // #[tokio::test] // todo not implemented yet
        // async fn test_drop_ws_client() {
        //     {
        //         let client = build_public_ws_client(MarkPriceHandler).await;
        //         client
        //             .send(Command::subscribe(vec![Channel::MarkPrice {
        //                 inst_id: "BTC-USDT".to_string(),
        //             }]))
        //             .await;
        //
        //         thread::sleep(Duration::from_secs(1));
        //     }
        //     dbg!("Drop client");
        //     thread::sleep(Duration::from_secs(1));
        // }
    }
}
