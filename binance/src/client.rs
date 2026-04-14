//! Binance client

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::time::Duration;

use hmac::{Hmac, Mac};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
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
            let used_weight: u32 = used_weight_1m(response.headers());
            let status: StatusCode = response.status();

            if status == StatusCode::TOO_MANY_REQUESTS {
                let sleep: Duration = retry_after_ms(response.headers())
                    .or_else(|| throttle_delay(used_weight, request_weight))
                    .unwrap_or_else(|| Duration::from_millis(200));

                tracing::warn!(
                    "Rate limit hit (429)! used={}. Sleeping {} ms before retry",
                    used_weight,
                    sleep.as_millis()
                );

                time::sleep(sleep).await;

                continue;
            }

            if let Some(sleep) = throttle_delay(used_weight, request_weight) {
                let available: u32 = MAX_WEIGHT_PER_MIN.saturating_sub(used_weight);
                let deficit: u32 = used_weight
                    .saturating_add(request_weight)
                    .saturating_sub(MAX_WEIGHT_PER_MIN);

                tracing::warn!(
                    "Rate limit near! used={} available={} deficit={}. Sleeping {} ms",
                    used_weight,
                    available,
                    deficit,
                    sleep.as_millis()
                );

                time::sleep(sleep).await;
            }

            return self.handle_http_response(response).await;
        }
    }

    /// Get exchange information
    pub async fn exchange_info(&self) -> Result<ExchangeInformation, Error> {
        self.get(BinanceApi::Spot(Spot::ExchangeInfo), None).await
    }

    /// Get account information
    pub async fn get_account(&self) -> Result<AccountInformation, Error> {
        let mut parameters = BTreeMap::new();
        parameters.insert(String::from("omitZeroBalances"), true.to_string());

        // Build signed request
        let request: String = build_signed_request(BTreeMap::new(), self.recv_window)?;

        // Get signed request
        self.get_signed(BinanceApi::Spot(Spot::Account), Some(request))
            .await
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

    async fn trade_history_for_symbols(
        &self,
        symbols: Vec<Symbol>,
    ) -> Result<HashMap<Symbol, Vec<Trade>>, Error> {
        let mut output = HashMap::with_capacity(symbols.len());

        for pair in symbols {
            let trades = self.trade_history_for_pair(pair.symbol.clone()).await?;

            output.insert(pair, trades);
        }

        Ok(output)
    }

    /// Get trades for all **bitcoin** pairs
    pub async fn trade_history(
        &self,
        account: &AccountInformation,
    ) -> Result<HashMap<Symbol, Vec<Trade>>, Error> {
        let relevant_assets: HashSet<String> = non_btc_assets_with_balance(&account.balances);

        if relevant_assets.is_empty() {
            return Ok(HashMap::new());
        }

        let btc_pairs: &Vec<Symbol> = self.bitcoin_pairs().await?;
        let symbols: Vec<Symbol> = filter_btc_pairs_by_assets(btc_pairs, &relevant_assets);

        self.trade_history_for_symbols(symbols).await
    }
}

#[inline]
fn used_weight_1m(headers: &HeaderMap) -> u32 {
    headers
        .get("X-MBX-USED-WEIGHT-1M")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[inline]
fn throttle_delay(used_weight: u32, request_weight: u32) -> Option<Duration> {
    let required_weight: u32 = used_weight.saturating_add(request_weight);
    if required_weight <= MAX_WEIGHT_PER_MIN {
        return None;
    }

    let deficit: u32 = required_weight - MAX_WEIGHT_PER_MIN;

    // Example: deficit=100, max=6000 -> sleep for 100/6000 minute
    let sleep_ms: u64 = (deficit as f64 / MAX_WEIGHT_PER_MIN as f64 * 60_000.0) as u64;
    let sleep_ms: u64 = sleep_ms.max(200);

    Some(Duration::from_millis(sleep_ms))
}

#[inline]
fn retry_after_ms(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1000))
        .map(Duration::from_millis)
}

