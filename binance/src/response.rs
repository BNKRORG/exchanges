//! Binance responses

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use common::deser::{deserialize_string_to_f64, deserialize_unix_timestamp_to_utc_seconds};
use serde::Deserialize;

use crate::constant::BTC_TICKER;

/// Exchange information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    /// Timezone
    pub timezone: String,
    /// Server time
    #[serde(deserialize_with = "deserialize_unix_timestamp_to_utc_seconds")]
    pub server_time: DateTime<Utc>,
    /// Rate limits
    pub rate_limits: Vec<RateLimit>,
    /// Exchange symbols
    pub symbols: Vec<Symbol>,
}

/// Rate limit
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    /// Rate limit type
    pub rate_limit_type: String,
    /// Interval for the rate limit
    pub interval: String,
    /// Interval number
    pub interval_num: u16,
    /// Limit
    pub limit: u64,
}

/// Symbol information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    /// Symbol
    pub symbol: String,
    /// Status
    pub status: String,
    /// Base asset
    pub base_asset: String,
    /// Base asset precision
    pub base_asset_precision: u64,
    /// Quote asset
    pub quote_asset: String,
    /// Quote precision
    pub quote_precision: u64,
    /// Order types
    pub order_types: Vec<String>,
    /// Iceberg allowed
    pub iceberg_allowed: bool,
    /// Spot trading allowed
    pub is_spot_trading_allowed: bool,
    /// Margin trading allowed
    pub is_margin_trading_allowed: bool,
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

impl Eq for Symbol {}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol.cmp(&other.symbol)
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
    }
}

/// Account information
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    /// Maker commission rate
    pub maker_commission: f32,
    /// Taker commission rate
    pub taker_commission: f32,
    /// Buyer commission rate
    pub buyer_commission: f32,
    /// Seller commission rate
    pub seller_commission: f32,
    /// Can trade
    pub can_trade: bool,
    /// Can withdraw
    pub can_withdraw: bool,
    /// Can deposit
    pub can_deposit: bool,
    /// Balances
    pub balances: Vec<Balance>,
}

impl AccountInformation {
    /// Get the balance for the given asset
    #[inline]
    pub fn balance_for_asset(&self, asset: &str) -> Option<&Balance> {
        self.balances.iter().find(|&balance| balance.asset == asset)
    }

    /// Get the BTC balance
    #[inline]
    pub fn bitcoin_balance(&self) -> Option<&Balance> {
        self.balance_for_asset(BTC_TICKER)
    }
}

/// Balance
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Asset
    pub asset: String,
    /// Free balance
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub free: f64,
    /// Locked balance
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub locked: f64,
}

impl Balance {
    /// Calculate the total balance
    #[inline]
    pub fn total(&self) -> f64 {
        self.free + self.locked
    }
}

/// Deposit transaction
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositTransaction {
    /// Deposit identifier.
    pub id: String,
    /// Asset.
    pub coin: String,
    /// Amount.
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Network.
    pub network: String,
    /// Deposit status.
    pub status: DepositStatus,
    /// Address.
    pub address: String,
    /// Transaction identifier.
    #[serde(rename = "txId")]
    pub tx_id: String,
    /// Deposit time.
    #[serde(deserialize_with = "deserialize_unix_timestamp_to_utc_seconds")]
    pub insert_time: DateTime<Utc>,
    /// Confirmation progress.
    pub confirm_times: String,
}

/// Deposit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepositStatus {
    /// Pending.
    Pending,
    /// Success.
    Success,
    /// Rejected
    Rejected,
    /// Credited but cannot be withdrawn yet.
    CreditedButCannotWithdraw,
    /// Wrong deposit.
    WrongDeposit,
    /// Waiting for user confirmation.
    WaitingUserConfirm,
    /// Unknown status code returned by Binance.
    Unknown(u8),
}

impl From<u8> for DepositStatus {
    fn from(status: u8) -> Self {
        match status {
            0 => Self::Pending,
            1 => Self::Success,
            2 => Self::Rejected,
            6 => Self::CreditedButCannotWithdraw,
            7 => Self::WrongDeposit,
            8 => Self::WaitingUserConfirm,
            value => Self::Unknown(value),
        }
    }
}

impl<'de> Deserialize<'de> for DepositStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(u8::deserialize(deserializer)?))
    }
}

/// Withdrawal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WithdrawStatus {
    /// Email sent.
    EmailSent,
    /// Awaiting approval.
    AwaitingApproval,
    /// Rejected.
    Rejected,
    /// Processing.
    Processing,
    /// Completed.
    Completed,
    /// Unknown status code returned by Binance.
    Unknown(u8),
}

impl From<u8> for WithdrawStatus {
    fn from(status: u8) -> Self {
        match status {
            0 => Self::EmailSent,
            2 => Self::AwaitingApproval,
            3 => Self::Rejected,
            4 => Self::Processing,
            6 => Self::Completed,
            value => Self::Unknown(value),
        }
    }
}

impl<'de> Deserialize<'de> for WithdrawStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(u8::deserialize(deserializer)?))
    }
}

