use std::sync::Arc;
use std::time::Duration;

use async_broadcast::{broadcast, Receiver, Sender};
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use tokio::{select, task};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};
use tungstenite::Message as WsMessage;
use url::Url;

use crate::okx::credential::Credential;
use crate::websocket::{Command, Message};
use crate::websocket::handler::WsMessageHandler;

pub struct OkxWsClient {
    sender: Sender<WsMessage>,
}

impl OkxWsClient {
    pub async fn public<H: WsMessageHandler>(demo: bool, ws_public_url: &str, handler: H) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/public");
        if demo {
            ws_public_url = format!("{ws_public_url}?brokerId=9999");
        }
        Self::new(&ws_public_url, None, handler).await
    }

    pub async fn business<H: WsMessageHandler>(demo: bool, ws_public_url: &str, handler: H) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/business");
        if demo {
            ws_public_url = "wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999".to_string(); // todo unhardcode host
        }
        Self::new(&ws_public_url, None, handler).await
    }

    pub async fn private<H: WsMessageHandler>(demo: bool, ws_private_url: &str, api_key: &str, api_secret: &str, passphrase: &str, handler: H) -> Self {
        let mut ws_private_url = format!("{ws_private_url}{}", "/ws/v5/private");
        if demo {
            ws_private_url = format!("{ws_private_url}?brokerId=9999");
        }
        Self::new(&ws_private_url, Some(Credential::new(api_key, api_secret, passphrase)), handler).await
    }

    async fn new<H: WsMessageHandler>(url: &str, credential: Option<Credential>, handler: H) -> Self {
        let url = Url::parse(url).unwrap();
        let sender = run(&url, credential, handler).await;
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

async fn run<H: WsMessageHandler>(url: &Url, credential: Option<Credential>, handler: H) -> Sender<WsMessage> {
    let (tx, rx) = broadcast::<WsMessage>(1);
    let handler = Arc::new(Mutex::new(handler));
    tokio::spawn({
        let url = url.clone();
        handling_loop(url, credential, rx, handler)
    });
    tx
}

async fn handling_loop<H: WsMessageHandler>(url: Url, credential: Option<Credential>, receiver: Receiver<WsMessage>, handler: Arc<Mutex<H>>) {
    loop {
        let handler = Arc::clone(&handler);
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
        let read_task = tokio::spawn(read(stream, handler));
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
        tokio::time::sleep(Duration::from_secs(2)).await; // todo remove this hotfix for login response & can lead to data loss in this timeout
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

async fn read<H: WsMessageHandler>(stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, handler: Arc<Mutex<H>>) {
    stream.for_each(|message| {
        let handler = Arc::clone(&handler);
        async move {
            match message {
                Ok(message) => match message {
                    WsMessage::Text(message) => {
                        trace!("Received text: {message:?}");
                        let message: Message = serde_json::from_str(&message.clone()).expect("Cannot deserialize message from OKX");
                        let mut handler = handler.lock().await;
                        handler.apply(message).await;
                    }
                    WsMessage::Pong(_) => { trace!("Received pong massage"); }
                    WsMessage::Close(message) => { warn!("Received close: {message:?}"); }
                    message => { warn!("Received unexpected message: {message:?}"); }
                }
                Err(error) => error!("Received error from socket: {error:?}"),
            }
        }
    }).await
}

fn login_message(credential: Credential) -> String {
    let login_command = Command::login(credential).unwrap();
    serde_json::to_string(&login_command).unwrap()
}
