use strike_api::prelude::*;

#[tokio::main]
async fn main() {
    let auth = StrikeAuth::api_key("<api-key>");

    let client = StrikeClient::new(auth).unwrap();

    let invoices = client.invoices().await.unwrap();

    println!("Invoices:");

    for invoice in invoices {
        println!("{invoice:#?}");
    }
}
