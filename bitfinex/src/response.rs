//! Bitfinex responses

use serde::Deserialize;
use serde_json::{Map, Value};

/// Bitfinex wallet
///
/// <https://docs.bitfinex.com/reference/rest-auth-wallets>
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "WalletArray")]
pub struct Wallet {
    /// Wallet type
    pub r#type: String,
    /// Currency
    pub currency: String,
    /// Balance
    pub balance: f64,
    /// Unsettled interest
    pub unsettled_interest: f64,
    /// Wallet balance available for orders/withdrawal/transfer
    pub available_balance: f64,
    /// Description of the last ledger entry
    pub last_change: String,
    /// Optional object with details
    pub last_change_metadata: Map<String, Value>,
}

impl From<WalletArray> for Wallet {
    fn from(arr: WalletArray) -> Self {
        Wallet {
            r#type: arr.0,
            currency: arr.1,
            balance: arr.2,
            unsettled_interest: arr.3,
            available_balance: arr.4,
            last_change: arr.5,
            last_change_metadata: arr.6,
        }
    }
}

#[derive(Deserialize)]
struct WalletArray(
    String,             // type
    String,             // currency
    f64,                // balance
    f64,                // unsettled_interest
    f64,                // available_balance
    String,             // last_change
    Map<String, Value>, // trade_details
);

/// Bitfinex movement (Deposit/Withdrawal)
///
/// <https://docs.bitfinex.com/reference/rest-auth-movements>
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "MovementArray")]
pub struct Movement {
    /// Movement identifier
    pub id: u64,
    /// The symbol of the currency (ex. "BTC")
    pub currency: String,
    /// The extended name of the currency (ex. "BITCOIN")
    pub currency_name: String,
    /// Movement started at
    pub mts_started: u64,
    /// Movement last updated at
    pub mts_updated: u64,
    /// Current status
    pub status: String,
    /// Amount of funds moved (positive for deposits, negative for withdrawals)
    pub amount: f64,
    /// Tx Fees applied
    pub fees: f64,
    /// ///Destination address
    pub destination_address: String,
    /// Payment ID (if relevant)
    pub payment_id: Option<String>,
    /// Transaction identifier
    pub transaction_id: String,
    /// Optional personal withdraw transaction note
    pub withdraw_transaction_note: Option<String>,
}

