use strike_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = StrikeAuth::api_key("<api-key>");

    let client = StrikeClient::new(auth).unwrap();

    let deposits = client.deposits().await.unwrap();

    println!("Deposits:");

    for deposit in deposits {
        println!("{deposit:#?}");
    }
}
