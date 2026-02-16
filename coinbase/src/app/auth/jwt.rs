//! Coinbase App API Key authentication via JWT
//!
//! <https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/api-key-authentication>

use std::str;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use p256::SecretKey;
use p256::pkcs8::{self, DecodePrivateKey, EncodePrivateKey};
use reqwest::Method;
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, Signature};
use serde::Serialize;
use url::Url;

use crate::app::error::Error;
use crate::util::time;

const JWT_ALGORITHM: &str = "ES256";
const JWT_ISSUER: &str = "cdp";

/// Coinbase App API authentication via JWT
#[derive(Debug, Clone)]
pub struct Jwt {
    /// API Key provided by the service.
    api_key: String,
    /// Pre-initialized ECDSA signing key pair.
    signing_key: Arc<EcdsaKeyPair>,
    /// RNG for signing.
    rng: SystemRandom,
}

impl Jwt {
    pub(crate) fn new<T1, T2>(api_key: T1, api_secret: T2) -> Result<Self, Error>
    where
        T1: Into<String>,
        T2: AsRef<str>,
    {
        // Format the secret key
        let secret: Vec<u8> = format_key(api_secret.as_ref())?;

        // Initialize SystemRandom.
        let rng: SystemRandom = SystemRandom::new();

        // Initialize the EcdsaKeyPair once with the RNG.
        let signing_key = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &secret, &rng)
            .map_err(|why| Error::InvalidPrivateKey(why.to_string()))?;

        Ok(Self {
            api_key: api_key.into(),
            signing_key: Arc::new(signing_key),
            rng,
        })
    }

    #[inline]
    pub(crate) fn build_uri(method: &Method, url: &Url) -> Result<String, Error> {
        let host: &str = url.host_str().ok_or(Error::HostNotFound)?;
        let path: &str = url.path();

        Ok(format!("{method} {host}{path}"))
    }

    /// Creates the header for the message.
    fn build_header(&self) -> Result<Header<'static>, Error> {
        // Generate 48 random bytes for the nonce (resulting in 64 Base64 characters)
        let mut nonce_bytes: [u8; 48] = [0u8; 48];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|why| Error::BadSignature(why.to_string()))?;

        Ok(Header {
            alg: JWT_ALGORITHM,
            kid: self.api_key.clone(),
            nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
        })
    }

    /// Creates the payload for the message.
    #[inline]
    fn build_payload(&self, uri: Option<String>) -> Payload<'static> {
        Payload::new(self.api_key.clone(), uri)
    }

    /// Signs a message using the pre-initialized ECDSA key pair.
    ///
    /// # Arguments
    ///
    /// * `message`: A byte slice (`&[u8]`) of the message to be signed.
    ///
    /// # Returns
    ///
    /// A `Result<String>` with the base64-encoded signature if successful; otherwise, an error.
    fn sign_message(&self, message: &[u8]) -> Result<String, Error> {
        let signature: Signature = self
            .signing_key
            .sign(&self.rng, message)
            .map_err(|why| Error::BadSignature(why.to_string()))?;
        Ok(to_base64(signature.as_ref()))
    }

    /// Encodes JWT headers and payload into a signed JWT token.
    ///
    /// # Arguments
    ///
    /// * `uri`: the URI being accessed.
    ///
    /// # Returns
    ///
    /// A `Result<String>` with the JWT token if successful; otherwise, an error.
    pub(crate) fn encode(&self, uri: Option<String>) -> Result<String, Error> {
        // Build header and encode to base64
        let header: Header = self.build_header()?;
        let header: String = base64_encode(&header)?;

        // Build payload and encode to base64
        let payload: Payload = self.build_payload(uri);
        let payload: String = base64_encode(&payload)?;

        // Estimate capacity: header + payload + signature + 2 dots
        // Assuming signature is ~43 characters for ECDSA P-256
        let mut message: String = String::with_capacity(header.len() + payload.len() + 50);
        message.push_str(&header);
        message.push('.');
        message.push_str(&payload);

        // Sign the message.
        let signature = self.sign_message(message.as_bytes())?;
        message.push('.');
        message.push_str(&signature);

        Ok(message)
    }
}

