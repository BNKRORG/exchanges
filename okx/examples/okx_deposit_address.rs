use okx_api::auth::OkxApiCredentials;
use okx_api::client::OkxClient;

#[tokio::main]
async fn main() {
    let credentials = OkxApiCredentials {
        api_key: "api_key".to_string(),
        api_secret: "api_secret".to_string(),
        passphrase: "passphrase".to_string(),
    };

    let client = OkxClient::new(credentials).unwrap();

    let address = client.bitcoin_deposit_address().await.unwrap();
    println!("BTC deposit address: {address}");
}
