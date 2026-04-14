use std::collections::HashMap;

use binance_api::auth::BinanceAuth;
use binance_api::client::BinanceClient;

#[tokio::main]
async fn main() {
    let auth = BinanceAuth::ApiKeys {
        api_key: "api_key".to_string(),
        secret_key: "api_secret".to_string(),
    };

    let client = BinanceClient::new(auth).unwrap();

    let account = client.get_account().await.unwrap();

    println!(
        "BTC balance: {}",
        account.bitcoin_balance().unwrap().total()
    );

    let mut cursor: HashMap<String, u64> = HashMap::new();

    let incremental = client
        .trade_history_bitcoin_incremental(&account, &mut cursor)
        .await
        .unwrap();
    println!("New BTC trades: {:#?}", incremental);
    println!("Cursor: {:#?}", cursor);
}
