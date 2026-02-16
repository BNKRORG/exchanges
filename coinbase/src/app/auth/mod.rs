//! Coinbase App APIs authentication
//!
//! <https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization>

use std::fmt;

pub(super) mod jwt;

/// Coinbase authentication
#[derive(Clone, Default)]
pub enum CoinbaseAuth {
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

impl fmt::Debug for CoinbaseAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoinbaseAuth").finish()
    }
}
