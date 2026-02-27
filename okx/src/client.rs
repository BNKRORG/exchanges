//! OKX client

use std::borrow::Cow;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Deserializer;
use url::Url;

use crate::auth::{self, OkxApiCredentials};
use crate::constant::{API_ROOT_URL, BTC_TICKER, USER_AGENT_NAME};
use crate::error::Error;
use crate::response::{
    Account, DepositTransaction, OkxApiErrorData, OkxApiResponse, Trade, WithdrawalTransaction,
};
use crate::util;

enum Api<'a> {
    Balance { currency: Option<&'a str> },
    DepositHistory { currency: Option<&'a str> },
    WithdrawalHistory { currency: Option<&'a str> },
    FillsHistory { instrument_type: Option<&'a str> },
}

impl<'a> Api<'a> {
    fn url_path(&self) -> Cow<'a, str> {
        match self {
            Self::Balance { currency } => match currency {
                Some(currency) => Cow::Owned(format!("/api/v5/account/balance?ccy={currency}")),
                None => Cow::Borrowed("/api/v5/account/balance"),
            },
            Self::DepositHistory { currency } => match currency {
                Some(currency) => {
                    Cow::Owned(format!("/api/v5/asset/deposit-history?ccy={currency}"))
                }
                None => Cow::Borrowed("/api/v5/asset/deposit-history"),
            },
            Self::WithdrawalHistory { currency } => match currency {
                Some(currency) => {
                    Cow::Owned(format!("/api/v5/asset/withdrawal-history?ccy={currency}"))
                }
                None => Cow::Borrowed("/api/v5/asset/withdrawal-history"),
            },
            Self::FillsHistory { instrument_type } => match instrument_type {
                Some(instrument_type) => Cow::Owned(format!(
                    "/api/v5/trade/fills-history?instType={instrument_type}"
                )),
                None => Cow::Borrowed("/api/v5/trade/fills-history"),
            },
        }
    }

    fn http_method(&self) -> Method {
        match self {
            Self::Balance { .. }
            | Self::DepositHistory { .. }
            | Self::WithdrawalHistory { .. }
            | Self::FillsHistory { .. } => Method::GET,
        }
    }
}

/// OKX client
#[derive(Debug, Clone)]
pub struct OkxClient {
    /// Root URL for the API.
    root_url: Url,
    /// HTTP client.
    client: Client,
    /// Authentication
    credentials: OkxApiCredentials,
}

impl OkxClient {
    /// Construct a new client.
    pub fn new(credentials: OkxApiCredentials) -> Result<Self, Error> {
        Ok(Self {
            root_url: Url::parse(API_ROOT_URL)?,
            client: Client::builder()
                .user_agent(USER_AGENT_NAME)
                .timeout(Duration::from_secs(25))
                .build()?,
            credentials,
        })
    }

    async fn send_request<T>(&self, api: Api<'_>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let method: Method = api.http_method();
        let path: Cow<str> = api.url_path();
        let path: &str = path.as_ref();
        let body: &str = "";

        // Get current timestamp
        let timestamp: DateTime<Utc> = Utc::now();

        // Generate the signature
        let signature: String = auth::generate_signature(
            &self.credentials.api_secret,
            &timestamp,
            &method,
            path,
            body,
        )?;

        let url: Url = self.root_url.join(path)?;

        let response: Response = self
            .client
            .request(method, url)
            .header("OK-ACCESS-KEY", &self.credentials.api_key)
            .header("OK-ACCESS-SIGN", signature)
            .header("OK-ACCESS-TIMESTAMP", util::format_timestamp(&timestamp))
            .header("OK-ACCESS-PASSPHRASE", &self.credentials.passphrase)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        let status_code: StatusCode = response.status();
        let response_body: String = response.text().await?;

        tracing::debug!("okx result: {response_body}");

        match status_code {
            StatusCode::OK => {
                // Use `serde_path_to_error` to obtain detailed field path information
                let deserializer = &mut Deserializer::from_str(&response_body);
                let result: OkxApiResponse = serde_path_to_error::deserialize(deserializer)?;

                if result.code == "0" {
                    return Ok(serde_json::from_value(result.data)?);
                }

                // Attempt to extract sMsg from the first element of the data array
                //
                // result={"code":"1","data":[{"clOrdId":"","ordId":"","sCode":"51000","sMsg":"Parameter ordId error","ts":"1752558485701"}],"inTime":"1752558485701589","msg":"All operations failed","outTime":"1752558485701884"}
                let smg: String = match serde_json::from_value(result.data) {
                    Ok(errors) => {
                        let errors: Vec<OkxApiErrorData> = errors;
                        let mut iter = errors.into_iter();
                        iter.next()
                            .and_then(|e| e.s_msg)
                            .unwrap_or_else(|| String::from("Unknown error"))
                    }
                    Err(..) => String::from("Failed to parse error message"),
                };

                tracing::error!("OKX API Error Response: {response_body}");
                Err(Error::OkxApiError {
                    code: result.code,
                    message: result.msg,
                    smg,
                })
            }
            StatusCode::NOT_FOUND => {
                tracing::error!("OKX API Error Response: {response_body}");
                Err(Error::OkxApiError {
                    code: "404".to_string(),
                    message: format!("API not found: '{path}'"),
                    smg: String::new(),
                })
            }
            _ => {
                tracing::error!("OKX API Error Response: {response_body}");
                Err(Error::OkxApiError {
                    code: status_code.to_string(),
                    message: response_body,
                    smg: String::new(),
                })
            }
        }
    }

    /// Get the **bitcoin** balance
    pub async fn balance(&self) -> Result<f64, Error> {
        let accounts: Vec<Account> = self
            .send_request(Api::Balance {
                currency: Some(BTC_TICKER),
            })
            .await?;

        let mut total: f64 = 0.0;

        for account in accounts {
            for detail in account.details {
                if detail.currency != BTC_TICKER {
                    continue;
                }

                total += detail.amount;
            }
        }

        Ok(total)
    }

    /// Get **bitcoin** account deposit history
    pub async fn deposit_history(&self) -> Result<Vec<DepositTransaction>, Error> {
        self.send_request(Api::DepositHistory {
            currency: Some(BTC_TICKER),
        })
        .await
    }

    /// Get **bitcoin** account withdrawals history
    pub async fn withdrawal_history(&self) -> Result<Vec<WithdrawalTransaction>, Error> {
        self.send_request(Api::WithdrawalHistory {
            currency: Some(BTC_TICKER),
        })
        .await
    }

    /// Get **bitcoin** spot trades.
    pub async fn trade_history(&self) -> Result<Vec<Trade>, Error> {
        let trades: Vec<Trade> = self
            .send_request(Api::FillsHistory {
                instrument_type: Some("SPOT"),
            })
            .await?;

        // Keep only trades that involve BTC in the pair.
        let trades: Vec<Trade> = trades
            .into_iter()
            .filter(|trade| trade.instrument_id.contains(BTC_TICKER))
            .collect();

        Ok(trades)
    }
}
