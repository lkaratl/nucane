use std::future::Future;
use async_nats::Client;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::task;
use anyhow::Result;
use tracing::{error, warn};

pub trait MessageSubject {
    type Type: Serialize + DeserializeOwned + Send;
    fn subject(&self) -> String;
}

pub trait RequestSubject: MessageSubject {
    type ResponseType: Serialize + DeserializeOwned + Send + Sync;
}

pub struct SynapseClient {
    client: Client,
}

impl SynapseClient {
    pub async fn new(address: &str) -> SynapseClient {
        Self {
            client: async_nats::connect(address).await.expect("Error during nats client creation")
        }
    }

    pub async fn on_message<S: MessageSubject + Send + 'static, H: FnMut(S::Type) -> F + Send + 'static, F: Future<Output=()> + Send + 'static>(&self, subject: &S, group: Option<String>, mut handler: H) -> Result<()> {
        let subject = subject.subject();
        let mut subscriber = if let Some(group) = group {
            self.client.queue_subscribe(subject.clone(), group).await?
        } else {
            self.client.subscribe(subject.clone()).await?
        };
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::Type>(&message.payload) {
                    Ok(payload) => {
                        handler(payload).await;
                    }
                    Err(error) => error!("Error during synapse message deserialization: '{}', for subject: '{}'", error, subject)
                }
            }
        });
        Ok(())
    }

    pub async fn message<S: MessageSubject + Send + 'static>(&self, subject: &S, message: &S::Type) -> Result<()> {
        let request_payload = serde_json::to_string(message)?;
        Ok(self.client.publish(subject.subject(), request_payload.into()).await?)
    }

    pub async fn on_request<S: RequestSubject + Send + 'static, H: FnMut(S::Type) -> F + Send + 'static, F: Future<Output=S::ResponseType> + Send + 'static>(&self, subject: &S, group: Option<String>, mut handler: H) -> Result<()> {
        let client = self.client.clone();
        let subject = subject.subject();
        let mut subscriber = if let Some(group) = group {
            self.client.queue_subscribe(subject.clone(), group).await?
        } else {
            self.client.subscribe(subject.clone()).await?
        };
        task::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<S::Type>(&message.payload) {
                    Ok(payload) => {
                        let response = handler(payload).await;
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

    pub async fn request<S: RequestSubject + Send + 'static>(&self, subject: &S, request_message: &S::Type) -> Result<S::ResponseType> {
        let request_payload = serde_json::to_string(request_message)?;
        let response_massage = self.client.request(subject.subject(), request_payload.into()).await?;
        let response = serde_json::from_slice(&response_massage.payload)?;
        Ok(response)
    }
}
