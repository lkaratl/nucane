use std::future::Future;
use crate::core::{MessageHandler, Synapse};
use anyhow::Result;
use crate::api::subject;
use crate::api::subject::TestMessage;

pub struct TestClient {
    client: Synapse,
}

impl TestClient {
    pub async fn new(address: &str) -> Self {
        let client = Synapse::new(address).await;
        Self {
            client
        }
    }
    pub async fn send_test(&self, text: String) -> Result<()> {
        let message = TestMessage {
            text
        };
        self.client.message(&subject::Test, &message).await
    }

    pub async fn on_test(&self, group: Option<String>, handler: impl MessageHandler<TestMessage>) {
        self.client.on_message(&subject::Test, group, handler)
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
    use crate::core::MessageHandler;

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
    impl MessageHandler<TestMessage> for TestMessageHandler {
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
