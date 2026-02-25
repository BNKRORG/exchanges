//! Strike responses

use chrono::{DateTime, Utc};
use common::deser::deserialize_string_to_f64;
use serde::Deserialize;

/// Strike balance
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Balance {
    /// The currency of the balance
    pub currency: String,
    /// The balance that is currently available for trading (currency conversion). Includes settled and any non spent pending amount
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub current: f64,
    /// The balance of all deposits, in all currencies, that are pending settlement denominated in this currency. This number impacts the available balance and can cause the available balance to be lower than the current balance
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub pending: f64,
    /// The balance currently being withdrawn
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub outgoing: f64,
    /// The balance that is reserved to be spent. E.g. for pending withdrawals, target orders, etc.
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub reserved: f64,
    /// The balance that is available to be spent and sent out without any restrictions. Available balance might be lower than the current because of the pending deposits
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub available: f64,
    /// The sum of the available and outgoing balance
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub total: f64,
}

impl Balance {
    pub(crate) fn new<T>(currency: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            currency: currency.into(),
            current: 0.0,
            pending: 0.0,
            outgoing: 0.0,
            reserved: 0.0,
            available: 0.0,
            total: 0.0,
        }
    }
}

/// Strike amount
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StrikeAmount {
    /// Amount
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub amount: f64,
    /// Currency
    pub currency: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Deposits {
    pub items: Vec<Deposit>,
}

/// State of a deposit
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositState {
    /// New deposit
    New,
    /// Deposit is being processed
    Pending,
    /// Deposit has been processed
    Completed,
    /// Deposit has been reversed
    Reversed,
    /// Deposit failed
    Failed,
}

/// Strike deposit
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
    /// The ID of the deposit
    pub id: String,
    /// The amount to be credited to the Strike account.
    pub amount: StrikeAmount,
    /// The amount to be charged as the fee for the deposit.
    pub fee: StrikeAmount,
    /// The total amount to be deposited from the payment method.
    pub total_amount: StrikeAmount,
    /// The status of the deposit (e.g. "COMPLETED", "PENDING", "FAILED")
    pub state: DepositState,
    /// The reason for the deposit failure.
    pub failure_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Invoices {
    pub items: Vec<Invoice>,
}

/// State of an invoice
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceState {
    /// The invoice has not been paid yet
    Unpaid,
    /// Invoice is being processed
    Pending,
    /// Invoice has been paid
    Paid,
    /// Invoice has been canceled
    Cancelled,
}

/// Strike invoice
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    /// The ID of the invoice
    pub invoice_id: String,
    /// The invoice acount
    pub amount: StrikeAmount,
    /// The status of the invoice
    pub state: InvoiceState,
    /// Timestamp when the invoice was created
    pub created: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_invoice() {
        let json = r#"{
 "invoiceId": "6b91e56d-fce9-4eec-995f-1d08fe6ba380",
 "amount": {
   "amount": "150.00",
   "currency": "USD"
 },
 "state": "UNPAID",
 "created": "2021-11-12T20:08:45.98159+00:00",
 "correlationId": "224bff37-021f-43e5-9b9c-390e3d834750",
 "description": "Invoice for order 123",
 "issuerId": "bf909224-3432-400b-895a-3010302f80f5",
 "receiverId": "bf909224-3432-400b-895a-3010302f80f5"
}"#;

        let invoice: Invoice = serde_json::from_str(json).unwrap();

        assert_eq!(invoice.invoice_id, "6b91e56d-fce9-4eec-995f-1d08fe6ba380");
        assert_eq!(invoice.amount.amount, 150.0);
        assert_eq!(invoice.amount.currency, "USD");
        assert_eq!(invoice.state, InvoiceState::Unpaid);
        assert_eq!(invoice.created.timestamp(), 1636747725);
    }
}
