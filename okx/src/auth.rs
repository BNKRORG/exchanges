//! OKX authentication

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Method;
use sha2::Sha256;

use crate::error::Error;
use crate::util;

/// OKX API credentials
///
/// <https://www.okx.com/docs-v5/en/#overview-api-key-creation>
#[derive(Clone)]
pub struct OkxApiCredentials {
    /// API Key
    pub api_key: String,
    /// API Secret
    pub api_secret: String,
    /// API Passphrase
    pub passphrase: String,
}

impl fmt::Debug for OkxApiCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OkxApiCredentials").finish()
    }
}

/// <https://www.okx.com/docs-v5/en/#overview-rest-authentication>
pub(crate) fn generate_signature(
    api_secret: &str,
    timestamp: &DateTime<Utc>,
    method: &Method,
    path: &str,
    body: &str,
) -> Result<String, Error> {
    // Format the timestamp
    let timestamp: String = util::format_timestamp(timestamp);

    // Create the pre-hash string
    let pre_hash_payload: String = format!("{timestamp}{method}{path}{body}");

    // Prepare the secret key
    let mut hmac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .map_err(|e| Error::AuthenticationError(format!("HMAC: {e}")))?;

    // Sign the pre-hash string with the secret key
    hmac.update(pre_hash_payload.as_bytes());

    // Finalize the HMAC-SHA256 and get the bytes
    let hmac = hmac.finalize();

    // Encode the signature bytes to base64
    Ok(general_purpose::STANDARD.encode(hmac.into_bytes()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_generate_signature() {
        let timestamp = DateTime::from_str("2020-12-08T09:08:57.715Z").unwrap();
        let signature: String = generate_signature(
            "22582BD0CFF14C41EDBF1AB98506286D",
            &timestamp,
            &Method::GET,
            "/api/v5/account/balance?ccy=BTC",
            "",
        )
        .unwrap();
        assert_eq!(signature, "HiZhvSfMtWJA3uUIVXV3a/bSXNPCWvYFXoGCVS8V4zY=");
    }
}
