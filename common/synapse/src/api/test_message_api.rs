use std::future::Future;
use crate::core::SynapseClient;
use crate::subject;
use crate::subject::TestMessage;
use anyhow::Result;

pub struct TestClient {
    client: SynapseClient,
}

impl TestClient {
    pub async fn new(address: &str) -> Self {
        let client = SynapseClient::new(address).await;
        Self {
            client
        }
    }
    pub async fn send_test(&self, text: String) -> Result<()> {
        let message = TestMessage {
            text
        };
        self.client.message(&subject::TEST_MESSAGE_SUBJECT, &message).await
    }

    pub async fn on_test<H: FnMut(TestMessage) -> F + Send + 'static, F: Future<Output=()> + Send + 'static>(&self, group: Option<String>, handler: H) {
        self.client.on_message(&subject::TEST_MESSAGE_SUBJECT, group, handler)
            .await
            .expect("");
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use crate::api::test_message_api::TestClient;

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

    #[tokio::test]
    async fn run_consumer() {
        let client = TestClient::new("localhost:4222").await;
        client.on_test(None, |message| async move {
            println!("{:?}", message); })
            .await;
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
