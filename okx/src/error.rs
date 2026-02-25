//! OKX error

use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

/// OKX error
#[derive(Debug, Error)]
pub enum Error {
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Invalid header error
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// Url error
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// Json error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Serde path error
    #[error(transparent)]
    SerdePath(#[from] serde_path_to_error::Error<serde_json::Error>),
    /// Authentication error
    #[error("authentication: {0}")]
    AuthenticationError(String),
    /// OKX API error
    #[error("OKX API error (code: {code}): {message},{smg}")]
    OkxApiError {
        /// Error code
        code: String,
        /// Error message
        message: String,
        /// Error message details
        smg: String,
    },
}
