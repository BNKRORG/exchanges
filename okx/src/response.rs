//! OKX API responses

use common::deser::{deserialize_string_or_number_to_u64, deserialize_string_to_f64};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
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

/// Status of deposit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum DepositStatus {
    /// Waiting for confirmation.
    #[serde(rename = "0")]
    WaitingForConfirmation,
    /// Credited
    #[serde(rename = "1")]
    DepositCredited,
    /// Success.
    #[serde(rename = "2")]
    DepositSuccessful,
    /// Pending due to temporary deposit suspension on this crypto currency
    #[serde(rename = "8")]
    PendingDueToTemporaryDepositSuspension,
    /// Match the address blacklist
    #[serde(rename = "11")]
    MatchAddressBlacklist,
    /// Account or deposit is frozen
    #[serde(rename = "12")]
    AccountOrDepositIsFrozen,
    /// Sub-account deposit interception
    #[serde(rename = "13")]
    SubAccountDepositInterception,
    /// KYC limit
    #[serde(rename = "14")]
    KycLimit,
    /// Pending response from Travel Rule vendor
    #[serde(rename = "17")]
    PendingTravelRuleVendor,
}

/// Status of withdrawal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WithdrawalStatus {
    /// Waiting withdrawal.
    #[serde(rename = "0")]
    WaitingWithdrawal,
    /// Waiting manual review.
    #[serde(
        rename = "4",
        alias = "5",
        alias = "6",
        alias = "8",
        alias = "9",
        alias = "12"
    )]
    WaitingManualReview,
    /// Approved.
    #[serde(rename = "7")]
    Approved,
    /// Broadcasting your transaction to chain.
    #[serde(rename = "1")]
    Withdrawing,
    /// Waiting transfer.
    #[serde(rename = "10")]
    WaitingTransfer,
    /// Pending transaction validation.
    #[serde(rename = "15")]
    PendingTransactionValidation,
    /// Delayed by local laws and regulations.
    #[serde(rename = "16")]
    RegulatoryDelay,
    /// Canceling.
    #[serde(rename = "-3")]
    Canceling,
    /// Canceled.
    #[serde(rename = "-2")]
    Canceled,
    /// Failed.
    #[serde(rename = "-1")]
    Failed,
    /// Success.
    #[serde(rename = "2")]
    WithdrawalSuccessful,
    /// Pending response from Travel Rule vendor.
    #[serde(rename = "17")]
    PendingTravelRuleVendor,
    /// Insufficient balance in the hot wallet.
    #[serde(rename = "19")]
    InsufficientHotWalletBalance,
}

fn deserialize_optional_enum<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    Ok(value.and_then(|value| serde_json::from_value(value).ok()))
}

/// Deposit transaction
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DepositTransaction {
    /// Deposit identifier.
    #[serde(rename = "depId")]
    pub id: String,
    /// Currency.
    #[serde(rename = "ccy")]
    pub currency: String,
    /// Deposit amount.
    #[serde(rename = "amt")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Deposit status.
    #[serde(default, deserialize_with = "deserialize_optional_enum")]
    pub state: Option<DepositStatus>,
    /// Deposit transaction identifier.
    #[serde(rename = "txId")]
    pub tx_id: String,
    /// Deposit timestamp.
    #[serde(rename = "ts")]
    #[serde(deserialize_with = "deserialize_string_or_number_to_u64")]
    pub timestamp: u64,
}

/// Withdrawal transaction
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WithdrawalTransaction {
    /// Withdrawal identifier.
    #[serde(rename = "wdId")]
    pub id: String,
    /// Currency
    #[serde(rename = "ccy")]
    pub currency: String,
    /// Amount
    #[serde(rename = "amt")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Fee
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub fee: f64,
    /// State
    #[serde(default, deserialize_with = "deserialize_optional_enum")]
    pub state: Option<WithdrawalStatus>,
    /// Transaction identifier.
    #[serde(rename = "txId")]
    pub tx_id: String,
    /// Withdrawal timestamp.
    #[serde(rename = "ts")]
    #[serde(deserialize_with = "deserialize_string_or_number_to_u64")]
    pub timestamp: u64,
}

/// Trade side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    /// Buy trade.
    Buy,
    /// Sell trade.
    Sell,
}

