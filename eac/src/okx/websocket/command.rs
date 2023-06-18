use super::Channel;
use crate::okx::error::OkExError;
use chrono::Utc;
use fehler::throws;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::okx::credential::Credential;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Subscribe(Vec<Channel>),
    Login(Vec<LoginArgs>),
    Ping,
}

impl Command {
    pub fn subscribe(topics: Vec<Channel>) -> Command {
        Command::Subscribe(topics)
    }

    #[throws(OkExError)]
    pub fn login(cred: Credential) -> Command {
        let timestamp = Utc::now().timestamp().to_string();

        let (key, sign) = cred.signature(
            http::Method::GET,
            &timestamp,
            &Url::parse("https://example.com/users/self/verify").unwrap(), // the domain name doesn't matter
            "",
        );
        Self::Login(vec![LoginArgs {
            api_key: key.into(),
            passphrase: cred.passphrase().into(),
            timestamp,
            sign,
        }])
    }

    pub fn ping() -> Command {
        Command::Ping
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginArgs {
    api_key: String,
    passphrase: String,
    timestamp: String,
    sign: String,
}