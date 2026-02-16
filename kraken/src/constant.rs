//! Kraken constants

use std::collections::HashSet;
use std::sync::LazyLock;

pub(crate) const API_ROOT_URL: &str = "https://api.kraken.com";

pub(crate) const API_VERSION: u16 = 0;

/// User Agent for the client
pub(super) const USER_AGENT_NAME: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Kraken BTC ticker symbol
pub const XBT_TICKER: &str = "XBT";
/// Kraken BTC ticker symbol
pub const XXBT_TICKER: &str = "XXBT";

/// Kraken BTC tickers
pub static TICKERS: LazyLock<HashSet<&str>> =
    LazyLock::new(|| HashSet::from([XBT_TICKER, XXBT_TICKER, "XBT.B", "XBT.M", "XBT.F", "XBT.T"]));
