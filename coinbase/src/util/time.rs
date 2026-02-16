use std::time::{SystemTime, UNIX_EPOCH};

/// Obtains the current timestamp in UNIX format.
///
/// # Panics
///
/// Panics if the system time is before the UNIX epoch.
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
