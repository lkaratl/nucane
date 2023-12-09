use async_trait::async_trait;
use serde_json::Value;
use tracing::error;

use crate::okx::websocket::{Action, Channel, Message};
use crate::okx::websocket::message::ErrorLiteral;

#[async_trait]
pub trait WsMessageHandler: Send + Sync + 'static {
    type Type: Send;

    async fn apply(&mut self, message: Message) {
        let typed_message = self.convert(message).await;
        if let Some(typed_message) = typed_message {
            self.handle(typed_message).await;
        }
    }
    async fn convert(&mut self, message: Message) -> Option<Self::Type> {
        match message {
            Message::Login { .. } => None,
            Message::Event { event, arg } => self.convert_event(event, arg).await,
            Message::Data { arg, action, data } =>
                self.convert_data(arg, action, data).await,
            Message::Error { event, code, msg } =>
                self.convert_error(event, code, msg).await,
            Message::Pong => None
        }
    }

    async fn convert_event(&mut self, _event: String, _arg: Channel) -> Option<Self::Type> {
        None
    }
    async fn convert_data(&mut self, arg: Channel, action: Option<Action>, data: Vec<Value>) -> Option<Self::Type>;
    async fn convert_error(&mut self, _event: ErrorLiteral, code: String, msg: String) -> Option<Self::Type> {
        error!("Error {}: {}", code, msg);
        None
    }
    async fn handle(&mut self, message: Self::Type);
}
