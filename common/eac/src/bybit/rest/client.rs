use chrono::Utc;
use derive_builder::Builder;
use fehler::{throw, throws};
use hyper::Method;
use reqwest::{Client, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::from_str;
use serde_urlencoded::to_string as to_ustring;
use tracing::{error, trace};
use url::Url;

use crate::bybit::credential::Credential;
use crate::bybit::error::BybitError;

use super::models::Request;

#[derive(Clone, Builder)]
pub struct BybitRest {
    url: String,
    client: Client,
    #[builder(default, setter(strip_option))]
    credential: Option<Credential>,
}

impl BybitRest {
    pub fn new(url: &str) -> Self {
        BybitRest {
            url: url.to_string(),
            client: Client::new(),
            credential: None,
        }
    }

    pub fn with_credential(url: &str, api_key: &str, api_secret: &str) -> Self {
        BybitRest {
            url: url.to_string(),
            client: Client::new(),
            credential: Some(Credential::new(api_key, api_secret)),
        }
    }

    pub fn builder() -> BybitRestBuilder {
        BybitRestBuilder::default()
    }

    #[throws(BybitError)]
    pub async fn request<R>(&self, req: R) -> R::Response
        where
            R: Request,
            R::Response: DeserializeOwned,
    {
        let url = format!("{}{}", self.url, R::ENDPOINT);
        let mut url = Url::parse(&url)?;
        if matches!(R::METHOD, Method::GET | Method::DELETE) && R::HAS_PAYLOAD {
            url.set_query(Some(&to_ustring(&req)?));
        }
        trace!("Request url: {url:?}");
        let body = match R::METHOD {
            Method::PUT | Method::POST => to_ustring(&req)?,
            _ => "".to_string(),
        };
        trace!("Request body: {body:?}");

        let mut builder = self.client.request(R::METHOD, url.clone());

        if R::SIGNED {
            let cred = self.get_credential()?;
            let timestamp = (Utc::now().timestamp() * 1000).to_string();
            let (key, signature) = cred.signature(R::METHOD, &timestamp, &url, &body, false);

            builder = builder
                .header("X-BAPI-API-KEY", key)
                .header("X-BAPI-SIGN", signature)
                .header("X-BAPI-TIMESTAMP", timestamp)
        }
        let resp = builder
            .header("content-type", "application/x-www-form-urlencoded")
            .header("user-agent", "bybit-rs")
            .body(body)
            .send()
            .await?;
        self.handle_response(resp).await?
    }

    #[throws(BybitError)]
    fn get_credential(&self) -> &Credential {
        match self.credential.as_ref() {
            None => throw!(BybitError::NoApiKeySet),
            Some(c) => c,
        }
    }

    #[throws(BybitError)]
    async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> T {
        let payload = resp.text().await?;
        trace!("Response: {payload}");

        match from_str::<BybitResponseEnvelope<T>>(&payload) {
            Ok(v) => v.result,
            Err(e) => {
                error!("Cannot deserialize response from {}: {}", payload, e);
                throw!(BybitError::CannotDeserializeResponse(payload))
            }
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BybitResponseEnvelope<T> {
    pub ret_code: i64,
    pub ret_msg: String,
    pub time: i64,
    pub result: T,
}
