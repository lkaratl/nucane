use std::net::TcpStream;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::SyncSender;
use std::{mem, thread};
use std::time::Duration;

use fehler::{throw, throws};
use serde_json::{from_str, to_string};
pub use serde_json::Value;
use tracing::{debug, error, info, trace, warn};
use tungstenite::{connect, WebSocket};
use tungstenite::protocol::Message as WSMessage;
use tungstenite::stream::MaybeTlsStream;
use url::Url;

use crate::okx::credential::Credential;
use crate::okx::error::OkExError;

pub use self::channel::Channel;
pub use self::command::Command;
pub use self::message::{Action, Message};

mod channel;
mod command;
mod message;
pub mod models;

pub struct OkExWebsocket {
    url: Url,
    credential: Option<Credential>,
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
    sender: Option<SyncSender<WSHandlerEvents>>,
    private: bool,
    sent_commands: Vec<Command>,
}

impl OkExWebsocket {
    #[throws(OkExError)]
    pub fn public(demo: bool, ws_public_url: &str) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/public");
        if demo {
            ws_public_url = format!("{ws_public_url}?brokerId=9999");
        }
        Self::new_impl(&ws_public_url, false, None)?
    }

    #[throws(OkExError)]
    pub fn business(demo: bool, ws_public_url: &str) -> Self {
        let mut ws_public_url = format!("{ws_public_url}{}", "/ws/v5/business");
        if demo {
            ws_public_url = "wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999".to_string(); // todo unhardcode host
        }
        Self::new_impl(&ws_public_url, false, None)?
    }

    #[throws(OkExError)]
    pub fn private(demo: bool, ws_private_url: &str, api_key: &str, api_secret: &str, passphrase: &str) -> Self {
        let mut ws_private_url = format!("{ws_private_url}{}", "/ws/v5/private");
        if demo {
            ws_private_url = format!("{ws_private_url}?brokerId=9999");
        }
        Self::new_impl(&ws_private_url, true, Some(Credential::new(api_key, api_secret, passphrase)))?
    }

    #[throws(OkExError)]
    fn new_impl(url: &str, private: bool, credential: Option<Credential>) -> Self {
        let url = Url::parse(url).unwrap();
        let socket = open_connection(url.clone()).unwrap();
        let websocket = Self {
            url,
            credential,
            socket: Arc::new(Mutex::new(socket)),
            sender: None,
            private,
            sent_commands: Vec::new(),
        };
        websocket.login()?
    }

    #[throws(OkExError)]
    fn login(self) -> Self {
        if self.private {
            let mut socket = self.socket
                .lock()
                .expect("Socket already blocked. Probably by handle_message method");
            let cred = self.credential.clone().unwrap();
            login_command(&mut socket, cred)?;
        }
        self
    }

    #[throws(OkExError)]
    pub fn send(&mut self, command: Command) {
        self.sent_commands.push(command.clone());
        if let Some(sender) = &self.sender {
            sender.send(WSHandlerEvents::Hold)
                .expect("Error during sending Hold event for massage handler");
        }
        let mut socket = self.socket
            .lock()
            .expect("Socket already blocked. Probably by handle_message method");
        send_command(&mut socket, command)?
    }

    pub fn handle_message<T: FnMut(Result<Message, OkExError>) + Send + 'static>(&mut self, mut callback: T) {
        let (ts, tr) = mpsc::sync_channel::<WSHandlerEvents>(1);
        self.sender = Some(ts);
        thread::spawn({
            let url = self.url.clone();
            let socket = Arc::clone(&self.socket);
            let sent_commands = self.sent_commands.clone(); // todo need fix: 1.subscribe 2.start handle 3.new subscribe 4.reconnect not consider sub after handling start
            let private = self.private;
            let cred = self.credential.clone();
            move || {
                loop {
                    if let Ok(event) = tr.try_recv() {
                        match event {
                            WSHandlerEvents::Hold => thread::sleep(Duration::from_millis(100)),
                            WSHandlerEvents::Stop => {
                                info!("Drop OKX websocket");
                                return;
                            }
                        }
                    }

                    let mut socket = socket
                        .as_ref()
                        .lock()
                        .expect("Socket already blocked. Probably by send method");
                    match socket.read_message() {
                        Ok(message) => match parse_message(message) {
                            Ok(message) => callback(Ok(message)),
                            Err(error) => {
                                match error {
                                    OkExError::WebsocketClosed(frame) => {
                                        if let Some(frame) = frame {
                                            warn!("Websocket closed with frame: {frame:?}");
                                        } else {
                                            warn!("Websocket closed with empty frame");
                                        }
                                        reconnect(&mut socket, url.clone(), private, cred.clone(), &sent_commands);
                                    }
                                    _ => { callback(Err(error)); }
                                }
                            }
                        },
                        Err(error) => {
                            error!("Error during message read from socket: {error}");
                            reconnect(&mut socket, url.clone(), private, cred.clone(), &sent_commands);
                        }
                    }
                }
            }
        });
    }

    fn stop(&mut self) {
        if let Some(sender) = &self.sender {
            sender.send(WSHandlerEvents::Stop)
                .expect("Error during massage handler shutdown");
        }
    }
}

#[throws(OkExError)]
fn login_command(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, cred: Credential) {
    debug!("Login to OKX websocket");
    let login_command = Command::login(cred).unwrap();
    send_command(socket, login_command)?;
    thread::sleep(Duration::from_secs(1)); // todo remove this shit
}

#[throws(OkExError)]
fn send_command(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, command: Command) {
    let command = match &command {
        &Command::Ping => "ping".to_string(),
        command => to_string(command)?,
    };
    trace!("Sending '{}' through websocket", command);
    socket.write_message(WSMessage::Text(command))?
}

#[throws(OkExError)]
fn open_connection(url: Url) -> WebSocket<MaybeTlsStream<TcpStream>> {
    let (socket, _) = connect(url)?;
    socket
}

fn reconnect(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, url: Url, private: bool, cred: Option<Credential>, sent_commands: &Vec<Command>) {
    info!("Reconnect OKX websocket");
    let _ = mem::replace(socket, open_connection(url).unwrap());
    if private {
        login_command(socket, cred.unwrap()).unwrap();
    }
    for command in sent_commands {
        send_command(socket, command.clone()).unwrap();
    }
}

impl Drop for OkExWebsocket {
    fn drop(&mut self) { self.stop() }
}

enum WSHandlerEvents {
    Hold,
    Stop,
}

#[throws(OkExError)]
fn parse_message(msg: WSMessage) -> Message {
    trace!("WS raw message: {msg:?}");
    match msg {
        WSMessage::Text(message) => match message.as_str() {
            "pong" => Message::Pong,
            others => match from_str(others) {
                Ok(r) => r,
                Err(_) => unreachable!("Cannot deserialize message from OkEx: '{}'", others),
            },
        },
        WSMessage::Close(frame) => throw!(OkExError::WebsocketClosed(frame)),
        WSMessage::Binary(_) => throw!(OkExError::UnexpectedWebsocketBinaryMessage),
        WSMessage::Ping(_) => throw!(OkExError::UnexpectedWebsocketPingMessage),
        WSMessage::Pong(_) => throw!(OkExError::UnexpectedWebsocketPongMessage),
        _ => throw!(OkExError::UnexpectedWebsocketMessage),
    }
}
