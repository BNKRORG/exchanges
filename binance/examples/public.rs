use binance_api::prelude::*;

#[tokio::main]
async fn main() {
    let client = BinanceClient::builder().build().unwrap();

    let info = client.exchange_info().await.unwrap();
    println!("{:#?}", info);
}
