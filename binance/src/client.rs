//! Binance client

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::time::Duration;

use hmac::{Hmac, Mac};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use tokio::sync::OnceCell;
use tokio::time;
use url::Url;

use crate::api::{BinanceApi, Spot};
use crate::auth::BinanceAuth;
use crate::builder::BinanceClientBuilder;
use crate::constant::{BTC_TICKER, MAX_WEIGHT_PER_MIN, USER_AGENT_NAME};
use crate::error::Error;
use crate::response::{AccountInformation, Balance, ExchangeInformation, Symbol, Trade};
use crate::util::build_signed_request;

/// Binance client
#[derive(Clone)]
pub struct BinanceClient {
    client: Client,
    host: Url,
    auth: BinanceAuth,
    recv_window: u64,
    bitcoin_pairs: OnceCell<Vec<Symbol>>,
}

impl fmt::Debug for BinanceClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinanceClient")
            .field("host", &self.host)
            .finish()
    }
}

impl BinanceClient {
    /// Construct a new client
    pub fn new(auth: BinanceAuth) -> Result<Self, Error> {
        Self::builder().auth(auth).build()
    }

    /// Get a new builder
    #[inline]
    pub fn builder() -> BinanceClientBuilder {
        BinanceClientBuilder::default()
    }

    #[inline]
    pub(super) fn from_builder(builder: BinanceClientBuilder) -> Result<Self, Error> {
        Ok(Self {
            client: Client::builder()
                .user_agent(USER_AGENT_NAME)
                .timeout(builder.timeout)
                .build()?,
            host: builder.endpoint.into_url(),
            auth: builder.auth,
            recv_window: builder.recv_window,
            bitcoin_pairs: OnceCell::new(),
        })
    }

    fn sign_request(&self, api: &BinanceApi, request: Option<String>) -> Result<Url, Error> {
        let secret_key: &str = self.auth.get_api_secret_key()?;

        let mut signed_key = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).unwrap();

        if let Some(request) = &request {
            signed_key.update(request.as_bytes());
        }

        let signature = hex::encode(signed_key.finalize().into_bytes());

        let request_body: String = match request {
            Some(request) => format!("{request}&signature={signature}"),
            None => format!("signature={signature}"),
        };

        // Build URL endpoint
        let mut url: Url = self.host.join(api.http_path())?;

        // Add query parameters
        url.set_query(Some(&request_body));

        Ok(url)
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap, Error> {
        let api_key: &str = self.auth.get_api_key()?;

        let mut custom_headers = HeaderMap::new();

        if content_type {
            custom_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        custom_headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(api_key)?,
        );

        Ok(custom_headers)
    }

    async fn handle_http_response<T>(&self, response: Response) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let response: Response = response.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn get<T>(&self, api: BinanceApi, request: Option<String>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        // Build URL endpoint
        let mut url: Url = self.host.join(api.http_path())?;

        if let Some(request) = request {
            if !request.is_empty() {
                url.set_query(Some(&request));
            }
        }

        let req = self.client.get(url);

        self.send_req(req, api.request_weight()).await
    }

    async fn get_signed<T>(&self, api: BinanceApi, request: Option<String>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = self.sign_request(&api, request)?;
        let headers = self.build_headers(true)?;
        let req = self.client.get(url).headers(headers);

        self.send_req(req, api.request_weight()).await
    }

    async fn send_req<T>(&self, req: RequestBuilder, request_weight: u32) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        loop {
            // Try to clone the request builder
            let req: RequestBuilder = req.try_clone().ok_or(Error::CantCloneRequest)?;

            // Send the request
            let response: Response = req.send().await?;

            // Extract weight header
            let used_weight: u32 = response
                .headers()
                .get("X-MBX-USED-WEIGHT-1M")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let available: u32 = MAX_WEIGHT_PER_MIN.saturating_sub(used_weight);

            if available >= request_weight {
                // Safe → parse and return response
                return self.handle_http_response(response).await;
            }

            // Need to slow down
            let deficit: u32 = request_weight - available;

            // Compute proportional wait (rolling window)
            // Example: deficit=100, max=6000 → sleep for 100/6000 minute
            let sleep_ms: u64 = (deficit as f64 / MAX_WEIGHT_PER_MIN as f64 * 60_000.0) as u64;

            // Minimum sleep of 200 ms to avoid thrashing
            let sleep_ms: u64 = sleep_ms.max(200);

            tracing::warn!(
                "Rate limit near! used={} available={} deficit={}. Sleeping {} ms",
                used_weight,
                available,
                deficit,
                sleep_ms
            );

            time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    }

    /// Get exchange information
    pub async fn exchange_info(&self) -> Result<ExchangeInformation, Error> {
        self.get(BinanceApi::Spot(Spot::ExchangeInfo), None).await
    }

    /// Get account information
    pub async fn get_account(&self) -> Result<AccountInformation, Error> {
        // Build signed request
        let request: String = build_signed_request(BTreeMap::new(), self.recv_window)?;

        // Get signed request
        self.get_signed(BinanceApi::Spot(Spot::Account), Some(request))
            .await
    }

    /// Get balance for a specific asset (i.e., "BTC")
    pub async fn balance_for_asset<S>(&self, asset: S) -> Result<Balance, Error>
    where
        S: AsRef<str>,
    {
        let account: AccountInformation = self.get_account().await?;

        let asset: &str = asset.as_ref();

        // Find the balance for the given asset
        for balance in account.balances.into_iter() {
            if balance.asset == asset {
                return Ok(balance);
            }
        }

        Err(Error::AssetNotFound)
    }

    /// Get **bitcoin** balance
    #[inline]
    pub async fn balance(&self) -> Result<Balance, Error> {
        self.balance_for_asset(BTC_TICKER).await
    }

    async fn bitcoin_pairs(&self) -> Result<&Vec<Symbol>, Error> {
        self.bitcoin_pairs
            .get_or_try_init(|| async {
                // Get exchange info
                let info = self.exchange_info().await?;

                // Filter paris
                let btc_pairs: Vec<Symbol> = info
                    .symbols
                    .into_iter()
                    .filter(|s| s.base_asset == BTC_TICKER || s.quote_asset == BTC_TICKER)
                    .collect();

                Ok(btc_pairs)
            })
            .await
    }

    /// Get trades for a specific symbol (i.e., "BTCUSDT")
    pub async fn trade_history_for_pair<S>(&self, symbol: S) -> Result<Vec<Trade>, Error>
    where
        S: Into<String>,
    {
        let mut parameters = BTreeMap::new();
        parameters.insert(String::from("symbol"), symbol.into());

        // Build signed request
        let request: String = build_signed_request(parameters, self.recv_window)?;

        // Get signed request
        self.get_signed(BinanceApi::Spot(Spot::MyTrades), Some(request))
            .await
    }

    /// Get trades for all **bitcoin** pairs
    pub async fn trade_history(&self) -> Result<HashMap<Symbol, Vec<Trade>>, Error> {
        // Get all bitcoin pairs
        let btc_pairs = self.bitcoin_pairs().await?.clone();

        let mut output = HashMap::with_capacity(btc_pairs.len());

        for pair in btc_pairs {
            let trades = self.trade_history_for_pair(pair.symbol.clone()).await?;

            output.insert(pair, trades);
        }

        Ok(output)
    }
}
