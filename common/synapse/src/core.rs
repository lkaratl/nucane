use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait MessageSubject: Send + Sync + 'static {
    type MessageType: Serialize + DeserializeOwned + Send + Sync;
    const SUBJECT: &'static str;
    fn subject(&self) -> String {
        Self::SUBJECT.to_string()
    }
}

pub trait RequestSubject: MessageSubject + Send + 'static {
    type ResponseType: Serialize + DeserializeOwned + Send + Sync;
}

#[async_trait]
pub trait MessageHandler<RequestType>: Send + Sync + 'static {
    async fn handle(&self, message: RequestType);
}

#[async_trait]
pub trait RequestHandler<RequestType, ResponseType>: Send + Sync + 'static {
    async fn handle(&self, message: RequestType) -> ResponseType;
}

#[async_trait]
pub trait MessageSend {
    async fn send_message<S: MessageSubject>(&self, subject: &S, message: &S::MessageType) -> Result<()>;
}

#[async_trait]
pub trait RequestSend {
    async fn send_request<S: RequestSubject>(&self, subject: &S, message: &S::MessageType) -> Result<S::ResponseType>;
}

#[async_trait]
pub trait MessageReceive {
    async fn handle_message<S: MessageSubject>(&self, subject: &S, group: Option<String>, handler: impl MessageHandler<S::MessageType>) -> Result<()>;
}

#[async_trait]
pub trait RequestReceive {
    async fn handle_request<S: RequestSubject>(&self, subject: &S, group: Option<String>, handler: impl RequestHandler<S::MessageType, S::ResponseType>) -> Result<()>;
}

pub trait SynapseSend: MessageSend + RequestSend {}

pub trait SynapseReceive: MessageReceive + RequestReceive {}
