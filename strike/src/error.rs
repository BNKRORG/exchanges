//! Strike error

use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

/// Strike error
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
    /// Missing deposit address in response
    #[error("missing deposit address")]
    MissingDepositAddress,
}
