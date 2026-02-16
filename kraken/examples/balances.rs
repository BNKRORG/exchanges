use kraken_api::auth::KrakenAuth;
use kraken_api::client::KrakenClient;

#[tokio::main]
async fn main() {
    let auth = KrakenAuth::api_keys("<api-key>", "<secret>");

    let client = KrakenClient::new(auth).unwrap();

    let balance = client.balance().await.unwrap();

    println!("Balance: {balance} BTC");
}
