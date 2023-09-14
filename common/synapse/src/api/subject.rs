use serde::{Deserialize, Serialize};

use crate::core::{MessageSubject, RequestSubject};


pub struct Test;

impl MessageSubject for Test {
    type Type = TestMessage;
    const SUBJECT: &'static str = "test";
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
