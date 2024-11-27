extern crate tokio;

#[tokio::main]
async fn main() -> () {
    let body = reqwest::get("https://docs.rs/").await.unwrap().text().await;
    let resp = reqwest::get("http://127.0.0.1:9732").await;
    println!("{body:#?}");
    println!("{resp:#?}");
    ()
}
