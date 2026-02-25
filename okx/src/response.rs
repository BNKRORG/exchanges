//! OKX API responses

use common::deser::deserialize_string_to_f64;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct OkxApiResponse {
    pub code: String,
    pub msg: String,
    pub data: Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OkxApiErrorData {
    #[serde(rename = "sMsg")]
    pub(crate) s_msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Account {
    /// Detailed asset information per currency
    pub details: Vec<CurrencyDetail>,
}

/// Detailed asset information per currency
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CurrencyDetail {
    /// Currency
    #[serde(rename = "ccy")]
    pub currency: String,
    /// Total equity of the currency
    #[serde(rename = "eq")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
}
