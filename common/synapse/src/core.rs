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
pub trait Handler<RequestType, ResponseType>: Send + Sync + 'static {
    async fn handle(&self, message: RequestType) -> ResponseType;
}

#[async_trait]
pub trait SynapseSend: Send + Sync {
    async fn send_message<S: MessageSubject>(&self, subject: S, message: &S::MessageType) -> Result<()>;
    async fn send_request<S: RequestSubject>(&self, subject: S, message: &S::MessageType) -> Result<S::ResponseType>;
}

#[async_trait]
pub trait SynapseReceive: Send {
    async fn handle_message<S: MessageSubject>(&self, subject: S, group: Option<String>, handler: impl Handler<S::MessageType, ()>) -> Result<()>;
    async fn handle_request<S: RequestSubject>(&self, subject: S, group: Option<String>, handler: impl Handler<S::MessageType, S::ResponseType>) -> Result<()>;
}
