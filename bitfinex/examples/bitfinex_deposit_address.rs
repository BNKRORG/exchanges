use bitfinex_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = BitfinexAuth::api_keys("<api-key>", "<api-secret>");
    let client = BitfinexClient::new(auth).unwrap();

    let address = client.bitcoin_deposit_address().await.unwrap();
    println!("BTC deposit address: {address}");
}
