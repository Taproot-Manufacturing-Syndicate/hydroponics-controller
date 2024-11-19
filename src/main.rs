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
pub struct Schedule {
    on: URL,
    off: URL,
    times: Vec<Event>, //
}

// Events in this form will are for simple on/off instructions
// the on field being true signifying an on signal, and false sinifying off
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

    let demo_json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("demo.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value to work after read-in");
    println!(
        "JSON is working inside main, here are some untyped values,  {:?}, cool? {:?}",
        demo_json_contents["devices"], demo_json_contents["cool"]
    );

    // JSON file for schedule
    let schedule_json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("schedule.json")
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON Value to work after read-in");
    println!(
        "JSON schedule is working inside main, {:?}",
        schedule_json_contents["schedule"]
    );

    // TODO NEW JSON scheme!
    // command schedule is hard coded from file, so not great for a demo
    let mut command_schedule: Vec<Instruction> = Vec::new();

    let kv_sched = serde_json::Map::deserialize(&schedule_json_contents["schedule"])
        .expect("deserialize to map to be working");
    println!("kv_sched: {:#?}", kv_sched);
    for (k, v) in kv_sched {
        println!("{:?}", k);
        let vee = serde_json::Value::as_str(&v)
            .expect("geting string of datetime from value should work");
        println!("{:?}", vee);
        let vee_utc: DateTime<Utc> =
            chrono::DateTime::from_str(vee).expect("chrono from_str to work");
        match k.as_str() {
            "PumpsOn" => {
                command_schedule.push(Instruction::Pumping(PumpInstruction::PumpsOn(vee_utc)))
            }
            "PumpsOff" => {
                command_schedule.push(Instruction::Pumping(PumpInstruction::PumpsOff(vee_utc)))
            }
            "LightsOn" => {
                command_schedule.push(Instruction::Lighting(LightInstruction::LightsOn(vee_utc)))
            }
            "LightsOff" => {
                command_schedule.push(Instruction::Lighting(LightInstruction::LightsOff(vee_utc)))
            }
            _ => panic!("OICH"),
        }
    }
    println!("COMMAND SCHEDULE: {:?}", command_schedule);

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
