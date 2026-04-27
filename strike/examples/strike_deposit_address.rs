use strike_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = StrikeAuth::api_key("<api-key>");
    let client = StrikeClient::new(auth).unwrap();

    let address = client.bitcoin_deposit_address().await.unwrap();
    println!("BTC deposit address: {address}");
}
