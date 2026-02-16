//! Binance API

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::large_futures)]
#![warn(rustdoc::bare_urls)]

mod api;
pub mod auth;
pub mod builder;
pub mod client;
mod constant;
pub mod error;
pub mod prelude;
pub mod response;
mod util;
