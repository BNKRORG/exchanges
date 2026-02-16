//! Coinbase App error

use thiserror::Error;

use super::response::CoinbaseErrorMessage;

/// Coinbase App error
#[derive(Debug, Error)]
pub enum Error {
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Url error
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// JSON error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Coinbase response error
    #[error("coinbase: {0}")]
    Coinbase(CoinbaseErrorMessage),
    /// Invalid private key
    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),
    /// Bad signature
    #[error("bad signature: {0}")]
    BadSignature(String),
    /// Host not found
    #[error("host not found")]
    HostNotFound,
}