fn non_btc_assets_with_balance(balances: &[Balance]) -> HashSet<String> {
    balances
        .iter()
        .filter(|balance| balance.total() > 0.0 && balance.asset != BTC_TICKER)
        .map(|balance| balance.asset.clone())
        .collect()
}

fn filter_btc_pairs_by_assets(btc_pairs: &[Symbol], assets: &HashSet<String>) -> Vec<Symbol> {
    btc_pairs
        .iter()
        .filter(|pair| {
            (pair.base_asset == BTC_TICKER && assets.contains(&pair.quote_asset))
                || (pair.quote_asset == BTC_TICKER && assets.contains(&pair.base_asset))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::*;
    use crate::response::{Balance, Symbol};

    #[test]
    fn test_used_weight_header_parsing() {
        let mut headers = HeaderMap::new();
        headers.insert("X-MBX-USED-WEIGHT-1M", HeaderValue::from_static("1234"));

        assert_eq!(used_weight_1m(&headers), 1234);
    }

    #[test]
    fn test_used_weight_missing_header_defaults_to_zero() {
        let headers = HeaderMap::new();
        assert_eq!(used_weight_1m(&headers), 0);
    }

    #[test]
    fn test_throttle_delay_when_weight_is_available() {
        let delay = throttle_delay(5_979, 20);
        assert_eq!(delay, None);
    }

    #[test]
    fn test_throttle_delay_when_weight_is_missing() {
        let delay = throttle_delay(5_990, 20);
        assert_eq!(delay, Some(Duration::from_millis(200)));
    }

    #[test]
    fn test_throttle_delay_when_limit_is_already_exceeded() {
        let delay = throttle_delay(6_100, 20);
        assert_eq!(delay, Some(Duration::from_millis(1_200)));
    }

    #[test]
    fn test_retry_after_ms_parsing() {
        let mut headers = HeaderMap::new();
        headers.insert("Retry-After", HeaderValue::from_static("2"));

        assert_eq!(retry_after_ms(&headers), Some(Duration::from_millis(2_000)));
    }

    fn make_symbol(symbol: &str, base: &str, quote: &str) -> Symbol {
        Symbol {
            symbol: symbol.to_string(),
            status: "TRADING".to_string(),
            base_asset: base.to_string(),
            base_asset_precision: 8,
            quote_asset: quote.to_string(),
            quote_precision: 8,
            order_types: vec!["LIMIT".to_string()],
            iceberg_allowed: true,
            is_spot_trading_allowed: true,
            is_margin_trading_allowed: false,
        }
    }

    #[test]
    fn test_non_btc_assets_with_balance() {
        let balances = vec![
            Balance {
                asset: "BTC".to_string(),
                free: 0.2,
                locked: 0.0,
            },
            Balance {
                asset: "ETH".to_string(),
                free: 1.1,
                locked: 0.0,
            },
            Balance {
                asset: "BNB".to_string(),
                free: 0.0,
                locked: 0.2,
            },
            Balance {
                asset: "XRP".to_string(),
                free: 0.0,
                locked: 0.0,
            },
        ];

        let assets = non_btc_assets_with_balance(&balances);
        assert_eq!(assets.len(), 2);
        assert!(assets.contains("ETH"));
        assert!(assets.contains("BNB"));
    }

    #[test]
    fn test_filter_btc_pairs_by_assets() {
        let pairs = vec![
            make_symbol("ETHBTC", "ETH", "BTC"),
            make_symbol("BTCEUR", "BTC", "EUR"),
            make_symbol("LTCBTC", "LTC", "BTC"),
            make_symbol("ETHUSDT", "ETH", "USDT"),
        ];
        let assets = ["ETH".to_string(), "EUR".to_string()].into_iter().collect();

        let symbols = filter_btc_pairs_by_assets(&pairs, &assets);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].symbol, "ETHBTC");
        assert_eq!(symbols[1].symbol, "BTCEUR");
    }
}
