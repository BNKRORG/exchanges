//! Binance responses

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use common::deser::deserialize_string_to_f64;
use serde::Deserialize;

/// Exchange information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    /// Timezone
    pub timezone: String,
    /// Server time
    pub server_time: u64,
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
    pub time: u64,
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
}
