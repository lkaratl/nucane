use std::future::Future;
use crate::core::SynapseClient;
use crate::subject;
use crate::subject::{TestMessage, TestResponse};

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
    pub async fn send_test(&self, text: String) -> TestResponse {
        let message = TestMessage {
            text
        };
        self.client.request(&subject::TEST_MESSAGE_SUBJECT, &message).await.unwrap()
    }

    pub async fn on_test<H: FnMut(TestMessage) -> F + Send + 'static, F: Future<Output=TestResponse> + Send + 'static>(&self, group: Option<String>, handler: H) {
        self.client.on_request(&subject::TEST_MESSAGE_SUBJECT, group, handler)
            .await
            .expect("");
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use crate::api::test_request_api::TestClient;
    use crate::subject::TestResponse;

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

    #[tokio::test]
    async fn run_responder() {
        let client = TestClient::new("localhost:4222").await;
        client.on_test(None, |message| async move {
            println!("Request {:?}", message);
        TestResponse{
            text: "Response".to_string()
        }})
            .await;
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
