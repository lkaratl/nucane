pub use error::{OkExError, Result};

mod credential;
pub mod enums;
mod error;
mod parser;
pub mod rest;
pub mod websocket;

#[cfg(test)]
mod tests {
    use std::sync::{Arc};
    use std::thread;
    use std::time::Duration;

    use tracing::{debug, Level};
    use tracing_subscriber::FmtSubscriber;
    use uuid::Uuid;
    use fehler::{throw, throws};
    use anyhow::Error;
    use serde_json::from_value;
    use futures::{SinkExt, StreamExt};
    use futures::executor::block_on;
    use tokio::sync::Mutex;
    use tracing::field::debug;

    use crate::okx::ws::model::{MarkPriceSub, Response};
    use crate::rest::MarkPriceResponse;
    use crate::websocket::{Channel, Command, Message, OkExWebsocket};
    use crate::websocket::models::Ticker;

    use super::*;

    fn init_logger() {
        let subscriber = FmtSubscriber::builder() // todo log only current app not libraries like hyper ...
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");
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
                    println!("{:?}", x);
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