#[derive(Serialize)]
struct Header<'a> {
    alg: &'a str,
    kid: String,
    nonce: String,
}

#[derive(Serialize)]
struct Payload<'a> {
    sub: String,
    iss: &'a str,
    nbf: u64,
    exp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
}

impl Payload<'_> {
    fn new(api_key: String, uri: Option<String>) -> Self {
        let now: u64 = time::now();

        Self {
            sub: api_key,
            iss: JWT_ISSUER,
            nbf: now,
            exp: now + 120,
            uri,
        }
    }
}

/// Formats a private key into PKCS#8 format and parses it.
///
/// This function takes a private key in PEM format, attempts to format it into PKCS#8 format,
/// and then parses it. If the key is already in PKCS#8 format, it parses the key directly.
/// The function supports both PKCS#1 and PKCS#8 PEM-encoded EC keys.
fn format_key(key: &str) -> Result<Vec<u8>, Error> {
    // Check if already in pkcs8 format.
    if let Ok(secret_key) = SecretKey::from_pkcs8_pem(key) {
        // Already in PKCS#8 format, verify it's correct
        let pkcs8_pem = secret_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .map_err(|e| Error::InvalidPrivateKey(e.to_string()))?;

        if pkcs8_pem.as_str() == key {
            return parse_key(key);
        }
    }

    // Try to parse as EC private key and convert to PKCS#8
    let secret_key = SecretKey::from_sec1_pem(key)
        .map_err(|e| Error::InvalidPrivateKey(format!("Failed to parse EC key: {}", e)))?;

    let pkcs8_pem = secret_key
        .to_pkcs8_pem(pkcs8::LineEnding::LF)
        .map_err(|e| Error::InvalidPrivateKey(format!("Failed to convert to PKCS#8: {}", e)))?;

    parse_key(pkcs8_pem.as_str())
}

/// Parses a PEM-encoded private key or a base64-encoded key.
///
/// This function takes a byte slice representing either a PEM-encoded private key
/// (with or without the "-----BEGIN PRIVATE KEY-----" and "-----END PRIVATE KEY-----" delimiters)
/// or a base64-encoded key, and returns the raw binary key data.
///
/// # Arguments
///
/// * `api_secret`: A byte slice (`&[u8]`) containing the PEM or base64-encoded private key.
///
/// # Returns
///
/// A `Result<Vec<u8>>` which is Ok containing the decoded binary key data if successful,
/// or an Err with a `Error::InvalidPrivateKey()` containing the error message if any error occurs.
fn parse_key(api_secret: &str) -> Result<Vec<u8>, Error> {
    // Checks for the headers and footers to remove them.
    let base64_encoded = if api_secret.starts_with("-----BEGIN") && api_secret.contains("-----END")
    {
        let start = api_secret
            .find("-----BEGIN")
            .and_then(|s| api_secret[s..].find('\n'))
            .ok_or_else(|| Error::InvalidPrivateKey("No BEGIN delimiter".to_string()))?
            + 1;

        let end = api_secret
            .find("-----END")
            .ok_or_else(|| Error::InvalidPrivateKey("No END delimiter".to_string()))?;

        // Get the data between the header and footer.
        api_secret[start..end]
            .lines()
            .collect::<String>()
            .replace(['\n', '\r'], "")
    } else {
        api_secret.replace(['\n', '\r'], "")
    };

    // Decode the key.
    STANDARD_NO_PAD
        .decode(base64_encoded)
        .map_err(|why| Error::InvalidPrivateKey(why.to_string()))
}

#[inline]
fn to_base64(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Implement serialization for Header to handle base64 encoding
fn base64_encode<T>(input: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let raw: Vec<u8> = serde_json::to_vec(input)?;
    Ok(to_base64(&raw))
}
