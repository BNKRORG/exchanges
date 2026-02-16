//! Strike authentication

use std::fmt;

/// Strike authentication
#[derive(Clone)]
pub enum StrikeAuth {
    /// API Key
    ApiKey(String),
}

impl fmt::Debug for StrikeAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StrikeAuth").finish()
    }
}

impl StrikeAuth {
    /// Construct API key credential
    pub fn api_key<K>(key: K) -> Self
    where
        K: Into<String>,
    {
        Self::ApiKey(key.into())
    }
}
