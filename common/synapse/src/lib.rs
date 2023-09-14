mod core;
mod api;

use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use async_trait::async_trait;

use futures::executor::block_on;
use rdkafka::{ClientConfig, Message};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::task;
use tracing::{error, trace, warn};
use uuid::Uuid;

// todo extensible on client side
pub enum Topic {
    Tick,
    Action,
    Deployment,
    Order,
    Position,
    Candle,
    Simulation,
    Plugin,
}

impl Topic {
    fn get_topic(&self) -> String {
        match self {
            Topic::Tick => "nucane.tick".to_string(),
            Topic::Action => "nucane.action".to_string(),
            Topic::Deployment => "nucane.deployment".to_string(),
            Topic::Order => "nucane.order".to_string(),
            Topic::Position => "nucane.position".to_string(),
            Topic::Candle => "nucane.candle".to_string(),
            Topic::Simulation => "nucane.simulation".to_string(),
            Topic::Plugin => "nucane.plugin".to_string(),
        }
    }
}

pub trait SynapseSend<T: Synapse> {
    fn send(&self, data: &T);
    fn send_to(&self, producer: &FutureProducer, data: &T) {
        let topic = &data.topic().get_topic();
        let message_key = &Uuid::new_v4().to_string();
        trace!("Produce new data: {data:?} to topic: {topic} with key: {message_key}");
        let message = serde_json::to_string(&data).unwrap();
        let result = block_on(producer.send(
            FutureRecord::to(topic)
                .payload(&message)
                .key(message_key), Duration::from_secs(0)));

        match result {
            Ok(_) => trace!("Data successfully sent"),
            Err(error) => error!("Error during data sending: {:?}", error)
        }
    }
}

//todo derive & rename
pub trait Synapse: Serialize + DeserializeOwned + Debug {
    fn topic(&self) -> Topic;
}

#[async_trait]
pub trait SynapseListen<T: Synapse> {
    async fn on<F: Future<Output=()> + Send + 'static, C: FnMut(T) -> F + Send + 'static>(self, topic: Topic, callback: C);
}

pub struct Reader {
    consumer: StreamConsumer,
}

impl Reader {
    fn new(bootstrap_server: &str, group_id: &str) -> Self {
        let consumer: StreamConsumer<_> = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", bootstrap_server)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Consumer creation failed");
        Self { consumer }
    }
}

#[async_trait]
impl<T: Synapse + Send> SynapseListen<T> for Reader {
    async fn on<F: Future<Output=()> + Send + 'static, C: FnMut(T) -> F + Send + 'static>(self, topic: Topic, mut callback: C) {
        let consumer = self.consumer;
        let topic = topic.get_topic();
        consumer
            .subscribe(&[&topic])
            .unwrap_or_else(|_| panic!("Can't subscribe to topic: {topic}"));

        task::spawn(async move {
            while let Ok(message) = consumer.recv().await {
                match message.topic() {
                    message_topic if topic.eq(message_topic) => {
                        if let Some(payload) = message.payload_view::<str>() {
                            match payload {
                                Ok(payload) => {
                                    let payload: T = serde_json::from_str(payload).expect("Error while deserializing message payload");
                                    trace!("Send message to synapse with raw payload: {:?}", &payload);
                                    callback(payload).await;
                                }
                                Err(error) => { error!("Error while deserializing message from topic: {message_topic} payload: {message:?}, error: {error:?}"); }
                            }
                        }
                    }
                    message_topic => { warn!("Unsupported message from topic: {message_topic}"); }
                }
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
        });
    }
}

pub fn reader(bootstrap_server: &str, group_id: &str) -> Reader {
    Reader::new(bootstrap_server, group_id)
}

pub struct Writer {
    producer: FutureProducer,
}

impl Writer {
    fn new(bootstrap_server: &str,) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_server)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");
        Self { producer }
    }
}

impl<T: Synapse> SynapseSend<T> for Writer {
    fn send(&self, data: &T) {
        self.send_to(&self.producer, data);
    }
}

pub fn writer(bootstrap_server: &str,) -> Writer {
    Writer::new(bootstrap_server)
}
