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
use async_broadcast::{broadcast, Receiver, Sender};
use futures::stream::{SplitSink, SplitStream};
use tokio::{select, task};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, trace, debug, warn, info};
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
        let sender = run(&url, credential, callback).await;
        Self {
            sender,
        }
    }

    pub async fn send(&self, command: Command) {
        let command = serde_json::to_string(&command).unwrap();
        debug!("Sending '{}' through sender channel", command);
        self.sender.broadcast(WsMessage::Text(command)).await.unwrap();
    }
}

async fn run<T: FnMut(Message) + Send + 'static>(url: &Url, credential: Option<Credential>, callback: T) -> Sender<WsMessage> {
    let (tx, rx) = broadcast::<WsMessage>(1);
    let callback = Arc::new(Mutex::new(callback));
    tokio::spawn({
        let url = url.clone();
        handling_loop(url, credential, rx, callback)
    });
    tx
}

async fn handling_loop<T: FnMut(Message) + Send + 'static>(url: Url, credential: Option<Credential>, receiver: Receiver<WsMessage>, callback: Arc<Mutex<T>>) {
    loop {
        let callback = Arc::clone(&callback);
        let rx = receiver.clone();
        info!("Connecting to websocket");
        let (ws_stream, _) = match connect_async(url.clone()).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Websocket failed to connect: {:?}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                warn!("Websocket reconnecting ...");
                continue;
            }
        };

        let (sink, stream) = ws_stream.split();
        let credential = credential.clone();
        let read_task = tokio::spawn(read(stream, callback));
        let write_task = tokio::spawn(write(rx, sink, credential));

        if let Err(e) = read_task.await {
            error!("Websocket lost connection: {:?}", e);
        } else {
            warn!("Websocket closed without errors");
        }
        write_task.abort();
        warn!("Websocket reconnecting ...");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn write(receiver: Receiver<WsMessage>, mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>, credential: Option<Credential>) {
    if let Some(credential) = credential {
        let msg = login_message(credential);
        info!("Send login command to websocket");
        let _ = sink.send(WsMessage::Text(msg)).await;
        tokio::time::sleep(Duration::from_secs(2)).await; // todo remove this hotfix for login response
    }

    let rx = Arc::new(Mutex::new(receiver));
    loop {
        let rx = Arc::clone(&rx);
        let message_received = task::spawn(async move { rx.lock().await.recv().await });
        let ping_interval_finished = task::spawn(tokio::time::sleep(Duration::from_secs(25)));

        select! {
            message = message_received => {
                if let Ok(Ok(msg)) = message {
                    info!("Send to websocket: {}", msg);
                    let _ = sink.send(msg).await;
                }
            }
            _ = ping_interval_finished => {
                trace!("Send ping websocket");
                let _ = sink.send(WsMessage::Ping(Vec::new())).await;
            }
        }
    }
}

async fn read<T: FnMut(Message) + Send + 'static>(stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, callback: Arc<Mutex<T>>) {
    let mut callback = callback.lock().await;
    stream.for_each(|message| {
        match message {
            Ok(message) => match message {
                WsMessage::Text(message) => {
                    trace!("Received text: {message:?}");
                    let message = serde_json::from_str(&message).expect("Cannot deserialize message from OKX");
                    callback(message);
                }
                WsMessage::Pong(_) => { trace!("Received pong massage"); }
                WsMessage::Close(message) => { warn!("Received close: {message:?}"); }
                message => { warn!("Received unexpected message: {message:?}"); }
            }
            Err(error) => error!("Received error from socket: {error:?}"),
        }
        futures::future::ready(())
    }).await
}

fn login_message(credential: Credential) -> String {
    let login_command = Command::login(credential).unwrap();
    serde_json::to_string(&login_command).unwrap()
}
