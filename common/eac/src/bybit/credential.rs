use http::Method;
use ring::hmac;
use url::Url;

#[derive(Clone, Debug)]
pub struct Credential {
    key: String,
    secret: String,
}

impl Credential {
    pub(crate) fn new(key: &str, secret: &str) -> Self {
        Self {
            key: key.into(),
            secret: secret.into(),
        }
    }

    pub(crate) fn signature(
        &self,
        method: Method,
        timestamp: &str,
        url: &Url,
        body: &str,
        receive_window: i64,
        ws: bool,
    ) -> (&str, String) {
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret.as_bytes());
        let sign_message = if ws {
            format!("{}{}{}", method.as_str(), url.path(), timestamp)
        } else {
            format!("{}{}{}{}{}", timestamp, self.key, receive_window, url.query().unwrap_or_default(), body,
            )
        };

        let signature = hex::encode(hmac::sign(&signed_key, sign_message.as_bytes()).as_ref());
        (self.key.as_str(), signature)
    }
}
