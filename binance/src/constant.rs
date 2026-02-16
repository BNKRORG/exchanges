use std::time::Duration;

pub(crate) const SPOT_MAINNET: &str = "https://api.binance.com";
pub(crate) const SPOT_MAINNET_US: &str = "https://api.binance.us";
pub(crate) const SPOT_TESTNET: &str = "https://testnet.binance.vision";

/// User Agent for the client
pub(crate) const USER_AGENT_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) const DEFAULT_RECV_WINDOW: u64 = 5000;
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(25);

/// <https://www.binance.com/en/support/announcement/detail/9820396bf54644c39e666b4780622846>
pub(crate) const MAX_WEIGHT_PER_MIN: u32 = 6000;

pub(crate) const BTC_TICKER: &str = "BTC";
