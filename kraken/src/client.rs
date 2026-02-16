//! Kraken client

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::{self, KrakenAuth};
use crate::constant::{API_ROOT_URL, API_VERSION, USER_AGENT_NAME, XBT_TICKER};
use crate::error::Error;
use crate::request::{
    DepositStatus, Empty, GetTradesHistory, KrakenRequestBody, Request, WithdrawStatus,
};
use crate::response::{
    BitcoinBalances, DepositTransaction, KrakenResult, Trade, TradesHistory, WithdrawTransaction,
};

enum Api<'a> {
    Balance,
    DepositStatus {
        /// Currency to get transactions for.
        asset: Option<&'a str>,
    },
    WithdrawStatus {
        /// Currency to get transactions for.
        asset: Option<&'a str>,
    },
    TradesHistory {
        start: Option<u64>,
        end: Option<u64>,
    },
}

impl Api<'_> {
    fn method(&self) -> &str {
        match self {
            Self::Balance => "Balance",
            Self::DepositStatus { .. } => "DepositStatus",
            Self::WithdrawStatus { .. } => "WithdrawStatus",
            Self::TradesHistory { .. } => "TradesHistory",
        }
    }

    fn body(&self) -> Request {
        match self {
            Self::Balance => Request::Empty(Empty {}),
            Self::DepositStatus { asset } => Request::DepositStatus(DepositStatus {
                asset: asset.as_deref(),
            }),
            Self::WithdrawStatus { asset } => Request::WithdrawStatus(WithdrawStatus {
                asset: asset.as_deref(),
            }),
            Self::TradesHistory { start, end } => Request::TradesHistory(GetTradesHistory {
                r#type: "closed position",
                trades: true,
                start: *start,
                end: *end,
                ofs: None,
            }),
        }
    }
}

/// Kraken client
#[derive(Debug, Clone)]
pub struct KrakenClient {
    /// Root URL for the API.
    root_url: Url,
    /// HTTP client.
    client: Client,
    /// Authentication
    auth: KrakenAuth,
}

impl KrakenClient {
    /// Construct a new client.
    pub fn new(auth: KrakenAuth) -> Result<Self, Error> {
        Ok(Self {
            root_url: Url::parse(API_ROOT_URL)?,
            client: Client::builder()
                .user_agent(USER_AGENT_NAME)
                .timeout(Duration::from_secs(25))
                .build()?,
            auth,
        })
    }

    async fn query<T>(&self, url: Url, headers: HeaderMap, body_json: String) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        // Post request
        let response: Response = self
            .client
            .post(url)
            .headers(headers)
            .body(body_json)
            .send()
            .await?;

        // If HTTP error, return error
        let response: Response = response.error_for_status()?;

        // Parse the response as JSON
        let result: KrakenResult<T> = response.json().await?;

        // Extract the result
        result.extract()
    }

    async fn query_private<T>(&self, api: Api<'_>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        match &self.auth {
            KrakenAuth::ApiKeys(creds) => {
                let method: &str = api.method();

                let path: String = format!("/{API_VERSION}/private/{method}");
                let url: Url = self.root_url.join(&path)?;

                // Construct body data
                let body: KrakenRequestBody = KrakenRequestBody {
                    nonce: auth::nonce(),
                    request: api.body(),
                };

                // Sign the request
                let (body_json, sig) = auth::sign_api(creds, &url, body)?;

                // Build headers
                let mut headers: HeaderMap = HeaderMap::with_capacity(2);
                headers.insert("API-Key", HeaderValue::from_str(&creds.key)?);
                headers.insert("API-Sign", HeaderValue::from_str(&sig)?);
                headers.insert("Content-Type", HeaderValue::from_static("application/json"));

                // Query
                self.query(url, headers, body_json).await
            }
            KrakenAuth::None => Err(Error::MissingCredentials),
        }
    }

    /// Get **bitcoin** balance.
    pub async fn balance(&self) -> Result<f64, Error> {
        // Get bitcoin balances
        let balances: BitcoinBalances = self.query_private(Api::Balance).await?;

        // Sum balances
        Ok(balances.sum())
    }

    /// Get **bitcoin** deposit transactions.
    pub async fn deposit_transactions(&self) -> Result<Vec<DepositTransaction>, Error> {
        self.query_private(Api::DepositStatus {
            asset: Some(XBT_TICKER),
        })
        .await
    }

    /// Get **bitcoin** withdraw transactions.
    pub async fn withdraw_transactions(&self) -> Result<Vec<WithdrawTransaction>, Error> {
        self.query_private(Api::WithdrawStatus {
            asset: Some(XBT_TICKER),
        })
        .await
    }

    /// Get **bitcoin** trade history.
    pub async fn trade_history(&self) -> Result<Vec<Trade>, Error> {
        let history: TradesHistory = self
            .query_private(Api::TradesHistory {
                start: None,
                end: None,
            })
            .await?;

        // Filter for trades involving Bitcoin (checking if pair contains the ticker)
        let trades: Vec<Trade> = history
            .trades
            .into_values()
            .filter(|trade| trade.pair.contains(XBT_TICKER))
            .collect();

        Ok(trades)
    }
}
