extern crate serde;
extern crate tokio;

use core::panic;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use tokio::task::JoinSet;
use tokio::time::sleep;
use url::Url;

/// Schedules in current form are per-device.
#[derive(Deserialize)]
pub struct Schedule {
    on: Url,
    off: Url,
    times: Vec<Event>, //
}

/// Events in current form will are for simple on/off instructions
/// the on field being true signifying an on signal, and false signifying off
#[derive(Debug, Deserialize)]
pub struct Event {
    timestamp: DateTime<Utc>,
    on: bool,
}

#[tokio::main]
async fn main() -> () {
    let schedule_json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("schedule.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value to work after read-in");

    let devices: Value = schedule_json_contents["devices"].clone();

    let mut set: JoinSet<()> = JoinSet::new();

    // parse events and commands, and launch async task in for loop
    for category in devices.as_object().expect("as object to work").keys() {
        let cat = devices
            .get(category)
            .expect("category to be found")
            .as_object()
            .expect("as object to work")
            .keys();

        for fixture in cat {
            // format String to make the call to pointer.
            let commands_path = format!("/{}/{}/commands", category, fixture);
            println!("commands_path {:?}", commands_path);

            // to access the url and event schedule.
            let fixture_schedule: Schedule = serde_json::from_value(
                devices
                    .pointer(&commands_path)
                    .expect("pointer to work")
                    .clone(),
            )
            .expect("serde to get schedule");

            set.spawn(async move { spawn_fixture_schedules(fixture_schedule).await });
        }
    }
    set.join_all().await;
    println!("!!**^%&@ schedule is completed @&%^**!!");
}

async fn spawn_fixture_schedules(fix_sched: Schedule) {
    let mut inner_set: JoinSet<()> = JoinSet::new();
    for ev in fix_sched.times {
        // the current scheme is to spawn a process per event.
        println!("reading Event : {:?}", ev);

        // the spawned helper function takes arguments
        // DateTime<Utc>, bool (our only current case for message content), Url
        // we could add String to pass in custom POST message body

        // passing in the Url is still awkward,
        // it is included in the fixture_schedule but not the event list, and the process needs it

        let passed_address: Url = match ev.on {
            true => fix_sched.on.clone(),
            false => fix_sched.off.clone(),
        };

        inner_set.spawn(async move { event_process(ev.timestamp, ev.on, passed_address).await });
    }
    inner_set.join_all().await;
}

async fn event_process(event_time: DateTime<Utc>, message: bool, destination: Url) -> () {
    if event_time > chrono::Utc::now() {
        sleep(
            (event_time - chrono::Utc::now())
                .to_std()
                .expect("to_std to work"),
        )
        .await;
    } else {
        panic!("Fatal! Event timestamp prior to system time detected.");
    }

    let poster = reqwest::Client::new();
    let res = poster
        .post(destination)
        .body(message.to_string())
        //seems a little wasteful to convert our bool back to string
        .send()
        .await;

    if res.is_err() {
        // TODO handle errors
        println!("error result in POST")
    }
}
