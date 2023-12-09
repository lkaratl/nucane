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

use crate::bybit::credential::Credential;
use crate::bybit::websocket::{Command, Message};
use crate::bybit::websocket::handler::WsMessageHandler;

pub struct BybitWsClient {
    sender: Sender<WsMessage>,
}

impl BybitWsClient {
    pub async fn public<H: WsMessageHandler>(ws_public_url: &str, handler: H) -> Self {
        let ws_public_url = format!("{ws_public_url}{}", "/v5/public/spot");
        Self::new(&ws_public_url, None, handler).await
    }

    pub async fn private<H: WsMessageHandler>(
        ws_private_url: &str,
        api_key: &str,
        api_secret: &str,
        handler: H,
    ) -> Self {
        let ws_private_url = format!("{ws_private_url}{}", "/v5/private");
        Self::new(
            &ws_private_url,
            Some(Credential::new(api_key, api_secret)),
            handler,
        ).await
    }

    async fn new<H: WsMessageHandler>(
        url: &str,
        credential: Option<Credential>,
        handler: H,
    ) -> Self {
        let url = Url::parse(url).unwrap();
        let sender = run(&url, credential, handler).await;
        Self { sender }
    }

    pub async fn send(&self, command: Command) {
        let command = serde_json::to_string(&command).unwrap();
        debug!("Sending '{}' through sender channel", command);
        self.sender
            .broadcast(WsMessage::Text(command))
            .await
            .unwrap();
    }
}

async fn run<H: WsMessageHandler>(
    url: &Url,
    credential: Option<Credential>,
    handler: H,
) -> Sender<WsMessage> {
    let (tx, rx) = broadcast::<WsMessage>(1);
    let handler = Arc::new(Mutex::new(handler));
    tokio::spawn({
        let url = url.clone();
        handling_loop(url, credential, rx, handler)
    });
    tx
}

async fn handling_loop<H: WsMessageHandler>(
    url: Url,
    credential: Option<Credential>,
    receiver: Receiver<WsMessage>,
    handler: Arc<Mutex<H>>,
) {
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

async fn write(
    receiver: Receiver<WsMessage>,
    mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>,
    credential: Option<Credential>,
) {
    if let Some(credential) = credential {
        let msg = login_message(credential);
        info!("Send login command to websocket");
        sink.send(WsMessage::Text(msg)).await.unwrap();
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
                    sink.send(msg).await.unwrap();
                }
            }
            _ = ping_interval_finished => {
                trace!("Send ping websocket");
                sink.send(WsMessage::Ping(Vec::new())).await.unwrap();
            }
        }
    }
}

async fn read<H: WsMessageHandler>(
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    handler: Arc<Mutex<H>>,
) {
    stream
        .for_each(|message| {
            let handler = Arc::clone(&handler);
            async move {
                match message {
                    Ok(message) => match message {
                        WsMessage::Text(message) => {
                            info!("Received text: {message:?}");
                            let message: Message = serde_json::from_str(&message.clone())
                                .expect("Cannot deserialize message from BYBIT");
                            let mut handler = handler.lock().await;
                            handler.apply(message).await;
                        }
                        WsMessage::Pong(_) => {
                            trace!("Received pong massage");
                        }
                        WsMessage::Close(message) => {
                            warn!("Received close: {message:?}");
                        }
                        message => {
                            warn!("Received unexpected message: {message:?}");
                        }
                    },
                    Err(error) => error!("Received error from socket: {error:?}"),
                }
            }
        })
        .await
}

fn login_message(credential: Credential) -> String {
    let login_command = Command::login(credential).unwrap();
    serde_json::to_string(&login_command).unwrap()
}
