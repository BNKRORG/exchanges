use strike_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = StrikeAuth::api_key("<api-key>");

    let client = StrikeClient::new(auth).unwrap();

    let balance = client.balance().await.unwrap();

    println!("Balance: {} BTC", balance.total);
}
