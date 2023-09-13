use serde::{Deserialize, Serialize};

use crate::core::{MessageSubject, RequestSubject};

pub static TEST_MESSAGE_SUBJECT: Test = Test {};

#[derive(Default)]
pub struct Test;

impl MessageSubject for Test {
    type Type = TestMessage;
    fn subject(&self) -> String {
        "test".to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TestMessage {
    pub text: String,
}

impl RequestSubject for Test {
    type ResponseType = TestResponse;
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TestResponse {
    pub text: String,
}
