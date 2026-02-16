use serde::Serialize;

/// Empty json object (used as arguments for some APIs)
#[derive(Debug, Serialize)]
pub(crate) struct Empty {}

#[derive(Debug, Serialize)]
pub(crate) struct KrakenRequestBody<'a> {
    pub(crate) nonce: u64,
    #[serde(flatten)]
    pub(crate) request: Request<'a>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum Request<'a> {
    Empty(Empty),
    DepositStatus(DepositStatus<'a>),
    WithdrawStatus(WithdrawStatus<'a>),
    TradesHistory(GetTradesHistory<'a>),
}

#[derive(Debug, Serialize)]
pub(crate) struct DepositStatus<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) asset: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub(crate) struct WithdrawStatus<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) asset: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GetTradesHistory<'a> {
    #[serde(rename = "type")]
    pub r#type: &'a str,
    pub trades: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ofs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_serialize() {
        let status = KrakenRequestBody {
            nonce: 1234567890,
            request: Request::Empty(Empty {}),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#"{"nonce":1234567890}"#);
    }

    #[test]
    fn test_deposit_status_serialize() {
        let status = KrakenRequestBody {
            nonce: 1234567890,
            request: Request::DepositStatus(DepositStatus { asset: Some("XBT") }),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#"{"nonce":1234567890,"asset":"XBT"}"#);
    }
}
