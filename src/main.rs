use chrono::{Local, NaiveTime, TimeDelta};
use serde::Deserialize;
use tokio::{task::JoinSet, time::sleep};
use url::Url;

extern crate tokio;

#[derive(Debug, Deserialize, Clone)]
pub struct Device {
    name: String,
    on: Url,
    off: Url,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Event {
    timestamp: NaiveTime,
    on: bool,
}

#[tokio::main]
async fn main() {
    let devices: Vec<Device> = serde_json::from_str(
        &(tokio::fs::read_to_string("the.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON resolves to Vec<Device>");
    println!("devices as Vec<Device> : {:#?}", devices);

    let mut set: JoinSet<()> = JoinSet::new();

    for device in devices {
        println!("working with device {:?}", device.name);
        let evs = device.events.clone();
        println!("events : {:#?}", evs);

        for e in evs {
            println!("about to spawn 'e' on inner_set {:?}", e);
            set.spawn(event_loop(device.clone(), e));
        }
    }

    set.join_all().await;
}

fn timedelta_to_str(timedelta: &TimeDelta) -> String {
    let total_seconds = timedelta.abs().num_seconds();
    let s = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let m = total_minutes % 60;
    let h = timedelta.num_hours();
    format!("{}:{:02}:{:02}", h, m, s)
}

async fn event_loop(d: Device, e: Event) {
    loop {
        let now: NaiveTime = Local::now().time();
        let mut diff: TimeDelta = e.timestamp - now;

        // with NaiveTime for events, any given time will be before AND after now.
        // Negative time occurs each time an event fires and is asked to sleep
        // until the same NaiveTime, and in some initial conditions.
        if diff.num_nanoseconds().expect(
            "num_nanoseconds to work [2^63 nano overflow.]",
            // fun fact: the number of nanoseconds in a day is only 8.64 x 10^13
        ) < 0
        // then, we must add 24 hours to the wait time.
        {
            diff += TimeDelta::days(1);
        }

        println!(
            "{:?}:{:?} now is {now}, about to sleep {}",
            d.name,
            e,
            timedelta_to_str(&diff)
        );

        let s = diff.to_std().expect("to_std to work");
        println!("{:?}:{:?} sleeping s={:#?}", d.name, e, s);
        sleep(s).await;
        println!("{:?}:{:?} awoke at {}", d.name, e, Local::now().time());

        // now it's time for action!
        if e.on {
            let resp = reqwest::get(d.on.clone()).await;
            println!("{:?}:{:?} On ! {:#?}", d.name, e, resp);
        } else {
            let resp = reqwest::get(d.off.clone()).await;
            println!("{:?}:{:?} Off ! {:#?}", d.name, e, resp);
        }
        // and the loop will go on forever.
    }
}
