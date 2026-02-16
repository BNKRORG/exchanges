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
    println!("{:#?}", account);

    let history = client.trade_history().await.unwrap();
    println!("{:#?}", history);
}
