use crate::api::subject;
use crate::api::subject::{TestMessage, TestResponse};
use crate::core::{RequestHandler, Synapse};

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
    pub async fn send_test(&self, text: String) -> TestResponse {
        let message = TestMessage {
            text
        };
        self.client.request(&subject::Test, &message).await.unwrap()
    }

    pub async fn on_test(&self, group: Option<String>, handler: impl RequestHandler<TestMessage, TestResponse>) {
        self.client.on_request(&subject::Test, group, handler)
            .await
            .expect("");
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use async_trait::async_trait;
    use crate::api::subject::{TestMessage, TestResponse};
    use crate::api::test_request_api::TestClient;
    use crate::core::RequestHandler;

    #[tokio::test]
    async fn run_requester() {
        let client = TestClient::new("localhost:4222").await;
        for _ in 0..10 {
            let response = client.send_test("Request".to_string())
                .await;
            println!("Response: {:?}", response);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    #[derive(Default)]
    struct TestRequestHandler;

    #[async_trait]
    impl RequestHandler<TestMessage, TestResponse> for TestRequestHandler {
        async fn handle(&self, message: TestMessage) -> TestResponse {
            TestResponse{
                text: "Response".to_string()
            }
        }
    }

    #[tokio::test]
    async fn run_responder() {
        let client = TestClient::new("localhost:4222").await;
        client.on_test(None, TestRequestHandler)
            .await;
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
