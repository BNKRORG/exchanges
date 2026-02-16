//! Binance error

use thiserror::Error;
use url::ParseError;

/// Binance error
#[derive(Debug, Error)]
pub enum Error {
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Invalid header
    #[error(transparent)]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    /// URL error
    #[error(transparent)]
    Url(#[from] ParseError),
    /// Timestamp error
    #[error(transparent)]
    Timestamp(#[from] std::time::SystemTimeError),
    /// Asset not found
    #[error("Asset not found")]
    AssetNotFound,
    /// API keys not available
    #[error("API keys not available")]
    ApiKeysNotAvailable,
    /// Can't clone the request
    #[error("can't clone the request")]
    CantCloneRequest,
}
