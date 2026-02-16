//! Coinbase App client builder

use std::time::Duration;

use super::auth::CoinbaseAuth;
use super::client::CoinbaseAppClient;
use super::error::Error;

/// Coinbase App client builder
#[derive(Debug, Clone)]
pub struct CoinbaseAppClientBuilder {
    /// Authentication
    pub auth: CoinbaseAuth,
    /// Use sandbox APIs
    pub sandbox: bool,
    /// Requests timeout
    pub timeout: Duration,
}

impl Default for CoinbaseAppClientBuilder {
    fn default() -> Self {
        Self {
            auth: CoinbaseAuth::default(),
            sandbox: false,
            timeout: Duration::from_secs(20),
        }
    }
}

impl CoinbaseAppClientBuilder {
    /// Set authentication
    #[inline]
    pub fn auth(mut self, auth: CoinbaseAuth) -> Self {
        self.auth = auth;
        self
    }

    /// Set sandbox APIs
    #[inline]
    pub fn sandbox(mut self, sandbox: bool) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Set timeout (default: 20 secs)
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build client
    #[inline]
    pub fn build(self) -> Result<CoinbaseAppClient, Error> {
        CoinbaseAppClient::from_builder(self)
    }
}
