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

fn string_or_integer_to_i64<E: de::Error>(raw: StringOrInteger) -> Result<i64, E> {
    match raw {
        StringOrInteger::String(value) => value.parse::<i64>().map_err(de::Error::custom),
        StringOrInteger::Signed(value) => Ok(value),
        StringOrInteger::Unsigned(value) => {
            i64::try_from(value).map_err(|_| de::Error::custom("timestamp does not fit in i64"))
        }
    }
}

/// Convert Unix timestamp (seconds) to UTC `DateTime`.
///
/// Returns `None` if the timestamp is out of chrono range.
pub fn unix_timestamp_seconds_to_utc_seconds(timestamp: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(timestamp, 0)
}

/// Convert Unix timestamp (milliseconds) to UTC `DateTime`, normalized to seconds.
///
/// Returns `None` if the timestamp is out of chrono range.
pub fn unix_timestamp_milliseconds_to_utc_seconds(timestamp: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(timestamp / 1_000, 0)
}

fn parse_deserialized_timestamp<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = StringOrInteger::deserialize(deserializer)?;
    string_or_integer_to_i64(raw)
}

/// Deserialize Unix timestamp (seconds) into UTC `DateTime`.
pub fn deserialize_unix_timestamp_seconds_to_utc_seconds<'de, D>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = parse_deserialized_timestamp(deserializer)?;
    unix_timestamp_seconds_to_utc_seconds(timestamp)
        .ok_or_else(|| de::Error::custom("timestamp is out of range"))
}

/// Deserialize Unix timestamp (milliseconds) into UTC `DateTime`, normalized to seconds.
pub fn deserialize_unix_timestamp_milliseconds_to_utc_seconds<'de, D>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = parse_deserialized_timestamp(deserializer)?;
    unix_timestamp_milliseconds_to_utc_seconds(timestamp)
        .ok_or_else(|| de::Error::custom("timestamp is out of range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_timestamp_seconds_to_utc_seconds() {
        let dt = unix_timestamp_seconds_to_utc_seconds(1_700_000_000).unwrap();
        assert_eq!(dt.timestamp(), 1_700_000_000);
        assert_eq!(dt.timestamp_subsec_nanos(), 0);
    }

    #[test]
    fn test_unix_timestamp_milliseconds_to_utc_seconds() {
        let dt = unix_timestamp_milliseconds_to_utc_seconds(1_700_000_000_123).unwrap();
        assert_eq!(dt.timestamp(), 1_700_000_000);
        assert_eq!(dt.timestamp_subsec_nanos(), 0);
    }

    #[test]
    fn test_deserialize_unix_timestamp_seconds_to_utc_seconds() {
        #[derive(Deserialize)]
        struct Payload {
            #[serde(deserialize_with = "deserialize_unix_timestamp_seconds_to_utc_seconds")]
            timestamp: DateTime<Utc>,
        }

        let payload: Payload = serde_json::from_str(r#"{"timestamp":"1700000000"}"#).unwrap();
        assert_eq!(payload.timestamp.timestamp(), 1_700_000_000);
        assert_eq!(payload.timestamp.timestamp_subsec_nanos(), 0);
    }

    #[test]
    fn test_deserialize_unix_timestamp_milliseconds_to_utc_seconds() {
        #[derive(Deserialize)]
        struct Payload {
            #[serde(deserialize_with = "deserialize_unix_timestamp_milliseconds_to_utc_seconds")]
            timestamp: DateTime<Utc>,
        }

        let payload: Payload = serde_json::from_str(r#"{"timestamp":"1700000000123"}"#).unwrap();
        assert_eq!(payload.timestamp.timestamp(), 1_700_000_000);
        assert_eq!(payload.timestamp.timestamp_subsec_nanos(), 0);
    }
}
