//! Deserialization utilities

use serde::{Deserialize, Deserializer, de};

/// Deserialize a string as f64
pub fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}
