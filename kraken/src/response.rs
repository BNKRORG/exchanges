//! Kraken API responses

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de};

use crate::constant::TICKERS;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub(crate) struct KrakenResult<T> {
    /// Kraken API returns error strings in an array marked "error"
    error: Vec<String>,
    /// Kraken API returns results here, separated from error
    /// Sometimes result is omitted if errors occured.
    result: Option<T>,
}

impl<T> KrakenResult<T> {
    pub(crate) fn extract(self) -> Result<T, Error> {
        if !self.error.is_empty() {
            return Err(Error::Kraken(self.error));
        }

        self.result.ok_or(Error::MissingResult)
    }
}

/// Bitcoin balances
///
/// This struct deserializes ONLY the bitcoin balances (see [`TICKERS`]).
pub(crate) struct BitcoinBalances(HashMap<String, f64>);

impl BitcoinBalances {
    /// Get the sum of the bitcoin balances
    #[inline]
    pub(crate) fn sum(self) -> f64 {
        self.0.values().sum()
    }
}

impl<'de> Deserialize<'de> for BitcoinBalances {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Kraken returns the balances as string
        let map: HashMap<String, String> = Deserialize::deserialize(deserializer)?;

        // HashMap to store the parsed balances
        let mut balances: HashMap<String, f64> = HashMap::new();

        for (coin, amount) in map.into_iter() {
            // If the ticker is NOT a bitcoin ticker, skip it.
            if !TICKERS.contains(coin.as_str()) {
                continue;
            }

            // Convert to f64
            let amount: f64 = amount.parse().map_err(de::Error::custom)?;

            // Insert into the balances map
            balances.insert(coin, amount);
        }

        Ok(Self(balances))
    }
}

/// Transaction status
///
/// <https://github.com/globalcitizen/ifex-protocol/blob/master/draft-ifex-00.txt#L837>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum TransactionStatus {
    /// Initial
    #[serde(alias = "initial", alias = "INITIAL")]
    Initial,
    /// Pending
    #[serde(alias = "pending", alias = "PENDING")]
    Pending,
    /// Settled
    #[serde(alias = "settled", alias = "SETTLED")]
    Settled,
    /// Success
    #[serde(alias = "success", alias = "SUCCESS")]
    Success,
    /// Failure
    #[serde(alias = "failure", alias = "FAILURE")]
    Failure,
}

/// Deposit transaction
#[derive(Debug, Deserialize)]
pub struct DepositTransaction {
    /// Reference ID
    #[serde(rename = "refid")]
    pub id: String,
    /// Asset
    pub asset: String,
    /// Asset class
    #[serde(rename = "aclass")]
    pub class: String,
    /// Name of deposit method
    pub method: String,
    /// Method transaction ID
    pub txid: String,
    /// Method transaction information
    pub info: String,
    /// Amount deposited
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Fees paid
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub fee: f64,
    /// Unix timestamp when request was made
    pub time: u64,
    /// Status of deposit
    pub status: TransactionStatus,
}

/// Withdraw transaction
#[derive(Debug, Deserialize)]
pub struct WithdrawTransaction {
    /// Reference ID
    #[serde(rename = "refid")]
    pub id: String,
    /// Asset
    pub asset: String,
    /// Asset class
    #[serde(rename = "aclass")]
    pub class: String,
    /// Name of withdrawal method
    pub method: String,
    /// Network name based on the funding method used
    pub network: String,
    /// Method transaction ID
    pub txid: String,
    /// Method transaction information
    pub info: String,
    /// Amount deposited
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Fees paid
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub fee: f64,
    /// Unix timestamp when request was made
    pub time: u64,
    /// Status of withdraw
    pub status: TransactionStatus,
}

/// Trades history
#[derive(Debug, Deserialize)]
pub(crate) struct TradesHistory {
    /// Trades
    pub trades: HashMap<String, Trade>,
    // /// Count of trades
    // pub count: usize,
}

/// Trade type
#[derive(Debug, Deserialize)]
pub enum TrateType {
    /// Buy
    #[serde(rename = "buy")]
    Buy,
    /// Sell
    #[serde(rename = "sell")]
    Sell,
}

/// Trade history trade entry
#[derive(Debug, Deserialize)]
pub struct Trade {
    /// Unique identifier of trade executed
    #[serde(rename = "trade_id")]
    pub id: String,
    /// Order responsible for execution of trade
    #[serde(rename = "ordertxid")]
    pub order_txid: String,
    /// Position responsible for execution of trade
    #[serde(rename = "postxid")]
    pub pos_txid: String,
    /// Asset pair
    pub pair: String,
    /// Unix timestamp of trade
    pub time: u64,
    /// Type of order (buy/sell)
    #[serde(rename = "type")]
    pub r#type: TrateType,
    /// Order type
    #[serde(rename = "ordertype")]
    pub order_type: String,
    /// Average price order was executed at (quote currency)
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
    /// Total cost of order (quote currency)
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub cost: f64,
    /// Total fee (quote currency)
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub fee: f64,
    /// Volume (base currency)
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub vol: f64,
}

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse().map_err(de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balances_deserialize_and_sum() {
        let json = r#"{
            "XBT": "1.5",
            "XBT.F": "2.3",
            "ETH": "10.0",
            "USD": "1000.50"
        }"#;

        let balances: BitcoinBalances = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(balances.0.len(), 2);
        assert!(balances.0.contains_key("XBT"));
        assert!(balances.0.contains_key("XBT.F"));
        assert!(!balances.0.contains_key("ETH"));
        assert!(!balances.0.contains_key("USD"));

        // Check sum
        let sum = balances.sum();
        assert!((sum - 3.8).abs() < 0.0001);
    }

    #[test]
    fn test_balances_parse_error() {
        let json = r#"{
            "XBT": "invalid_number"
        }"#;

        let result: Result<BitcoinBalances, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_balances_empty() {
        let json = r#"{
            "ETH": "10.0",
            "USD": "1000.50"
        }"#;

        let balances: BitcoinBalances = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(balances.0.len(), 0);
        assert_eq!(balances.sum(), 0.0);
    }

    #[test]
    fn test_deposit_transaction_deserialization() {
        let json = r#"{"aclass": "currency", "amount": "0.0000500000", "asset": "XXBT", "fee": "0.0000000000", "info": "lnbc50u1p5w0uh4pp5zm5h54cfsfan258hx5yxejm6hj28nakdwmjwycnfdlq00fgcq3wqdqhfdexz6m9dcsygetsdaekjaqcqzysxqrrsssp5t6zvny0j826dgxahpuyzzhk9m9n2m75zj9wnxy396rlxcuxd462s9qxpqysgquuygd682k3t6dq7wmw7amt00fghaqqpny22l8ssakcjts53jwe882hskaq4zeydpwfks0u47l5zzxk0hyg049wrgwv5fw067kzptd5gqmg30y6", "method": "Bitcoin Lightning", "refid": "FTKo1pI-55ynnZ4GFwca8XsAIjxqpl", "status": "Success", "time": 1760031475, "txid": "16e97a5709827b3550f735086ccb7abc9479f6cd76e4e262696fc0f7a518045c"}"#;

        let tx: DepositTransaction = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(tx.class, "currency");
        assert_eq!(tx.amount, 0.00005);
        assert_eq!(tx.fee, 0.0);
        assert_eq!(tx.method, "Bitcoin Lightning");
        assert_eq!(tx.status, TransactionStatus::Success);
        assert_eq!(tx.time, 1760031475);
    }
}
