use coinbase_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = CoinbaseAuth::ApiKeys {
        api_key: String::from("<api-key>"),
        secret_key: String::from("<secret-key>"),
    };
    let client = CoinbaseAppClient::new(auth).unwrap();

    let accounts = client.accounts().await.unwrap();

    for account in accounts {
        println!("{:#?}", account);
    }

    let address = client.bitcoin_deposit_address().await.unwrap();
    println!("BTC deposit address: {address}");
}