/// Executed trade.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Trade {
    /// Trade identifier.
    #[serde(rename = "tradeId")]
    pub id: String,
    /// Instrument identifier (for example, `BTC-USDT`).
    #[serde(rename = "instId")]
    pub instrument_id: String,
    /// Order identifier.
    #[serde(rename = "ordId")]
    pub order_id: String,
    /// Trade side.
    pub side: TradeSide,
    /// Filled size.
    #[serde(rename = "fillSz")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub size: f64,
    /// Fill price.
    #[serde(rename = "fillPx")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
    /// Trade fee.
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub fee: f64,
    /// Fee currency.
    #[serde(rename = "feeCcy")]
    pub fee_currency: String,
    /// Unix timestamp in milliseconds.
    #[serde(rename = "ts")]
    #[serde(deserialize_with = "deserialize_string_or_number_to_u64")]
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_deposit_tx() {
        let json = r#"{
        "actualDepBlkConfirm": "2",
        "amt": "1",
        "areaCodeFrom": "",
        "ccy": "BTC",
        "chain": "BTC",
        "depId": "88****33",
        "from": "",
        "fromWdId": "",
        "state": "2",
        "to": "TN4hGjVXMzy*********9b4N1aGizqs",
        "ts": "1674038705000",
        "txId": "fee235b3e812********857d36bb0426917f0df1802"
    }"#;

        let tx: DepositTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(
            tx,
            DepositTransaction {
                id: "88****33".to_string(),
                currency: "BTC".to_string(),
                amount: 1.0,
                state: Some(DepositStatus::DepositSuccessful),
                tx_id: "fee235b3e812********857d36bb0426917f0df1802".to_string(),
                timestamp: 1674038705000,
            }
        );
    }

    #[test]
    fn test_deserialize_withdrawal_tx() {
        let json = r#"{
      "note": "",
      "chain": "BTC",
      "fee": "0.00007",
      "feeCcy": "BTC",
      "ccy": "BTC",
      "clientId": "",
      "toAddrType": "1",
      "amt": "0.029809",
      "txId": "35c******b360a174d",
      "from": "bc1q****359",
      "areaCodeFrom": "86",
      "to": "bc1q...",
      "areaCodeTo": "",
      "state": "2",
      "ts": "1655251200000",
      "nonTradableAsset": false,
      "wdId": "15447421"
    }"#;

        let tx: WithdrawalTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(
            tx,
            WithdrawalTransaction {
                id: "15447421".to_string(),
                currency: "BTC".to_string(),
                amount: 0.029809,
                fee: 0.00007,
                state: Some(WithdrawalStatus::WithdrawalSuccessful),
                tx_id: "35c******b360a174d".to_string(),
                timestamp: 1655251200000,
            }
        );
    }

    #[test]
    fn test_deserialize_deposit_tx_unknown_state_as_none() {
        let json = r#"{
        "amt": "1",
        "ccy": "BTC",
        "depId": "88****33",
        "state": "999",
        "ts": "1674038705000",
        "txId": "fee235b3e812********857d36bb0426917f0df1802"
    }"#;

        let tx: DepositTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.state, None);
    }

    #[test]
    fn test_deserialize_withdrawal_tx_unknown_state_as_none() {
        let json = r#"{
      "fee": "0.00007",
      "ccy": "BTC",
      "amt": "0.029809",
      "txId": "35c******b360a174d",
      "state": "999",
      "ts": "1655251200000",
      "wdId": "15447421"
    }"#;

        let tx: WithdrawalTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.state, None);
    }

    #[test]
    fn test_deserialize_trade() {
        let json = r#"{
            "side": "buy",
            "fillSz": "0.00192834",
            "fillPx": "51858",
            "fillPxVol": "",
            "fillFwdPx": "",
            "fee": "-0.00000192834",
            "fillPnl": "0",
            "ordId": "680800019749904384",
            "feeRate": "-0.001",
            "instType": "SPOT",
            "fillPxUsd": "",
            "instId": "BTC-USDT",
            "clOrdId": "",
            "posSide": "net",
            "billId": "680800019754098688",
            "subType": "1",
            "fillMarkVol": "",
            "tag": "",
            "fillTime": "1708587373361",
            "execType": "T",
            "fillIdxPx": "",
            "tradeId": "744876980",
            "fillMarkPx": "",
            "feeCcy": "BTC",
            "ts": "1708587373362",
            "tradeQuoteCcy": "USDT"
        }"#;

        let trade: Trade = serde_json::from_str(json).unwrap();

        assert_eq!(
            trade,
            Trade {
                id: "744876980".to_string(),
                instrument_id: "BTC-USDT".to_string(),
                order_id: "680800019749904384".to_string(),
                side: TradeSide::Buy,
                size: 0.00192834,
                price: 51858.0,
                fee: -0.00000192834,
                fee_currency: "BTC".to_string(),
                timestamp: 1708587373362,
            }
        )
    }
}
