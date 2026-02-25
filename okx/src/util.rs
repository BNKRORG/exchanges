use chrono::{DateTime, Utc};

/// Format timestamp to the following format: YYYY-MM-DDTHH:mm:ss.sssZ (i.e., 2020-12-08T09:08:57.715Z)
#[inline]
pub(crate) fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}