impl From<MovementArray> for Movement {
    fn from(arr: MovementArray) -> Self {
        Movement {
            id: arr.0,
            currency: arr.1,
            currency_name: arr.2,
            mts_started: arr.5,
            mts_updated: arr.6,
            status: arr.9,
            amount: arr.12,
            fees: arr.13,
            destination_address: arr.16,
            payment_id: arr.17,
            transaction_id: arr.20,
            withdraw_transaction_note: arr.21,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct MovementArray(
    u64,            // ID
    String,         // CURRENCY
    String,         // CURRENCY_NAME
    Option<Value>,  // PLACEHOLDER
    Option<Value>,  // PLACEHOLDER
    u64,            // MTS_STARTED
    u64,            // MTS_UPDATED
    Option<Value>,  // PLACEHOLDER
    Option<Value>,  // PLACEHOLDER
    String,         // STATUS
    Option<Value>,  // PLACEHOLDER
    Option<Value>,  // PLACEHOLDER
    f64,            // AMOUNT
    f64,            // FEES
    Option<Value>,  // PLACEHOLDER
    Option<Value>,  // PLACEHOLDER
    String,         // DESTINATION_ADDRESS
    Option<String>, // PAYMENT_ID
    Option<Value>,  // PLACEHOLDER
    Option<Value>,  // PLACEHOLDER
    String,         // TRANSACTION_ID
    Option<String>, // WITHDRAW_TRANSACTION_NOTE
);

/// Bitfinex executed trade
///
/// <https://docs.bitfinex.com/reference/rest-auth-trades>
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "TradeArray")]
pub struct Trade {
    /// Trade database id
    pub id: u64,
    /// Symbol
    pub symbol: String,
    /// Execution timestamp
    pub timestamp: u64,
    /// Order id
    pub order_id: u64,
    /// Positive means buy, negative means sell
    pub amount: f64,
    /// Execution price
    pub price: f64,
    /// Order type
    pub order_type: String,
    /// Order price
    pub order_price: f64,
    /// Whether the trade was a maker
    pub is_maker: bool,
    /// Fee
    pub fee: f64,
    /// Fee currency
    pub fee_currency: String,
    /// Client Order ID
    pub cid: Option<u64>,
}

impl From<TradeArray> for Trade {
    fn from(arr: TradeArray) -> Self {
        Trade {
            id: arr.0,
            symbol: arr.1,
            timestamp: arr.2,
            order_id: arr.3,
            amount: arr.4,
            price: arr.5,
            order_type: arr.6,
            order_price: arr.7,
            is_maker: arr.8 == 1,
            fee: arr.9,
            fee_currency: arr.10,
            cid: arr.11,
        }
    }
}

#[derive(Deserialize)]
struct TradeArray(
    u64,         // ID
    String,      // SYMBOL
    u64,         // MTS
    u64,         // ORDER_ID
    f64,         // EXEC_AMOUNT
    f64,         // EXEC_PRICE
    String,      // ORDER_TYPE
    f64,         // ORDER_PRICE
    i8,          // MAKER
    f64,         // FEE
    String,      // FEE_CURRENCY
    Option<u64>, // CID
);

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_wallet_deserialization() {
        let json = r#"["exchange","UST",19788.6529257,0,19788.6529257,"Exchange 2.0 UST for USD @ 11.696",{
  		"reason": "TRADE",
  		"order_id": 1189740779,
  		"order_id_oppo": 1189785673,
  		"trade_price": "11.696",
  		"trade_amount": "-2.0",
  		"order_cid": 1598516362757,
  		"order_gid": 1598516362629
  	}
  ]"#;

        let wallet: Wallet = serde_json::from_str(json).unwrap();

        let mut expected_metadata = Map::new();
        expected_metadata.insert("reason".to_string(), json!("TRADE"));
        expected_metadata.insert("order_id".to_string(), json!(1189740779));
        expected_metadata.insert("order_id_oppo".to_string(), json!(1189785673));
        expected_metadata.insert("trade_price".to_string(), json!("11.696"));
        expected_metadata.insert("trade_amount".to_string(), json!("-2.0"));
        expected_metadata.insert("order_cid".to_string(), json!(1598516362757u64));
        expected_metadata.insert("order_gid".to_string(), json!(1598516362629u64));

        assert_eq!(
            wallet,
            Wallet {
                r#type: String::from("exchange"),
                currency: String::from("UST"),
                balance: 19788.6529257,
                unsettled_interest: 0.0,
                available_balance: 19788.6529257,
                last_change: String::from("Exchange 2.0 UST for USD @ 11.696"),
                last_change_metadata: expected_metadata
            }
        );
    }

    #[test]
    fn test_movement_deserialization() {
        let json = r#"[
            13293039,
            "BTC",
            "BITCOIN",
            null,
            null,
            1574175052000,
            1574181326000,
            null,
            null,
            "CANCELED",
            null,
            null,
            -0.24,
            -0.00135,
            null,
            null,
            "DESTINATION_ADDRESS",
            null,
            null,
            null,
            "TRANSACTION_ID",
            "Purchase of 10000 pizzas"
        ]"#;

        let movement: Movement = serde_json::from_str(json).unwrap();

        assert_eq!(
            movement,
            Movement {
                id: 13293039,
                currency: String::from("BTC"),
                currency_name: String::from("BITCOIN"),
                mts_started: 1574175052000,
                mts_updated: 1574181326000,
                status: String::from("CANCELED"),
                amount: -0.24,
                fees: -0.00135,
                destination_address: String::from("DESTINATION_ADDRESS"),
                payment_id: None,
                transaction_id: String::from("TRANSACTION_ID"),
                withdraw_transaction_note: Some(String::from("Purchase of 10000 pizzas")),
            }
        );
    }

    #[test]
    fn test_trade_deserialization() {
        let json = r#"[
                402088407,
                "tBTCUST",
                1574963975602,
                34938060782,
                -0.2,
                153.57,
                "MARKET",
                0.0,
                -1,
                -0.061668,
                "USD",
                1234
            ]"#;

        let trade: Trade = serde_json::from_str(json).unwrap();

        assert_eq!(
            trade,
            Trade {
                id: 402088407,
                symbol: String::from("tBTCUST"),
                timestamp: 1574963975602,
                order_id: 34938060782,
                amount: -0.2,
                price: 153.57,
                order_type: String::from("MARKET"),
                order_price: 0.0,
                is_maker: false,
                fee: -0.061668,
                fee_currency: String::from("USD"),
                cid: Some(1234),
            }
        );
    }
}
