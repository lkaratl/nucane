use chrono::{SecondsFormat, Utc};
use derive_builder::Builder;
use fehler::{throw, throws};
use hyper::Method;
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{from_str, to_string as to_jstring};
use serde_urlencoded::to_string as to_ustring;
use tracing::{debug, error, trace};
use url::Url;
use crate::okx::error::OkExError;

use crate::okx::credential::Credential;

use super::models::Request;

#[derive(Clone, Builder)]
pub struct OkExRest {
    url: String,
    client: Client,
    #[builder(default, setter(strip_option))]
    credential: Option<Credential>,
    demo: bool,
}

impl OkExRest {
    pub fn new(url: &str, demo: bool) -> Self {
        OkExRest {
            url: url.to_string(),
            client: Client::new(),
            credential: None,
            demo,
        }
    }

    pub fn with_credential(url: &str, demo: bool, api_key: &str, api_secret: &str, passphrase: &str) -> Self {
        OkExRest {
            url: url.to_string(),
            client: Client::new(),
            credential: Some(Credential::new(api_key, api_secret, passphrase)),
            demo,
        }
    }

    pub fn builder() -> OkExRestBuilder {
        OkExRestBuilder::default()
    }

    #[throws(OkExError)]
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
            Method::PUT | Method::POST => to_jstring(&req)?,
            _ => "".to_string(),
        };
        trace!("Request body: {body:?}");

        let mut builder = self.client.request(R::METHOD, url.clone());

        if R::SIGNED {
            let cred = self.get_credential()?;
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let (key, signature) = cred.signature(R::METHOD, &timestamp, &url, &body);

            builder = builder
                .header("OK-ACCESS-KEY", key)
                .header("OK-ACCESS-SIGN", signature)
                .header("OK-ACCESS-TIMESTAMP", timestamp)
                .header("OK-ACCESS-PASSPHRASE", cred.passphrase());
        }
        if self.demo {
            builder = builder
                .header("x-simulated-trading", "1");
        }

        let resp = builder
            .header("content-type", "application/json")
            .header("user-agent", "okex-rs")
            .body(body)
            .send()
            .await?;
        self.handle_response(resp).await?
    }

    #[throws(OkExError)]
    fn get_credential(&self) -> &Credential {
        match self.credential.as_ref() {
            None => throw!(OkExError::NoApiKeySet),
            Some(c) => c,
        }
    }

    #[throws(OkExError)]
    async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> T {
        let payload = resp.text().await?;
        trace!("Response: {payload}");

        match from_str::<OkExResponseEnvolope<T>>(&payload) {
            Ok(v) => v.data,
            Err(e) => {
                error!("Cannot deserialize response from {}: {}", payload, e);
                throw!(OkExError::CannotDeserializeResponse(payload))
            }
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct OkExResponseEnvolope<T> {
    code: String,
    msg: String,
    data: T,
}
