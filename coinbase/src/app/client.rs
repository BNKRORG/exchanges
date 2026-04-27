//! Coinbase App client

use reqwest::Response;
use serde::Serialize;

use super::agent::SecureHttpClientAgent;
use super::auth::CoinbaseAuth;
use super::error::Error;
use super::response::{Account, Address, CoinbaseResponse, Transaction};
use crate::app::builder::CoinbaseAppClientBuilder;

const BITCOIN_NETWORK: &str = "bitcoin";
const BTC_CURRENCY_CODE: &str = "BTC";
const WALLET_ACCOUNT_TYPE: &str = "wallet";

#[derive(Debug, Serialize)]
struct CreateAddressRequest<'a> {
    network: &'a str,
}

/// Coinbase App client
#[derive(Debug, Clone)]
pub struct CoinbaseAppClient {
    client: SecureHttpClientAgent,
}

impl CoinbaseAppClient {
    /// Construct a new Coinbase App client.
    pub fn new(auth: CoinbaseAuth) -> Result<Self, Error> {
        Self::builder().auth(auth).build()
    }

    /// Get a new builder
    #[inline]
    pub fn builder() -> CoinbaseAppClientBuilder {
        CoinbaseAppClientBuilder::default()
    }

    #[inline]
    pub(super) fn from_builder(builder: CoinbaseAppClientBuilder) -> Result<Self, Error> {
        Ok(Self {
            client: SecureHttpClientAgent::new(builder.auth, builder.sandbox, builder.timeout)?,
        })
    }

    /// Get accounts
    ///
    /// <https://docs.cdp.coinbase.com/coinbase-app/track-apis/accounts#list-accounts>
    pub async fn accounts(&self) -> Result<Vec<Account>, Error> {
        let mut accounts = Vec::new();

        let mut next_uri: Option<String> = None;

        loop {
            let uri: &str = match &next_uri {
                Some(next_uri) => next_uri.as_str(),
                None => "/v2/accounts",
            };

            let res: Response = self.client.get(uri, Some("limit=100")).await?;

            let res: CoinbaseResponse<Vec<Account>> = res.json().await?;

            accounts.extend(res.data);

            // Check if there is another page
            if let Some(pagination) = res.pagination {
                if let Some(next) = pagination.next_uri {
                    next_uri = Some(next);
                    continue;
                }
            }

            break;
        }

        Ok(accounts)
    }

    /// Get account by ID
    ///
    /// <https://docs.cdp.coinbase.com/coinbase-app/track-apis/accounts#show-account>
    pub async fn account(&self, id: &str) -> Result<Account, Error> {
        let endpoint: String = format!("/v2/accounts/{id}");
        let res: Response = self.client.get(&endpoint, None).await?;
        let res: CoinbaseResponse<Account> = res.json().await?;
        Ok(res.data)
    }

    /// Create a new **bitcoin** deposit address.
    ///
    /// <https://docs.cdp.coinbase.com/coinbase-app/transfer-apis/onchain-addresses#create-address>
    pub async fn bitcoin_deposit_address(&self) -> Result<String, Error> {
        let accounts: Vec<Account> = self.accounts().await?;
        let account_id: &str =
            find_bitcoin_wallet_account_id(&accounts).ok_or(Error::BitcoinWalletAccountNotFound)?;

        let endpoint: String = format!("/v2/accounts/{account_id}/addresses");
        let body: String = serde_json::to_string(&CreateAddressRequest {
            network: BITCOIN_NETWORK,
        })?;

        let res: Response = self.client.post(&endpoint, Some(body)).await?;
        let res: CoinbaseResponse<Address> = res.json().await?;

        if res.data.address.is_empty() {
            return Err(Error::MissingDepositAddress);
        }

        Ok(res.data.address)
    }

    /// Get transactions by account ID
    ///
    /// <https://docs.cdp.coinbase.com/coinbase-app/track-apis/transactions#list-transactions>
    pub async fn transactions(&self, account_id: &str) -> Result<Vec<Transaction>, Error> {
        let mut transactions = Vec::new();

        let mut next_uri: Option<String> = None;

        loop {
            let uri: String =
                next_uri.unwrap_or_else(|| format!("/v2/accounts/{account_id}/transactions"));

            let res: Response = self.client.get(&uri, Some("limit=100")).await?;

            let res: CoinbaseResponse<Vec<Transaction>> = res.json().await?;

            transactions.extend(res.data);

            // Check if there is another page
            if let Some(pagination) = res.pagination {
                if let Some(next) = pagination.next_uri {
                    next_uri = Some(next);
                    continue;
                }
            }

            break;
        }

        Ok(transactions)
    }
}

fn find_bitcoin_wallet_account_id(accounts: &[Account]) -> Option<&str> {
    accounts
        .iter()
        .find(|account| {
            account.currency.code == BTC_CURRENCY_CODE && account.r#type == WALLET_ACCOUNT_TYPE
        })
        .map(|account| account.id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::response::{Balance, Currency};

    fn make_account(id: &str, account_type: &str, currency_code: &str) -> Account {
        Account {
            id: id.to_string(),
            name: "test".to_string(),
            primary: false,
            r#type: account_type.to_string(),
            currency: Currency {
                asset_id: "asset".to_string(),
                code: currency_code.to_string(),
                name: currency_code.to_string(),
            },
            balance: Balance {
                amount: 0.0,
                currency: currency_code.to_string(),
            },
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_find_bitcoin_wallet_account_id() {
        let accounts = vec![
            make_account("fiat-btc", "fiat", "BTC"),
            make_account("eth-wallet", "wallet", "ETH"),
            make_account("btc-wallet", "wallet", "BTC"),
        ];

        assert_eq!(
            find_bitcoin_wallet_account_id(&accounts),
            Some("btc-wallet")
        );
    }

    #[test]
    fn test_find_bitcoin_wallet_account_id_missing() {
        let accounts = vec![
            make_account("fiat-btc", "fiat", "BTC"),
            make_account("eth-wallet", "wallet", "ETH"),
        ];

        assert_eq!(find_bitcoin_wallet_account_id(&accounts), None);
    }
}
