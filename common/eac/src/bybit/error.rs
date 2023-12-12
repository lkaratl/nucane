use thiserror::Error;
use tungstenite::protocol::CloseFrame;

pub type Result<T> = std::result::Result<T, BybitError>;

#[derive(Debug, Error)]
pub enum BybitError {
    #[error("Cannot deserialize response from {0}")]
    CannotDeserializeResponse(String),

    #[error("No Api key set for private api")]
    NoApiKeySet,

    #[error("Unexpected websocket binary message")]
    UnexpectedWebsocketBinaryMessage,

    #[error("Unexpected websocket ping message")]
    UnexpectedWebsocketPingMessage,

    #[error("Unexpected websocket pong message")]
    UnexpectedWebsocketPongMessage,

    #[error("Unexpected websocket message")]
    UnexpectedWebsocketMessage,

    #[error("Websocket closed")]
    WebsocketClosed(Option<CloseFrame<'static>>),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),

    #[error(transparent)]
    JsonParse(#[from] serde_json::Error),

    #[error(transparent)]
    HttpRequest(#[from] reqwest::Error),

    #[error(transparent)]
    Websocket(#[from] tungstenite::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
