//! Strike client

use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Method, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::auth::StrikeAuth;
use crate::constant::{API_ROOT_URL, BTC_TICKER, USER_AGENT_NAME};
use crate::error::Error;
use crate::response::{Balance, Deposit, Deposits, Invoice, Invoices, ReceiveRequest};

#[derive(Debug, Serialize)]
struct ReceiveRequestParams<'a> {
    onchain: EmptyObject,
    #[serde(rename = "targetCurrency")]
    target_currency: &'a str,
}

#[derive(Debug, Serialize)]
struct EmptyObject {}

enum Api {
    Balances,
    Deposits,
    Invoices,
    ReceiveRequests,
}

impl Api {
    fn url_path(&self) -> &str {
        match self {
            Self::Balances => "/v1/balances",
            Self::Deposits => "/v1/deposits",
            Self::Invoices => "/v1/invoices",
            Self::ReceiveRequests => "/v1/receive-requests",
        }
    }

    fn http_method(&self) -> Method {
        match self {
            Self::Balances => Method::GET,
            Self::Deposits => Method::GET,
            Self::Invoices => Method::GET,
            Self::ReceiveRequests => Method::POST,
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

    async fn call_api<T>(&self, api: Api, body: Option<String>) -> Result<T, Error>
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
        let mut request = self.client.request(method, url).headers(headers);

        if let Some(body) = body {
            request = request.body(body);
        }

        let response: Response = request.send().await?;

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
        let balances: Vec<Balance> = self.call_api(Api::Balances, None).await?;

        // Find balance for BTC
        let balance: Balance = balances
            .into_iter()
            .find(|b| b.currency == BTC_TICKER)
            .unwrap_or_else(|| Balance::new(BTC_TICKER));

        Ok(balance)
    }

    /// Get a **bitcoin** deposit address.
    ///
    /// <https://docs.strike.me/api/create-a-receive-request/>
    pub async fn bitcoin_deposit_address(&self) -> Result<String, Error> {
        let body: String = serde_json::to_string(&ReceiveRequestParams {
            onchain: EmptyObject {},
            target_currency: BTC_TICKER,
        })?;

        let request: ReceiveRequest = self.call_api(Api::ReceiveRequests, Some(body)).await?;

        let onchain = request.onchain.ok_or(Error::MissingDepositAddress)?;

        if onchain.address.is_empty() {
            return Err(Error::MissingDepositAddress);
        }

        Ok(onchain.address)
    }

    /// Get **bitcoin** deposits.
    ///
    /// <https://docs.strike.me/api/get-deposits>
    pub async fn deposits(&self) -> Result<Vec<Deposit>, Error> {
        // Get deposits
        let deposits: Deposits = self.call_api(Api::Deposits, None).await?;

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
        let invoices: Invoices = self.call_api(Api::Invoices, None).await?;

        // Filter bitcoin invoices
        let invoices: Vec<Invoice> = invoices
            .items
            .into_iter()
            .filter(|b| b.amount.currency == BTC_TICKER)
            .collect();

        Ok(invoices)
    }
}
