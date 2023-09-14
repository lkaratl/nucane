use async_nats::{Client, Subscriber};
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::task;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{error, warn};

pub trait MessageSubject: Send + 'static {
    type Type: Serialize + DeserializeOwned + Send;
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

pub struct Synapse {
    client: Client,
}

impl Synapse {
    pub async fn new(address: &str) -> Synapse {
        Self {
            client: async_nats::connect(address).await.expect("Error during nats client creation")
        }
    }

    pub async fn new_subscriber(&self, subject: &str, group: Option<String>) -> Result<Subscriber> {
        let subscriber = if let Some(group) = group {
            self.client.queue_subscribe(subject.to_string(), group).await?
        } else {
            self.client.subscribe(subject.to_string()).await?
        };
        Ok(subscriber)
    }

    pub async fn on_message<S: MessageSubject>(&self, subject: &S, group: Option<String>, handler: impl MessageHandler<S::Type>) -> Result<()> {
        let subject = subject.subject();
        let mut subscriber = self.new_subscriber(&subject, group).await?;
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::Type>(&message.payload) {
                    Ok(payload) => {
                        handler.handle(payload).await;
                    }
                    Err(error) => error!("Error during synapse message deserialization: '{}', for subject: '{}'", error, subject)
                }
            }
        });
        Ok(())
    }

    pub async fn message<S: MessageSubject>(&self, subject: &S, message: &S::Type) -> Result<()> {
        let request_payload = serde_json::to_string(message)?;
        Ok(self.client.publish(subject.subject(), request_payload.into()).await?)
    }

    pub async fn on_request<S: RequestSubject>(&self, subject: &S, group: Option<String>, handler: impl RequestHandler<S::Type, S::ResponseType>) -> Result<()> {
        let client = self.client.clone();
        let subject = subject.subject();
        let mut subscriber = self.new_subscriber(&subject, group).await?;
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::Type>(&message.payload) {
                    Ok(payload) => {
                        let response = handler.handle(payload).await;
                        match serde_json::to_string(&response) {
                            Ok(response_payload) => {
                                if let Some(reply_subject) = message.reply {
                                    let _ = client.publish(reply_subject, response_payload.into())
                                        .await
                                        .map_err(|error| error!("Error during response send: '{}', for subject: '{}'", error, message.subject));
                                } else {
                                    warn!("Expected response for subject: '{}', but reply topic doesn't provided", message.subject);
                                }
                            }
                            Err(error) => error!("Error during synapse response serialization: '{}', for subject: '{}'", error, subject)
                        }
                    }
                    Err(error) => error!("Error during synapse message deserialization: '{}', for subject: '{}'", error, subject)
                }
            }
        });
        Ok(())
    }

    pub async fn request<S: RequestSubject>(&self, subject: &S, request_message: &S::Type) -> Result<S::ResponseType> {
        let request_payload = serde_json::to_string(request_message)?;
        let response_massage = self.client.request(subject.subject(), request_payload.into()).await?;
        let response = serde_json::from_slice(&response_massage.payload)?;
        Ok(response)
    }
}
