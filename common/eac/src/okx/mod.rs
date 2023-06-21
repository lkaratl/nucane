pub use error::{OkExError, Result};

mod credential;
pub mod enums;
mod error;
mod parser;
pub mod rest;
pub mod websocket;

// todo add logging
// account settings:
// - single-currency margin
// - isolated margin auto transfer
#[cfg(test)]
mod tests {
    mod rest {
        use std::env;
        use crate::enums::{Side, TdMode};
        use crate::rest::{OkExRest, PlaceOrderRequest};

        pub fn build_private_rest_client() -> OkExRest {
            OkExRest::with_credential("https://www.okx.com", true,
                                      &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                                      &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
                                      &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-PASSPHRASE").unwrap())
        }

        #[tokio::test]
        async fn test_place_spot_market_buy_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market("BTC-USDT", TdMode::Cash, Side::Buy, 100.0);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[tokio::test]
        async fn test_place_spot_market_sell_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market("BTC-USDT", TdMode::Cash, Side::Sell, 0.0038);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[tokio::test]
        async fn test_place_spot_limit_buy_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit("BTC-USDT", TdMode::Cash, Side::Buy, 26000.0, 0.0038);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[tokio::test]
        async fn test_place_spot_limit_sell_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::limit("BTC-USDT", TdMode::Cash, Side::Sell, 26000.0, 0.0038);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        // todo fix needed
        // #[tokio::test]
        async fn test_place_margin_market_buy_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market("BTC-USDT", TdMode::Isolated, Side::Buy, 100.0);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[tokio::test]
        async fn test_place_margin_market_sell_order() {
            let rest_client = build_private_rest_client();
            let request = PlaceOrderRequest::market("BTC-USDT", TdMode::Isolated, Side::Sell, 0.0038);
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
        }

        #[tokio::test]
        async fn test_place_spot_limit_buy_order_with_sp() {
        }

        #[tokio::test]
        async fn test_place_spot_limit_buy_order_with_tp() {
        }

        #[tokio::test]
        async fn test_place_spot_limit_buy_order_with_tp_and_sp() {
        }

        #[tokio::test]
        async fn test_place_margin_market_buy_order_with_sp() {
        }

        #[tokio::test]
        async fn test_place_margin_market_buy_order_with_tp() {
        }

        #[tokio::test]
        async fn test_place_margin_market_buy_order_with_tp_and_sp() {
        }
    }

    mod websocket {
        use std::{env, thread};
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        use serde_json::from_value;
        use crate::enums::{InstType, Side, TdMode};
        use crate::okx::tests::rest::build_private_rest_client;

        use crate::rest::{MarkPriceResponse, OrderDetailsResponse, PlaceOrderRequest};
        use crate::websocket::{Channel, Command, Message, OkxWsClient};

        async fn build_private_ws_client<T: FnMut(Message) + Send + 'static>(callback: T) -> OkxWsClient {
            OkxWsClient::private(true, "wss://ws.okx.com:8443",
                                 &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                                 &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
                                 &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-PASSPHRASE").unwrap(), callback).await
        }

        async fn build_public_ws_client<T: FnMut(Message) + Send + 'static>(callback: T) -> OkxWsClient {
            OkxWsClient::public(true, "wss://ws.okx.com:8443", callback).await
        }

        #[tokio::test]
        async fn test_handle_mark_price() {
            let result = Arc::new(Mutex::new(Vec::new()));
            let handler = {
                let result = Arc::clone(&result);
                move |message: Message| {
                    match message {
                        Message::Data { arg, mut data, .. } => {
                            assert!(matches!(arg, Channel::MarkPrice { .. }));
                            let data = data.pop().unwrap();
                            let mark_price: MarkPriceResponse = from_value(data).unwrap();
                            dbg!(&mark_price);
                            result.lock()
                                .unwrap()
                                .push(mark_price)
                        }
                        Message::Event { .. } => {}
                        message => panic!("Unexpected message: '{message:?}'")
                    }
                }
            };

            let client = build_public_ws_client(handler).await;
            client.send(Command::subscribe(vec![Channel::MarkPrice {
                inst_id: "BTC-USDT".to_string(),
            }])).await;

            tokio::time::sleep(Duration::from_secs(1)).await;
            assert!(!result.lock().unwrap().is_empty());
        }

        #[tokio::test]
        async fn test_handle_order() {
            let result = Arc::new(Mutex::new(Vec::new()));
            let handler = {
                let result = Arc::clone(&result);
                move |message: Message| {
                    match message {
                        Message::Data { arg, mut data, .. } => {
                            assert!(matches!(arg, Channel::Orders { .. }));
                            let data = data.pop().unwrap();
                            let order: OrderDetailsResponse = from_value(data).unwrap();
                            dbg!(&order);
                            result.lock()
                                .unwrap()
                                .push(order)
                        }
                        Message::Event { .. } => {}
                        Message::Login { .. } => {}
                        message => panic!("Unexpected message: '{message:?}'")
                    }
                }
            };

            let client = build_private_ws_client(handler).await;
            client.send(Command::subscribe(vec![Channel::Orders {
                inst_type: InstType::Any,
                inst_id: None,
                uly: None,
            }])).await;

            tokio::time::sleep(Duration::from_secs(30)).await;
            assert!(result.lock().unwrap().is_empty());

            let rest_client = build_private_rest_client();
            let mut request = PlaceOrderRequest::market("BTC-USDT", TdMode::Cash, Side::Buy, 100.0);
            let order_id = "test";
            request.cl_ord_id = Some(order_id.to_string());
            let [response] = rest_client.request(request).await.unwrap();
            if response.s_code != 0 {
                dbg!(response);
                panic!("Error during order creation");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            let result = result.lock().unwrap();
            assert!(!result.is_empty());
            for order in result.iter() {
                assert_eq!(order.cl_ord_id, order_id);
            }
        }

        // #[tokio::test] // todo not implemented yet
        async fn test_drop_ws_client() {
            let result = Arc::new(Mutex::new(Vec::new()));
            let mut length = 0;
            {
                let handler = {
                    let result = Arc::clone(&result);
                    move |message: Message| {
                        match message {
                            Message::Data { arg, mut data, .. } => {
                                assert!(matches!(arg, Channel::MarkPrice { .. }));
                                let data = data.pop().unwrap();
                                let mark_price: MarkPriceResponse = from_value(data).unwrap();
                                dbg!(&mark_price);
                                result.lock()
                                    .unwrap()
                                    .push(mark_price)
                            }
                            Message::Event { .. } => {}
                            message => panic!("Unexpected message: '{message:?}'")
                        }
                    }
                };

                let client = build_public_ws_client(handler).await;
                client.send(Command::subscribe(vec![Channel::MarkPrice {
                    inst_id: "BTC-USDT".to_string(),
                }])).await;

                thread::sleep(Duration::from_secs(1));
                length = result.lock().unwrap().len();
                assert_ne!(length, 0);
            }
            dbg!("Drop client");
            thread::sleep(Duration::from_secs(1));
            assert_eq!(length, result.lock().unwrap().len());
        }
    }
}
