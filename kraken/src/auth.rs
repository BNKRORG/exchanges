//! Kraken authentication

use std::borrow::Cow;
use std::fmt;
use std::time::SystemTime;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};
use url::Url;

use crate::error::Error;
use crate::request::KrakenRequestBody;

type HmacSha512 = Hmac<Sha512>;

/// Credentials needed to use private Kraken APIs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KrakenApiCredentials {
    /// The name of the API key
    pub key: String,
    /// The API key secret
    pub secret: String,
}

/// Kraken authentication
#[derive(Clone, Default)]
pub enum KrakenAuth {
    /// No authentication
    #[default]
    None,
    /// API Keys
    ApiKeys(KrakenApiCredentials),
}

impl fmt::Debug for KrakenAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KrakenAuth").finish()
    }
}

impl KrakenAuth {
    /// Construct API key credentials
    pub fn api_keys<K, S>(key: K, secret: S) -> Self
    where
        K: Into<String>,
        S: Into<String>,
    {
        Self::ApiKeys(KrakenApiCredentials {
            key: key.into(),
            secret: secret.into(),
        })
    }
}

/// Serialize a json payload, adding a nonce, and producing a signature using Kraken's scheme
///
/// Arguments:
/// * query_data for the request, with "nonce" value not yet assigned
/// * url path for the request
///
/// Returns:
/// * post_data for the request (encoded query data, with nonce added)
/// * signature over that post data string
pub(crate) fn sign_api(
    credentials: &KrakenApiCredentials,
    url: &Url,
    body: KrakenRequestBody<'_>,
) -> Result<(String, String), Error> {
    // Get the path and query data
    let url_path: &str = url.path();
    let query_str: Option<&str> = url.query();

    // Serialize body to JSON
    let body_json: String = serde_json::to_string(&body)?;

    // Concatenate query and body strings
    let data: Cow<str> = match query_str {
        Some(query) => Cow::Owned(format!("{query}{body_json}")),
        None => Cow::Borrowed(&body_json),
    };

    let sha2_result = {
        let mut hasher = Sha256::default();
        hasher.update(body.nonce.to_string());
        hasher.update(data.as_ref());
        hasher.finalize()
    };

    let hmac_sha_key: Vec<u8> = STANDARD.decode(&credentials.secret)?;

    let mut mac =
        HmacSha512::new_from_slice(&hmac_sha_key).expect("Hmac should work with any key length");
    mac.update(url_path.as_bytes());
    mac.update(&sha2_result);
    let mac = mac.finalize().into_bytes();

    let sig: String = STANDARD.encode(mac);

    Ok((body_json, sig))
}

pub(super) fn nonce() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
