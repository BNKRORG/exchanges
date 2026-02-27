//! Deserialization utilities

use serde::{Deserialize, Deserializer, de};

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(u64),
}

/// Deserialize a string as f64
pub fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

/// Deserialize a string or number as u64
pub fn deserialize_string_or_number_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(value) => value.parse().map_err(de::Error::custom),
        StringOrNumber::Number(value) => Ok(value),
    }
}
