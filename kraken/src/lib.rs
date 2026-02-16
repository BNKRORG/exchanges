//! Kraken APIs

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::large_futures)]
#![warn(rustdoc::bare_urls)]

pub mod auth;
pub mod client;
pub mod constant;
pub mod error;
pub mod prelude;
mod request;
pub mod response;
