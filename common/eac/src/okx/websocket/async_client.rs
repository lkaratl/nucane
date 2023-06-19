use std::time::Duration;
use futures::{SinkExt, StreamExt, TryStreamExt};
use async_broadcast::{broadcast, Sender};
use tokio::{select, task};
use tokio_tungstenite::{connect_async};
use tracing::{error, info, trace, debug, warn};
use tungstenite::Message;
use url::Url;
use crate::okx::credential::Credential;
use crate::websocket::Command;

pub struct OkxWsClient {
    url: Url,
    sender: Sender<Message>,
}

impl OkxWsClient {
    pub async fn public(demo: bool, ws_public_url: &str) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/public");
        if demo {
            ws_public_url = format!("{ws_public_url}?brokerId=9999");
        }
        Self::new(&ws_public_url, None).await
    }

    pub async fn business(demo: bool, ws_public_url: &str) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/business");
        if demo {
            ws_public_url = "wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999".to_string(); // todo unhardcode host
        }
        Self::new(&ws_public_url, None).await
    }

    pub async fn private(demo: bool, ws_private_url: &str, api_key: &str, api_secret: &str, passphrase: &str) -> Self {
        let mut ws_private_url = format!("{ws_private_url}{}", "/ws/v5/private");
        if demo {
            ws_private_url = format!("{ws_private_url}?brokerId=9999");
        }
        Self::new(&ws_private_url, Some(Credential::new(api_key, api_secret, passphrase))).await
    }

    async fn new(url: &str, credential: Option<Credential>) -> Self {
        let url = Url::parse(url).unwrap();
        let sender = run(&url, credential).await;
        Self {
            url,
            sender,
        }
    }

    async fn send(&self, command: Command) {
        let command = match &command {
            &Command::Ping => "ping".to_string(), // todo maybe not needed
            command => serde_json::to_string(command).unwrap(),
        };
        debug!("Sending '{}' through sender channel", command);
        self.sender.broadcast(Message::Text(command)).await.unwrap();
    }
}

async fn run(url: &Url, credential: Option<Credential>) -> Sender<Message> {
    let (tx, rx) = broadcast::<Message>(1);
    tokio::spawn({
        let url = url.clone();
        let tx = tx.clone();
        async move {
            loop {
                let mut rx = rx.clone();

                let (ws_stream, _) = match connect_async(url.clone()).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Failed to connect: {:?}", e);
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
                        debug!("Send login command");
                        let _ = write.send(Message::Text(msg)).await;
                    }

                    loop {
                        let mut rx = rx.new_receiver();
                        let message_received = task::spawn(async move { rx.recv().await});
                        let ping_interval_finished = task::spawn(tokio::time::sleep(Duration::from_secs(25)));

                        select! {
                            message = message_received => {
                                if let Ok(Ok(msg)) = message {
                                    debug!("Send: {}", msg);
                                    let _ = write.send(msg).await;
                                }
                            }
                            _ = ping_interval_finished => {
                                debug!("Send ping");
                            let _ = write.send(Message::Ping(Vec::new())).await;
                            }
                        }
                    }
                });

                // Spawn the read task
                let read_task = tokio::spawn(async move {
                    read.for_each(|message| {
                        debug!("+");
                        match message.unwrap() {
                            Message::Text(message) => { debug!("Received text: {message:?}"); }
                            Message::Pong(_) => { debug!("Received pong massage"); }
                            Message::Close(message) => { warn!("Received close: {message:?}"); }
                            message => { warn!("Received unexpected message: {message:?}"); }
                        }
                        futures::future::ready(())
                    }).await;
                });

                // Wait for the read task to finish
                if let Err(e) = read_task.await {
                    error!("Lost connection: {:?}", e);
                } else {
                    error!("Closed without errors");
                }
                info!("Websocket reconnecting ...");
            }
        }
    });
    tokio::time::sleep(Duration::from_secs(1)).await; // todo remove this hotfix for login response
    tx
}

fn login_message(credential: Credential) -> String {
    let login_command = Command::login(credential).unwrap();
    serde_json::to_string(&login_command).unwrap()
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::time::Duration;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::SubscriberBuilder;
    use crate::enums::InstType;
    use crate::websocket::async_client::OkxWsClient;
    use crate::websocket::{Channel, Command};

    fn init_logger() {
        let subscriber = SubscriberBuilder::default()
            // todo move to config or init on standalone side
            .with_env_filter(EnvFilter::new("INFO,eac=DEBUG"))
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber).unwrap()
    }


    #[tokio::test]
    async fn listen_ticks() {
        init_logger();
        let client = OkxWsClient::public(false, "wss://ws.okx.com:8443").await;
        client.send(Command::subscribe(vec![Channel::MarkPrice {
            inst_id: "BTC-USDT".to_string(),
        }])).await;

        tokio::time::sleep(Duration::from_secs(10)).await
    }

    #[tokio::test]
    async fn listen_account() {
        init_logger();
        let client = OkxWsClient::private(true, "wss://ws.okx.com:8443",
                                          &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-KEY").unwrap(),
                                          &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-SECRET").unwrap(),
                                          &env::var("INTERACTOR_EAC_EXCHANGES_OKX_AUTH_API-PASSPHRASE").unwrap(),
        ).await;
        client.send(Command::subscribe(vec![Channel::Orders {
            inst_type: InstType::Any,
            inst_id: None,
            uly: None,
        }])).await;

        tokio::time::sleep(Duration::from_secs(90)).await
    }
}
