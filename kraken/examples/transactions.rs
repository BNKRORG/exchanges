use kraken_api::auth::KrakenAuth;
use kraken_api::client::KrakenClient;

#[tokio::main]
async fn main() {
    let auth = KrakenAuth::api_keys("<api-key>", "<secret>");

    let client = KrakenClient::new(auth).unwrap();

    let transactions = client.deposit_transactions().await.unwrap();

    for tx in transactions {
        println!("{:#?}", tx);
    }

    let transactions = client.withdraw_transactions().await.unwrap();

    for tx in transactions {
        println!("{:#?}", tx);
    }
}
