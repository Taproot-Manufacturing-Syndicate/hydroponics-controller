use chrono::NaiveTime;
use url::Url;
extern crate tokio;

pub struct Device {
    name: String,
    on: Url,
    off: Url,
    events: Vec<Event>,
}

pub struct Event {
    timestamp: NaiveTime,
    on: bool,
}

#[tokio::main]
async fn main() -> () {
    let resp = reqwest::get("http://127.0.0.1:9732").await;
    println!("{resp:#?}");

    let post_client = reqwest::Client::new();
    let res_one = post_client
        .post("https://httpbin.org/post")
        .body("this is the body one")
        .send()
        .await;
    println!("res_one : {res_one:#?}");
    ()
}
