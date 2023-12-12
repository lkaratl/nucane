use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Message<Row = Value> {
    Error {
        success: Error,
        ret_msg: String,
        conn_id: String,
        op: String,
    },
    Auth {
        success: bool,
        ret_msg: String,
        conn_id: String,
        op: AuthOperation,
    },
    Pong {
        success: bool,
        ret_msg: PingPongMessage,
        conn_id: String,
        op: PingPongOperation,
    },
    Event {
        success: bool,
        ret_msg: String,
        conn_id: String,
        op: String,
    },
    Data {
        topic: String,
        data: Row,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PingPongMessage {
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PingPongOperation {
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthOperation {
    Auth
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Error {
    True
}
