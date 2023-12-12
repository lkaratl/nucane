use chrono::Utc;
use fehler::throws;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::bybit::BybitError;
use crate::bybit::credential::Credential;

use super::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Subscribe(Vec<String>),
    Auth(Vec<String>),
}

impl Command {
    pub fn subscribe(topics: Vec<Channel>) -> Command {
        let topics = topics.into_iter()
            .map(|topic| topic.to_string())
            .collect();
        Command::Subscribe(topics)
    }

    #[throws(BybitError)]
    pub fn login(cred: Credential) -> Command {
        let timestamp = (Utc::now().timestamp() * 1000).to_string();

        let (key, sign) = cred.signature(
            http::Method::GET,
            &timestamp,
            &Url::parse("https://example.com/realtime").unwrap(), // the domain name doesn't matter
            "",
            0,
            true,
        );
        Self::Auth(vec![
            key.into(),
            timestamp.to_string(),
            sign,
        ])
    }
}
