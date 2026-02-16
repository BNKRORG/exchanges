pub(crate) const API_ROOT_URL: &str = "https://api.bitfinex.com/";
pub(crate) const API_SIGNATURE_PATH: &str = "/api/v2/auth/r/";

/// User Agent for the client
pub(super) const USER_AGENT_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(super) const BTC_TICKER: &str = "BTC";
pub(super) const TBTC_TICKER: &str = "tBTC";
