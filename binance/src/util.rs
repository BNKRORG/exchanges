use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;

pub fn build_request(parameters: BTreeMap<String, String>) -> String {
    let mut request = String::new();
    for (key, value) in parameters {
        let param = format!("{}={}&", key, value);
        request.push_str(param.as_ref());
    }
    request.pop();
    request
}

pub fn build_signed_request(
    parameters: BTreeMap<String, String>,
    recv_window: u64,
) -> Result<String, Error> {
    build_signed_request_custom(parameters, recv_window, SystemTime::now())
}

pub fn build_signed_request_custom(
    mut parameters: BTreeMap<String, String>,
    recv_window: u64,
    start: SystemTime,
) -> Result<String, Error> {
    if recv_window > 0 {
        parameters.insert("recvWindow".into(), recv_window.to_string());
    }
    let timestamp = get_timestamp(start)?;

    parameters.insert("timestamp".into(), timestamp.to_string());
    Ok(build_request(parameters))
}

fn get_timestamp(start: SystemTime) -> Result<u64, Error> {
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000)
}
