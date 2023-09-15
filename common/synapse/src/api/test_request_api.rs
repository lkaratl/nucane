use crate::api::subject;
use crate::api::subject::{TestMessage, TestResponse};
use crate::core::{Handler, MessageSend, RequestReceive, RequestSend};
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
    pub async fn send_test(&self, text: String) -> TestResponse {
        let message = TestMessage {
            text
        };
        self.send_client.send_request(&subject::Test, &message).await.unwrap()
    }

    pub async fn on_test(&self, group: Option<String>, handler: impl Handler<TestMessage, TestResponse>) {
        self.receive_client.handle_request(&subject::Test, group, handler)
            .await
            .expect("");
    }

    pub async fn send_test_binary(&self, content: Vec<u8>) -> Vec<u8> {
        self.send_client.send_request(&subject::TestBinary, &content).await.unwrap()
    }

    pub async fn on_test_binary(&self, group: Option<String>, handler: impl Handler<Vec<u8>, Vec<u8>>) {
        self.receive_client.handle_request(&subject::TestBinary, group, handler)
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
    use crate::core::Handler;

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
    impl Handler<TestMessage, TestResponse> for TestRequestHandler {
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

    #[tokio::test]
    async fn run_requester_binary() {
        let client = TestClient::new("localhost:4222").await;
        for _ in 0..10 {
            let response = client.send_test_binary("Test".as_bytes().to_vec())
                .await;
            println!("Response: {:?}", String::from_utf8(response).unwrap());
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    #[derive(Default)]
    struct TestBinaryRequestHandler;

    #[async_trait]
    impl Handler<Vec<u8>, Vec<u8>> for TestBinaryRequestHandler {
        async fn handle(&self, message: Vec<u8>) -> Vec<u8> {
            let string = String::from_utf8(message).unwrap();
            println!("Request: {:?}", string);
            string.as_bytes().to_vec()
        }
    }

    #[tokio::test]
    async fn run_responder_binary() {
        let client = TestClient::new("localhost:4222").await;
        client.on_test_binary(None, TestBinaryRequestHandler)
            .await;
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}
