use anyhow::Result;
use async_nats::{Client, Subscriber};
use async_trait::async_trait;
use futures::StreamExt;
use tokio::task;
use tracing::{error, warn};

use crate::core::{Handler, MessageReceive, MessageSend, MessageSubject, RequestReceive, RequestSend, RequestSubject, SynapseReceive, SynapseSend};

// todo separate impls
pub struct NatsSender {
    client: Client,
}

impl NatsSender {
    pub async fn new(address: &str) -> NatsSender {
        Self {
            client: async_nats::connect(address).await.expect("Error during nats client creation")
        }
    }
}

#[async_trait]
impl MessageSend for NatsSender {
    async fn send_message<S: MessageSubject>(&self, subject: &S, message: &S::MessageType) -> Result<()> {
        let request_payload = serde_json::to_string(message)?;
        Ok(self.client.publish(subject.subject(), request_payload.into()).await?)
    }
}

#[async_trait]
impl RequestSend for NatsSender {
    async fn send_request<S: RequestSubject>(&self, subject: &S, message: &S::MessageType) -> Result<S::ResponseType> {
        let request_payload = serde_json::to_string(message)?;
        let response_massage = self.client.request(subject.subject(), request_payload.into()).await?;
        let response = serde_json::from_slice(&response_massage.payload)?;
        Ok(response)
    }
}

impl SynapseSend for NatsSender {}

pub struct NatsReceiver {
    client: Client,
}

impl NatsReceiver {
    pub async fn new(address: &str) -> NatsReceiver {
        Self {
            client: async_nats::connect(address).await.expect("Error during nats client creation")
        }
    }

    async fn new_subscriber(&self, subject: &str, group: Option<String>) -> Result<Subscriber> {
        let subscriber = if let Some(group) = group {
            self.client.queue_subscribe(subject.to_string(), group).await?
        } else {
            self.client.subscribe(subject.to_string()).await?
        };
        Ok(subscriber)
    }
}

#[async_trait]
impl MessageReceive for NatsReceiver {
    async fn handle_message<S: MessageSubject>(&self, subject: &S, group: Option<String>, handler: impl Handler<S::MessageType, ()>) -> Result<()> {
        let subject = subject.subject();
        let mut subscriber = self.new_subscriber(&subject, group).await?;
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::MessageType>(&message.payload) {
                    Ok(payload) => {
                        handler.handle(payload).await;
                    }
                    Err(error) => error!("Error during synapse message deserialization: '{}', for subject: '{}'", error, subject)
                }
            }
        });
        Ok(())
    }
}

#[async_trait]
impl RequestReceive for NatsReceiver {
    async fn handle_request<S: RequestSubject>(&self, subject: &S, group: Option<String>, handler: impl Handler<S::MessageType, S::ResponseType>) -> Result<()> {
        let client = self.client.clone();
        let subject = subject.subject();
        let mut subscriber = self.new_subscriber(&subject, group).await?;
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::MessageType>(&message.payload) {
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
}

impl SynapseReceive for NatsReceiver {}
