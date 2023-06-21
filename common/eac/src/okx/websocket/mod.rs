pub use self::channel::Channel;
pub use self::command::Command;
pub use self::message::{Action, Message};

mod channel;
mod command;
mod message;
pub mod models;

use std::sync::Arc;
use std::time::Duration;
use futures::{SinkExt, StreamExt};
use async_broadcast::{broadcast, Sender};
use tokio::{select, task};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async};
use tracing::{error, info, trace, debug, warn};
use tungstenite::Message as WsMessage;
use url::Url;
use crate::okx::credential::Credential;

pub struct OkxWsClient {
    sender: Sender<WsMessage>,
}

impl OkxWsClient {
    pub async fn public<T: FnMut(Message) + Send + 'static>(demo: bool, ws_public_url: &str, callback: T) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/public");
        if demo {
            ws_public_url = format!("{ws_public_url}?brokerId=9999");
        }
        Self::new(&ws_public_url, None, callback).await
    }

    pub async fn business<T: FnMut(Message) + Send + 'static>(demo: bool, ws_public_url: &str, callback: T) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/business");
        if demo {
            ws_public_url = "wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999".to_string(); // todo unhardcode host
        }
        Self::new(&ws_public_url, None, callback).await
    }

    pub async fn private<T: FnMut(Message) + Send + 'static>(demo: bool, ws_private_url: &str, api_key: &str, api_secret: &str, passphrase: &str, callback: T) -> Self {
        let mut ws_private_url = format!("{ws_private_url}{}", "/ws/v5/private");
        if demo {
            ws_private_url = format!("{ws_private_url}?brokerId=9999");
        }
        Self::new(&ws_private_url, Some(Credential::new(api_key, api_secret, passphrase)), callback).await
    }

    async fn new<T: FnMut(Message) + Send + 'static>(url: &str, credential: Option<Credential>, callback: T) -> Self {
        let url = Url::parse(url).unwrap();
        let (_, sender) = run(&url, credential, callback).await;
        Self {
            sender,
        }
    }

    pub async fn send(&self, command: Command) {
        let command = match &command {
            &Command::Ping => "ping".to_string(), // todo maybe not needed
            command => serde_json::to_string(command).unwrap(),
        };
        debug!("Sending '{}' through sender channel", command);
        self.sender.broadcast(WsMessage::Text(command)).await.unwrap();
    }
}

async fn run<T: FnMut(Message) + Send + 'static>(url: &Url, credential: Option<Credential>, callback: T) -> (JoinHandle<()>, Sender<WsMessage>) {
    let (tx, rx) = broadcast::<WsMessage>(1);
    let callback = Arc::new(Mutex::new(callback));
    let handler_handle = tokio::spawn({
        let url = url.clone();
        async move {
            loop {
                let callback = Arc::clone(&callback);
                let rx = rx.clone();

                let (ws_stream, _) = match connect_async(url.clone()).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Websocket failed to connect: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let (mut write, read) = ws_stream.split();

                // Spawn the write task
                let credential = credential.clone();
                tokio::spawn(async move {
                    if let Some(credential) = credential {
                        let msg = login_message(credential);
                        debug!("Send login command to websocket");
                        let _ = write.send(WsMessage::Text(msg)).await;
                    }

                    loop {
                        let mut rx = rx.new_receiver();
                        let message_received = task::spawn(async move { rx.recv().await });
                        let ping_interval_finished = task::spawn(tokio::time::sleep(Duration::from_secs(25)));

                        select! {
                            message = message_received => {
                                if let Ok(Ok(msg)) = message {
                                    debug!("Send to websocket: {}", msg);
                                    let _ = write.send(msg).await;
                                }
                            }
                            _ = ping_interval_finished => {
                                debug!("Send ping websocket");
                                let _ = write.send(WsMessage::Ping(Vec::new())).await;
                            }
                        }
                    }
                });

                // Spawn the read task
                let read_task = tokio::spawn(async move {
                    let mut callback = callback.lock().await;
                    read.for_each(|message| {
                        match message.unwrap() {
                            WsMessage::Text(message) => {
                                trace!("Received text: {message:?}");
                                let message = match serde_json::from_str(&message) {
                                    Ok(message) => message,
                                    Err(err) => unreachable!("Cannot deserialize message from OKX: '{message}', error: '{err}'", ),
                                };
                                callback(message);
                            }
                            WsMessage::Pong(_) => { debug!("Received pong massage"); }
                            WsMessage::Close(message) => { warn!("Received close: {message:?}"); }
                            message => { warn!("Received unexpected message: {message:?}"); }
                        }
                        futures::future::ready(())
                    }).await;
                });

                // Wait for the read task to finish
                if let Err(e) = read_task.await {
                    error!("Websocket lost connection: {:?}", e);
                } else {
                    error!("Websocket closed without errors");
                }
                info!("Websocket reconnecting ...");
            }
        }
    });
    tokio::time::sleep(Duration::from_secs(1)).await; // todo remove this hotfix for login response
    (handler_handle, tx)
}

fn login_message(credential: Credential) -> String {
    let login_command = Command::login(credential).unwrap();
    serde_json::to_string(&login_command).unwrap()
}
