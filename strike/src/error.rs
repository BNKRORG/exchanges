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
}