/// Withdrawal transaction
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalTransaction {
    /// Withdrawal identifier.
    pub id: String,
    /// Asset.
    pub coin: String,
    /// Amount.
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Fee.
    #[serde(rename = "transactionFee")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub transaction_fee: f64,
    /// Withdrawal status.
    pub status: WithdrawStatus,
    /// Address.
    pub address: String,
    /// Transaction identifier.
    #[serde(rename = "txId")]
    pub tx_id: String,
    /// Requested time.
    pub apply_time: String,
    /// Network.
    pub network: String,
}

/// Binance trade
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade ID
    pub id: u64,
    /// Price
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
    /// Quantity
    #[serde(rename = "qty")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub base_qty: f64,
    /// Quote quantity
    #[serde(rename = "quoteQty")]
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub quote_qty: f64,
    /// Commission
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub commission: f64,
    /// Commission asset
    pub commission_asset: String,
    /// Time
    #[serde(deserialize_with = "deserialize_unix_timestamp_to_utc_seconds")]
    pub time: DateTime<Utc>,
    /// Whether is buyer
    pub is_buyer: bool,
    /// Whether is maker
    pub is_maker: bool,
    /// Whether is best match
    pub is_best_match: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_account_information() {
        let json = r#"{
    "makerCommission": 15,
    "takerCommission": 15,
    "buyerCommission": 0,
    "sellerCommission": 0,
    "canTrade": true,
    "canWithdraw": true,
    "canDeposit": true,
    "updateTime": 123456789,
    "accountType": "SPOT",
    "balances": [{
            "asset": "BTC",
            "free": "4723846.89208129",
            "locked": "0.00000000"
        },
        {
            "asset": "LTC",
            "free": "4763368.68006011",
            "locked": "0.00000000"
        }
    ],
    "permissions": [
        "SPOT"
    ]
}"#;

        let account: AccountInformation = serde_json::from_str(json).unwrap();

        assert_eq!(account.maker_commission, 15.0);
        assert_eq!(account.taker_commission, 15.0);
        assert_eq!(account.buyer_commission, 0.0);
        assert_eq!(account.seller_commission, 0.0);
        assert_eq!(account.can_trade, true);
        assert_eq!(account.can_withdraw, true);
        assert_eq!(account.can_deposit, true);
        assert_eq!(
            account.balances,
            vec![
                Balance {
                    asset: "BTC".to_string(),
                    free: 4723846.89208129,
                    locked: 0.0,
                },
                Balance {
                    asset: "LTC".to_string(),
                    free: 4763368.68006011,
                    locked: 0.0,
                }
            ]
        );
    }

    #[test]
    fn test_deserialize_deposit_transaction() {
        let json = r#"{
    "id": "769800519366885376",
    "amount": "0.001",
    "coin": "BTC",
    "network": "BTC",
    "status": 1,
    "address": "bc1q...",
    "txId": "0x123",
    "insertTime": 1661493146000,
    "confirmTimes": "1/1"
}"#;

        let tx: DepositTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.id, "769800519366885376");
        assert_eq!(tx.amount, 0.001);
        assert_eq!(tx.coin, "BTC");
        assert_eq!(tx.network, "BTC");
        assert_eq!(tx.status, DepositStatus::Success);
        assert_eq!(tx.tx_id, "0x123");
        assert_eq!(
            tx.insert_time,
            DateTime::from_timestamp(1661493146, 0).unwrap()
        );
        assert_eq!(tx.confirm_times, "1/1");
    }

    #[test]
    fn test_deserialize_deposit_transaction_unknown_status() {
        let json = r#"{
    "id": "769800519366885376",
    "amount": "0.001",
    "coin": "BTC",
    "network": "BTC",
    "status": 99,
    "address": "bc1q...",
    "txId": "0x123",
    "insertTime": 1661493146000,
    "confirmTimes": "1/1"
}"#;

        let tx: DepositTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.status, DepositStatus::Unknown(99));
    }

    #[test]
    fn test_deserialize_withdrawal_transaction() {
        let json = r#"{
    "id": "b6ae22b3aa844210a7041aee7589627c",
    "amount": "8.91000000",
    "transactionFee": "0.004",
    "coin": "USDT",
    "status": 6,
    "address": "0x94df...",
    "txId": "0xb7...",
    "applyTime": "2019-10-12 11:12:02",
    "network": "ETH"
}"#;

        let tx: WithdrawalTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.id, "b6ae22b3aa844210a7041aee7589627c");
        assert_eq!(tx.amount, 8.91);
        assert_eq!(tx.transaction_fee, 0.004);
        assert_eq!(tx.coin, "USDT");
        assert_eq!(tx.status, WithdrawStatus::Completed);
        assert_eq!(tx.tx_id, "0xb7...");
        assert_eq!(tx.apply_time, "2019-10-12 11:12:02");
        assert_eq!(tx.network, "ETH");
    }

    #[test]
    fn test_deserialize_withdrawal_transaction_unknown_status() {
        let json = r#"{
    "id": "b6ae22b3aa844210a7041aee7589627c",
    "amount": "8.91000000",
    "transactionFee": "0.004",
    "coin": "USDT",
    "status": 99,
    "address": "0x94df...",
    "txId": "0xb7...",
    "applyTime": "2019-10-12 11:12:02",
    "network": "ETH"
}"#;

        let tx: WithdrawalTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.status, WithdrawStatus::Unknown(99));
    }
}
