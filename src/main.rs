use chrono::{NaiveTime, TimeDelta};

use serde::Deserialize;
use tokio::{task::JoinSet, time::sleep, time::Duration};
use url::Url;

extern crate tokio;

#[derive(Debug, Deserialize)]
pub struct Device {
    name: String,
    on: Url,
    off: Url,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    timestamp: NaiveTime,
    on: bool,
}

#[tokio::main]
async fn main() -> () {
    let devices: Vec<Device> = serde_json::from_str(
        &(tokio::fs::read_to_string("the.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value works after read-in");
    println!("{:?}", devices);

    let mut set: JoinSet<()> = JoinSet::new();

    for device in devices {
        set.spawn(run_schedule(device));
    }

    set.join_all().await;
    ()
}

async fn run_schedule(d: Device) {
    let demo_duration: Duration = Duration::from_secs(5);
    loop {
        // first find the difference bewteen now (in naivetime) and the schedule,
        // sleep that duration/timedelta
        println!("{:?}", &d);
        let resp = reqwest::get(d.on.clone()).await;
        println!("{resp:#?}");
        sleep(demo_duration).await;
        // do the thing (get)
    }
}
