//! Coinbase App client

use reqwest::Response;

use super::agent::SecureHttpClientAgent;
use super::auth::CoinbaseAuth;
use super::error::Error;
use super::response::{Account, CoinbaseResponse, Transaction};
use crate::app::builder::CoinbaseAppClientBuilder;

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
