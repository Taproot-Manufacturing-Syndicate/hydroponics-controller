extern crate serde;
extern crate tokio;

use chrono::{DateTime, SubsecRound, TimeDelta, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;
use tokio::task::JoinSet;
use tokio::time::sleep;
use url::Url;

// Schedules in this form are per-device.
#[derive(Deserialize)]
pub struct Schedule {
    on: Url,
    off: Url,
    times: Vec<Event>, //
}

// Events in this form will are for simple on/off instructions
// the on field being true signifying an on signal, and false sinifying off
#[derive(Debug, Deserialize)]
pub struct Event {
    timestamp: DateTime<Utc>,
    on: bool,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Lighting(LightInstruction),
    Pumping(PumpInstruction),
}

#[derive(Debug, Clone)]
pub enum LightInstruction {
    LightsOn(DateTime<Utc>),
    LightsOff(DateTime<Utc>),
}

#[derive(Debug, Clone)]
pub enum PumpInstruction {
    PumpsOn(DateTime<Utc>),
    PumpsOff(DateTime<Utc>),
}

impl Instruction {
    pub fn inspect(self) -> DateTime<Utc> {
        match self {
            Instruction::Pumping(PumpInstruction::PumpsOn(x)) => x,
            Instruction::Pumping(PumpInstruction::PumpsOff(x)) => x,
            Instruction::Lighting(LightInstruction::LightsOn(x)) => x,
            Instruction::Lighting(LightInstruction::LightsOff(x)) => x,
        }
    }
}

#[tokio::main]
async fn main() -> () {
    let current_datetime = chrono::Utc::now();

    // JSON file for schedule
    let schedule_json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("schedule.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value to work after read-in");

    let devices: Value = schedule_json_contents["devices"].clone();

    // parse events and commands, and launch async task in for loop
    for category in devices.as_object().expect("as object to work").keys() {
        let cat = devices
            .get(category)
            .expect("category to be found")
            .as_object()
            .expect("as object to work")
            .keys();

        for fixture in cat {
            println!("category : {:?} fixture : {:?}", category, fixture);
            // format strings to make the call to pointer.
            // TODO ONLY COMMANDS?
            let commands_path = format!("/{}/{}/commands", category, fixture);
            let events_path = format!("/{}/{}/commands/events", category, fixture);
            println!("commands_path {:?}", commands_path);
            println!("event_path {:?}", events_path);

            //now we can access the url and event schedule.
            let current_events = devices
                .pointer(&events_path)
                .expect("pointer to work")
                .as_array()
                .expect("as array to contain a vec of values")
                .clone();
            println!("current events : {:?}", current_events);
            for ce in current_events {
                let event: Event =
                    serde_json::from_value(ce).expect("json to deserialize to event struct");
                println!("Event! : {:?}", event);
            }
        }
    }

    // command schedule is hard coded from file, so not great for a demo
    let mut _command_schedule: Vec<Instruction> = Vec::new();

    // for demonstration, we will hard code a single day demo schedule
    // multi day schedules could be generated algorithmically, ie, same for 12 days or, reduce light by 5/min a day for 40 days, etc
    // would this be good as a test?
    let lights_off_time = Instruction::Lighting(LightInstruction::LightsOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(14))
            .expect("pump off init to work"),
    ));
    let lights_on_time = Instruction::Lighting(LightInstruction::LightsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(3))
            .expect("pump off init to work"),
    ));
    let pump_on_time = Instruction::Pumping(PumpInstruction::PumpsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(8))
            .expect("pump off init to work"),
    ));
    let pump_off_time = Instruction::Pumping(PumpInstruction::PumpsOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(13))
            .expect("pump off init to work"),
    ));

    let mut demo_schedule: Vec<Instruction> =
        vec![lights_on_time, lights_off_time, pump_on_time, pump_off_time];

    //TODO pretty print upcoming schedule instead
    // manual display, but to show user would also require similar shinanigans
    println!("demo schedule init: {:?}", demo_schedule);
    demo_schedule
        .sort_by(|a, b| Instruction::inspect(a.clone()).cmp(&Instruction::inspect(b.clone())));
    println!("demo schedule sorted: {:?}", demo_schedule);

    // now sorted into temporal order, parse into device cues
    let mut lights_schedule: Vec<Instruction> = Vec::new();
    let mut pumps_schedule: Vec<Instruction> = Vec::new();

    for cmd in &demo_schedule {
        match cmd {
            Instruction::Lighting(_) => lights_schedule.push(cmd.clone()),
            Instruction::Pumping(_) => pumps_schedule.push(cmd.clone()),
        }
    }

    println!("light schedule : {:?}", lights_schedule);
    println!("pump schedule : {:?}", pumps_schedule);

    // When set completes the schedule is exhausted.
    let mut set: JoinSet<()> = JoinSet::new();
    // Adding spawned tasks to set. These are tokio::task::join_set::JoinSet.spawn
    set.spawn(async move { light_process(lights_schedule).await });
    set.spawn(async move { pump_process(pumps_schedule).await });

    // then waiting for them all to complete.
    set.join_all().await;
    println!("schedule is completed");
    // therefore main returns () and the program exits.
}

async fn light_process(l_schedule: Vec<Instruction>) -> () {
    for l in l_schedule {
        sleep(
            (l.clone().inspect() - chrono::Utc::now())
                .to_std()
                .expect("to_std to work"),
        )
        .await;

        println!("{:?}", l);

        let poster = reqwest::Client::new();

        let post_body = match l {
            Instruction::Lighting(LightInstruction::LightsOn(_)) => "LightsOn",
            Instruction::Lighting(LightInstruction::LightsOff(_)) => "LightsOff",
            _ => panic!("wrong or malformed type in lighting"),
        };

        let res = poster
            .post("http://127.0.0.1:9732")
            .body(post_body)
            .send()
            .await;

        if res.is_err() {
            // TODO handle errors
            println!("error result in POST, lighting")
        }
    }
}
async fn pump_process(p_schedule: Vec<Instruction>) -> () {
    for p in p_schedule {
        sleep(
            (p.clone().inspect() - chrono::Utc::now())
                .to_std()
                .expect("to_std to work"),
        )
        .await;

        println!("{:?}", p);

        let poster = reqwest::Client::new();

        let post_body = match p {
            Instruction::Pumping(PumpInstruction::PumpsOn(_)) => "PumpsOn",
            Instruction::Pumping(PumpInstruction::PumpsOff(_)) => "PumpsOff",
            _ => panic!("wrong or malformed type in Pumping"),
        };

        let res = poster
            .post("http://127.0.0.1:9732")
            .body(post_body)
            .send()
            .await;

        if res.is_err() {
            // TODO handle errors
            println!("error result in POST, Pumping")
        }
    }
}
