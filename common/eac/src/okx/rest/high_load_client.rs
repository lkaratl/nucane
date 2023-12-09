use std::collections::HashMap;

use anyhow::Result;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tracing::debug;

use crate::okx::OkExError;
use crate::okx::rest::{OkExRest, Request};

pub struct RateLimitedRestClient {
    client: OkExRest,
    requests_rate: Mutex<HashMap<String, Rate>>,
}

impl RateLimitedRestClient {
    pub fn new(client: OkExRest) -> Self {
        Self {
            client,
            requests_rate: Mutex::new(HashMap::new()),
        }
    }

    pub async fn request<R>(&self, request: R) -> Result<R::Response, OkExError>
        where
            R: Request,
            R::Response: DeserializeOwned
    {
        let key = format!("{}{}", R::METHOD, R::ENDPOINT);
        let mut requests_rate = self.requests_rate
            .lock()
            .await;
        let now_millis = chrono::Utc::now().timestamp_millis();
        if let Some(rate) = requests_rate.get_mut(&key) {
            if rate.requests >= R::REQUESTS_PER_SECOND {
                if rate.last_request + 1000 > now_millis {
                    debug!("Rate limit reached for {key} waiting 1 second");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                rate.requests = 0;
                rate.last_request = now_millis;
            }
            rate.requests += 1;
        } else {
            requests_rate.insert(key, Rate {
                requests: 1,
                last_request: now_millis,
            });
        }
        self.client.request(request).await
    }
}

struct Rate {
    requests: u8,
    last_request: i64,
}
