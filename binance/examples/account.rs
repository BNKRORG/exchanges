use std::collections::HashMap;

use binance_api::auth::BinanceAuth;
use binance_api::builder::{BinanceEndpoint, BinanceEndpointType};
use binance_api::client::BinanceClient;

#[tokio::main]
async fn main() {
    let auth = BinanceAuth::ApiKeys {
        api_key: "api_key".to_string(),
        secret_key: "api_secret".to_string(),
    };

    let client = BinanceClient::builder()
        .auth(auth)
        .endpoint(BinanceEndpoint::from_type(BinanceEndpointType::Mainnet))
        .build()
        .unwrap();

    let account = client.get_account().await.unwrap();

    println!(
        "BTC balance: {}",
        account.bitcoin_balance().unwrap().total()
    );

    let deposits = client.deposit_history_bitcoin().await.unwrap();
    println!("Deposits: {:?}", deposits);

    let withdrawals = client.withdrawal_history_bitcoin().await.unwrap();
    println!("Withdrawals: {:?}", withdrawals);

    let mut cursor: HashMap<String, u64> = HashMap::new();

    let incremental = client
        .trade_history_bitcoin_incremental(&account, &mut cursor)
        .await
        .unwrap();
    println!("Trades: {:#?}", incremental);
    println!("Cursor: {:#?}", cursor);

    let address = client.bitcoin_deposit_address().await.unwrap();
    println!("BTC deposit address: {address}");
}
