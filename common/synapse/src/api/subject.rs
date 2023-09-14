use serde::{Deserialize, Serialize};

use crate::core::{MessageSubject, RequestSubject};


pub struct Test;

impl MessageSubject for Test {
    type MessageType = TestMessage;
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

pub struct TestBinary;

impl MessageSubject for TestBinary {
    type MessageType = Vec<u8>;
    const SUBJECT: &'static str = "test-binary";
}


impl RequestSubject for TestBinary {
    type ResponseType = Vec<u8>;
}
