//! Bitfinex authentication

use std::fmt;

use hmac::{Hmac, Mac};
use sha3::Sha3_384;

use crate::error::Error;

type HmacSha384 = Hmac<Sha3_384>;

/// Bitfinex authentication
#[derive(Clone)]
pub enum BitfinexAuth {
    /// API Keys
    ApiKeys {
        /// API Key
        api_key: String,
        /// Secret Key
        api_secret: String,
    },
}

impl fmt::Debug for BitfinexAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitfinexAuth").finish()
    }
}

impl BitfinexAuth {
    /// Construct API keys credential
    pub fn api_keys<K, S>(api_key: K, api_secret: S) -> Self
    where
        K: Into<String>,
        S: Into<String>,
    {
        Self::ApiKeys {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
        }
    }
}

pub(crate) fn sign_payload<S, P>(secret: S, payload: P) -> Result<String, Error>
where
    S: AsRef<[u8]>,
    P: AsRef<[u8]>,
{
    let mut mac: HmacSha384 = HmacSha384::new_from_slice(secret.as_ref())?;
    mac.update(payload.as_ref());
    let result = mac.finalize();
    let signature: String = hex::encode(result.into_bytes());
    Ok(signature)
}
