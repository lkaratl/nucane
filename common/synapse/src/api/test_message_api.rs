use std::future::Future;
use crate::core::{Handler, MessageReceive, MessageSend};
use anyhow::Result;
use crate::api::subject;
use crate::api::subject::TestMessage;
use crate::impls::nats::{NatsReceiver, NatsSender};

pub struct TestClient {
    send_client: NatsSender,
    receive_client: NatsReceiver
}

impl TestClient {
    pub async fn new(address: &str) -> Self {
        Self {
            send_client: NatsSender::new(address).await,
            receive_client: NatsReceiver::new(address).await
        }
    }
    pub async fn send_test(&self, text: String) -> Result<()> {
        let message = TestMessage {
            text
        };
        self.send_client.send_message(&subject::Test, &message).await
    }

    pub async fn on_test(&self, group: Option<String>, handler: impl Handler<TestMessage, ()>) {
        self.receive_client.handle_message(&subject::Test, group, handler)
            .await
            .expect("");
    }

    pub async fn send_test_binary(&self, content: Vec<u8>) -> Result<()> {
        self.send_client.send_message(&subject::TestBinary, &content).await
    }

    pub async fn on_test_binary(&self, group: Option<String>, handler: impl Handler<Vec<u8>, ()>) {
        self.receive_client.handle_message(&subject::TestBinary, group, handler)
            .await
            .expect("");
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use async_trait::async_trait;
    use crate::api::subject::TestMessage;
    use crate::api::test_message_api::TestClient;
    use crate::core::Handler;

    #[tokio::test]
    async fn run_producer() {
        let client = TestClient::new("localhost:4222").await;
        for _ in 0..10 {
            client.send_test("test".to_string())
                .await
                .expect("");
            println!("sent");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    #[derive(Default)]
    struct TestMessageHandler;

    #[async_trait]
    impl Handler<TestMessage, ()> for TestMessageHandler {
        async fn handle(&self, message: TestMessage) {
            println!("{:?}", message);
        }
    }

    #[tokio::test]
    async fn run_consumer() {
        let client = TestClient::new("localhost:4222").await;


        client.on_test(None, TestMessageHandler)
            .await;
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
