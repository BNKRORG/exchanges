pub(crate) const API_ROOT_URL: &str = "https://api.strike.me/";

/// User Agent for the client
pub(super) const USER_AGENT_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) const BTC_TICKER: &str = "BTC";
