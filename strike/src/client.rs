//! Strike client

use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::StrikeAuth;
use crate::constant::{API_ROOT_URL, BTC_TICKER, USER_AGENT_NAME};
use crate::error::Error;
use crate::response::{Balance, Deposit, Deposits, Invoice, Invoices};

enum Api {
    Balances,
    Deposits,
    Invoices,
}

impl Api {
    fn url_path(&self) -> &str {
        match self {
            Self::Balances => "/v1/balances",
            Self::Deposits => "/v1/deposits",
            Self::Invoices => "/v1/invoices",
        }
    }

    fn http_method(&self) -> Method {
        match self {
            Self::Balances => Method::GET,
            Self::Deposits => Method::GET,
            Self::Invoices => Method::GET,
        }
    }
}

/// Strike client
#[derive(Debug, Clone)]
pub struct StrikeClient {
    /// Root URL for the API.
    root_url: Url,
    /// HTTP client.
    client: Client,
    /// Authentication
    auth: StrikeAuth,
}

impl StrikeClient {
    /// Construct a new client.
    pub fn new(auth: StrikeAuth) -> Result<Self, Error> {
        Ok(Self {
            root_url: Url::parse(API_ROOT_URL)?,
            client: Client::builder()
                .user_agent(USER_AGENT_NAME)
                .timeout(Duration::from_secs(25))
                .build()?,
            auth,
        })
    }

    async fn call_api<T>(&self, api: Api) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url: Url = self.root_url.join(api.url_path())?;
        let method: Method = api.http_method();

        // Build headers
        let mut headers: HeaderMap = HeaderMap::with_capacity(2);
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        match &self.auth {
            StrikeAuth::ApiKey(key) => {
                let val: String = format!("Bearer {key}");

                let mut value: HeaderValue = HeaderValue::from_str(&val)?;
                value.set_sensitive(true);

                headers.insert(AUTHORIZATION, value);
            }
        }

        // Send request
        let response: Response = self
            .client
            .request(method, url)
            .headers(headers)
            .send()
            .await?;

        // Propagate error if any
        let response: Response = response.error_for_status()?;

        // Deserialize response
        Ok(response.json().await?)
    }

    /// Get **bitcoin** balance.
    ///
    /// <https://docs.strike.me/api/get-account-balance-details/>
    pub async fn balance(&self) -> Result<Balance, Error> {
        // Get balances
        let balances: Vec<Balance> = self.call_api(Api::Balances).await?;

        // Find balance for BTC
        let balance: Balance = balances
            .into_iter()
            .find(|b| b.currency == BTC_TICKER)
            .unwrap_or_else(|| Balance::new(BTC_TICKER));

        Ok(balance)
    }

    /// Get **bitcoin** deposits.
    ///
    /// <https://docs.strike.me/api/get-deposits>
    pub async fn deposits(&self) -> Result<Vec<Deposit>, Error> {
        // Get deposits
        let deposits: Deposits = self.call_api(Api::Deposits).await?;

        // Filter bitcoin deposits
        let deposits: Vec<Deposit> = deposits
            .items
            .into_iter()
            .filter(|b| b.amount.currency == BTC_TICKER)
            .collect();

        Ok(deposits)
    }

    /// Get **bitcoin** invoices.
    ///
    /// <https://docs.strike.me/api/get-invoices>
    pub async fn invoices(&self) -> Result<Vec<Invoice>, Error> {
        // Get invoices
        let invoices: Invoices = self.call_api(Api::Invoices).await?;

        // Filter bitcoin invoices
        let invoices: Vec<Invoice> = invoices
            .items
            .into_iter()
            .filter(|b| b.amount.currency == BTC_TICKER)
            .collect();

        Ok(invoices)
    }
}
