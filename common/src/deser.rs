//! Deserialization utilities

use chrono::{DateTime, Utc};
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

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrInteger {
    String(String),
    Signed(i64),
    Unsigned(u64),
}

/// Normalize Unix timestamp to seconds.
///
/// Supported input units:
/// - seconds
/// - milliseconds
/// - microseconds
/// - nanoseconds
pub fn normalize_unix_timestamp_seconds(timestamp: i64) -> i64 {
    let abs = timestamp.unsigned_abs();

    if abs >= 1_000_000_000_000_000_000 {
        timestamp / 1_000_000_000
    } else if abs >= 1_000_000_000_000_000 {
        timestamp / 1_000_000
    } else if abs >= 1_000_000_000_000 {
        timestamp / 1_000
    } else {
        timestamp
    }
}

/// Convert Unix timestamp to UTC `DateTime`, normalized to seconds.
///
/// Returns `None` if the timestamp is out of chrono range.
pub fn unix_timestamp_to_utc_seconds(timestamp: i64) -> Option<DateTime<Utc>> {
    let normalized = normalize_unix_timestamp_seconds(timestamp);
    DateTime::from_timestamp(normalized, 0)
}

/// Deserialize Unix timestamp (`s/ms/us/ns`) into UTC `DateTime`,
/// normalized to seconds.
pub fn deserialize_unix_timestamp_to_utc_seconds<'de, D>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = StringOrInteger::deserialize(deserializer)?;

    let timestamp = match raw {
        StringOrInteger::String(value) => value.parse::<i64>().map_err(de::Error::custom)?,
        StringOrInteger::Signed(value) => value,
        StringOrInteger::Unsigned(value) => {
            i64::try_from(value).map_err(|_| de::Error::custom("timestamp does not fit in i64"))?
        }
    };

    unix_timestamp_to_utc_seconds(timestamp)
        .ok_or_else(|| de::Error::custom("timestamp is out of range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_unix_timestamp_seconds() {
        assert_eq!(
            normalize_unix_timestamp_seconds(1_700_000_000),
            1_700_000_000
        );
        assert_eq!(
            normalize_unix_timestamp_seconds(1_700_000_000_123),
            1_700_000_000
        );
        assert_eq!(
            normalize_unix_timestamp_seconds(1_700_000_000_123_456),
            1_700_000_000
        );
        assert_eq!(
            normalize_unix_timestamp_seconds(1_700_000_000_123_456_789),
            1_700_000_000
        );
    }

    #[test]
    fn test_unix_timestamp_to_utc_seconds() {
        let dt = unix_timestamp_to_utc_seconds(1_700_000_000_123).unwrap();
        assert_eq!(dt.timestamp(), 1_700_000_000);
        assert_eq!(dt.timestamp_subsec_nanos(), 0);
    }

    #[test]
    fn test_deserialize_unix_timestamp_to_utc_seconds() {
        #[derive(Deserialize)]
        struct Payload {
            #[serde(deserialize_with = "deserialize_unix_timestamp_to_utc_seconds")]
            timestamp: DateTime<Utc>,
        }

        let payload: Payload = serde_json::from_str(r#"{"timestamp":"1700000000123"}"#).unwrap();
        assert_eq!(payload.timestamp.timestamp(), 1_700_000_000);
        assert_eq!(payload.timestamp.timestamp_subsec_nanos(), 0);
    }
}
