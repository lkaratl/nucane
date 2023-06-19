pub use error::{OkExError, Result};

mod credential;
pub mod enums;
mod error;
mod parser;
pub mod rest;
pub mod websocket;

#[cfg(test)]
mod tests {
    use std::{env, thread};
    use std::time::Duration;

    use tracing::{debug, error};
    use fehler::throws;
    use anyhow::Error;
    use serde_json::from_value;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::SubscriberBuilder;
    use crate::enums::{InstType, Side, TdMode};

    use crate::rest::{MarkPriceResponse, OkExRest, OrderDetailsResponse, PlaceOrderRequest};
    use crate::websocket::{Channel, Command, Message, OkExWebsocket};
    use crate::websocket::models::Ticker;

    fn init_logger() {
        let subscriber = SubscriberBuilder::default()
            .with_env_filter(EnvFilter::new("INFO,eac=DEBUG"))
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");
    }

    fn build_rest_client() -> OkExRest {
        OkExRest::with_credential("https://www.okx.com", true,
                                  &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                                  &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
                                  &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-PASSPHRASE").unwrap())
    }

    fn build_private_ws_client() -> OkExWebsocket {
        OkExWebsocket::private(true, "wss://ws.okx.com:8443",
                               &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                               &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
                               &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-PASSPHRASE").unwrap()).unwrap()
    }

    #[tokio::test]
    async fn test_place_spot_market_buy_order() {
        let rest_client = build_rest_client();
        let request = PlaceOrderRequest::market("BTC-USDT", TdMode::Cash, Side::Buy, 100.0);
        let [response] = rest_client.request(request).await.unwrap();
        if response.s_code != 0 {
            dbg!(response);
            panic!("Error during order creation");
        }
    }

    #[tokio::test]
    async fn test_place_spot_limit_buy_order() {
        let rest_client = build_rest_client();
        let request = PlaceOrderRequest::limit("BTC-USDT", TdMode::Cash, Side::Buy, 26000.0, 100.0);
        let [response] = rest_client.request(request).await.unwrap();
        if response.s_code != 0 {
            dbg!(response);
            panic!("Error during order creation");
        }
    }

    #[throws(Error)]
    #[tokio::test]
    async fn test_handle_tickers() {
        let mut client = OkExWebsocket::public(true, "wss://ws.okx.com:8443")?;
        client.send(Command::subscribe(vec![Channel::Tickers {
            inst_id: "BTC-USDT".to_string(),
        }]))?;

        client.handle_message(|message| {
            match message.unwrap() {
                Message::Data { arg, mut data, .. } => {
                    assert!(matches!(arg, Channel::Tickers { .. }));
                    let data = data.pop().unwrap();
                    let ticker: Ticker = from_value(data).unwrap();
                    println!("{:?}", ticker);
                }
                Message::Error { code, msg, .. } => {
                    println!("Error {}: {}", code, msg);
                }
                Message::Event { .. } => {}
                _ => unreachable!(),
            }
        });
        thread::sleep(Duration::from_secs(5));
    }


    #[tokio::test]
    async fn test_handle_orders() {
        init_logger();
        let mut client = build_private_ws_client();
        client.send(Command::subscribe(vec![Channel::Orders {
            inst_type: InstType::Any,
            inst_id: None,
            uly: None,
        }])).unwrap();

        client.handle_message(|message| {
            match message.unwrap() {
                Message::Data { arg, mut data, .. } => {
                    assert!(matches!(arg, Channel::Orders { .. }));
                    let data = data.pop().unwrap();
                    let order: OrderDetailsResponse = from_value(data).unwrap();
                    println!("{:?}", order);
                }
                message => println!("Unexpected message: '{message:?}'"),
            }
        });
        thread::sleep(Duration::from_secs(90));
    }

    #[throws(Error)]
    #[tokio::test]
    async fn test_handle_mark_price() {
        let mut client = OkExWebsocket::public(true, "wss://ws.okx.com:8443")?;
        client.send(Command::subscribe(vec![Channel::MarkPrice {
            inst_id: "BTC-USDT".to_string(),
        }]))?;

        client.handle_message(|message| {
            match message.unwrap() {
                Message::Data { arg, mut data, .. } => {
                    assert!(matches!(arg, Channel::MarkPrice { .. }));
                    let data = data.pop().unwrap();
                    let x: MarkPriceResponse = from_value(data).unwrap();
                    debug!("{:?}", x);
                }
                Message::Error { code, msg, .. } => {
                    error!("Error {}: {}", code, msg);
                }
                Message::Event { .. } => {}
                _ => unreachable!(),
            }
        });
        thread::sleep(Duration::from_secs(5));
    }

    #[throws(Error)]
    #[tokio::test]
    async fn test_subscribe_after_handling_start() {
        let mut client = OkExWebsocket::public(true, "wss://ws.okx.com:8443")?;
        client.send(Command::subscribe(vec![Channel::Tickers {
            inst_id: "BTC-USDT".to_string(),
        }]))?;

        client.handle_message(|message| {
            match message.unwrap() {
                Message::Data { arg, mut data, .. } => {
                    match arg {
                        Channel::Tickers { .. } => {
                            let data = data.pop().unwrap();
                            let ticker: Ticker = from_value(data).unwrap();
                            println!("+ {:?}", ticker);
                        }
                        Channel::MarkPrice { .. } => {
                            let data = data.pop().unwrap();
                            let x: MarkPriceResponse = from_value(data).unwrap();
                            println!("- {:?}", x);
                        }
                        _ => debug!("Retrieved message from unhandled channel")
                    }
                }
                Message::Error { code, msg, .. } => {
                    println!("Error {}: {}", code, msg);
                }
                Message::Event { .. } => {}
                _ => unreachable!(),
            }
        });
        thread::sleep(Duration::from_secs(5));
        dbg!("Subscribe on tickers");

        client.send(Command::subscribe(vec![Channel::MarkPrice {
            inst_id: "BTC-USDT".to_string(),
        }]))?;
        thread::sleep(Duration::from_secs(5));
    }

    #[throws(Error)]
    #[tokio::test]
    async fn test_drop_ws_client() {
        {
            let mut client = OkExWebsocket::public(true, "wss://ws.okx.com:8443")?;
            client.send(Command::subscribe(vec![Channel::MarkPrice {
                inst_id: "BTC-USDT".to_string(),
            }]))?;

            client.handle_message(|message| {
                match message.unwrap() {
                    Message::Data { arg, mut data, .. } => {
                        assert!(matches!(arg, Channel::MarkPrice { .. }));
                        let data = data.pop().unwrap();
                        let x: MarkPriceResponse = from_value(data).unwrap();
                        println!("{:?}", x);
                    }
                    Message::Error { code, msg, .. } => {
                        println!("Error {}: {}", code, msg);
                    }
                    Message::Event { .. } => {}
                    _ => unreachable!(),
                }
            });
            thread::sleep(Duration::from_secs(2));
        }
        dbg!("Drop client");
        thread::sleep(Duration::from_secs(3));
    }
}
