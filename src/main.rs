use chrono::{NaiveTime, TimeDelta};

use serde_json::Value;
use tokio::{task::JoinSet, time::sleep, time::Duration};
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
    let json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("the.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value works after read-in");
    println!("{:?}", json_contents);

    // TODO ingest into structs

    let mut set: JoinSet<()> = JoinSet::new();

    // TODO just a demo
    let devices = vec![1, 2];
    for device in devices {
        set.spawn(run_schedule(device));
    }

    let resp = reqwest::get("http://127.0.0.1:9732").await;
    println!("{resp:#?}");

    let post_client = reqwest::Client::new();
    let res_one = post_client
        .post("https://httpbin.org/post")
        .body("this is the body one")
        .send()
        .await;
    println!("res_one : {res_one:#?}");
    set.join_all().await;
    ()
}

// helper
async fn run_schedule(n: usize) {
    let demo_duration: Duration = Duration::from_secs(5);
    loop {
        println!("{:?}", n);
        sleep(demo_duration).await;
    }
}
