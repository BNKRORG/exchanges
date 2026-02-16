//! Binance authentication

use std::fmt;

use crate::error::Error;

/// Binance authentication
#[derive(Clone, Default)]
pub enum BinanceAuth {
    /// No authentication
    #[default]
    None,
    /// API Keys
    ApiKeys {
        /// API Key
        api_key: String,
        /// Secret Key
        secret_key: String,
    },
}

impl fmt::Debug for BinanceAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth").finish()
    }
}

impl BinanceAuth {
    /// Get API Key
    pub(super) fn get_api_key(&self) -> Result<&str, Error> {
        match self {
            Self::ApiKeys { api_key, .. } => Ok(api_key),
            _ => Err(Error::ApiKeysNotAvailable),
        }
    }

    /// Get API secret key
    pub(super) fn get_api_secret_key(&self) -> Result<&str, Error> {
        match self {
            Self::ApiKeys { secret_key, .. } => Ok(secret_key),
            _ => Err(Error::ApiKeysNotAvailable),
        }
    }
}
