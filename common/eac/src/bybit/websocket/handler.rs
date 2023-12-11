use async_trait::async_trait;
use serde_json::Value;
use tracing::{error, info};

use crate::bybit::websocket::Message;

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
            Message::Auth { .. } => {
                info!("Retrieved Auth message");
                None
            }
            Message::Event { success, op, ret_msg, conn_id } => self.convert_event(success, op, ret_msg, conn_id).await,
            Message::Data { topic, data } => self.convert_data(topic, data).await,
            Message::Error { op, ret_msg, .. } => self.convert_error(op, ret_msg).await,
            Message::Pong { .. } => {
                info!("Retrieved Pong message");
                None
            }
        }
    }

    async fn convert_event(&mut self, _success: bool, _msg: String, _conn_id: String, _op: String) -> Option<Self::Type> {
        None
    }
    async fn convert_data(&mut self, topic: String, data: Value) -> Option<Self::Type>;
    async fn convert_error(&mut self, op: String, msg: String) -> Option<Self::Type> {
        error!("Error {}: {}", op, msg);
        None
    }
    async fn handle(&mut self, message: Self::Type);
}
