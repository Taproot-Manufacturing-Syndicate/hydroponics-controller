extern crate serde;
extern crate tokio;

use chrono::DateTime;
use chrono::SubsecRound;
use chrono::TimeDelta;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio::task::JoinSet;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum Instruction {
    Lighting(Lights),
    Pumping(Pumps),
}

#[derive(Debug, Clone)]
pub enum Lights {
    LightsOn(DateTime<Utc>),
    LightsOff(DateTime<Utc>),
}

#[derive(Debug, Clone)]
pub enum Pumps {
    PumpsOn(DateTime<Utc>),
    PumpsOff(DateTime<Utc>),
}

impl Instruction {
    pub fn inspect(self) -> DateTime<Utc> {
        match self {
            Instruction::Pumping(Pumps::PumpsOn(x)) => x,
            Instruction::Pumping(Pumps::PumpsOff(x)) => x,
            Instruction::Lighting(Lights::LightsOn(x)) => x,
            Instruction::Lighting(Lights::LightsOff(x)) => x,
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

    // TODO JSON file for schedule (MVP)
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

    // command schedule is hard coded from file, so not good for demo
    let mut command_schedule: Vec<Instruction> = Vec::new();

    let kv_sched = serde_json::Map::deserialize(&schedule_json_contents["schedule"])
        .expect("deserialize to map to be working");
    println!("HEY : {:#?}", kv_sched);

    /*
    for (k, v) in kv_sched {
        println!("{:?}", k);
        let vee = serde_json::Value::as_str(&v)
            .expect("geting string of datetime from value should work");
        println!("{:?}", vee);
        match k.as_str() {
           "PumpOn" => {
                command_schedule.push(Instruction::Pumping(Pumps::PumpsOn(DateTime::parse_from_str(vee).into().expect("Something good"))));
                _ => ()
            }
        }i
        //unwrap Value (string) into chrono::something
        //key can match enum OR just create something if that's easier
    }
        */
    // s is map of String / Value

    //let sched = chrono::Local::from(schedule_json_contents["schedule"])
    //     ::from_value(schedule_json_contents)
    //        .expect("JSON schedule contents to provide Value");
    //println!("{:?}", sched);

    // for demonstration, we will hard code a single day demo schedule
    // multi day schedules could be generated algorithmically, ie, same for 12 days or, reduce light by 5/min a day for 40 days, etc
    // would this be good as a test?
    let lights_off_time = Instruction::Lighting(Lights::LightsOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(14))
            .expect("pump off init to work"),
    ));
    let lights_on_time = Instruction::Lighting(Lights::LightsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(3))
            .expect("pump off init to work"),
    ));
    let pump_on_time = Instruction::Pumping(Pumps::PumpsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(8))
            .expect("pump off init to work"),
    ));
    let pump_off_time = Instruction::Pumping(Pumps::PumpsOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(13))
            .expect("pump off init to work"),
    ));

    let mut demo_schedule: Vec<Instruction> =
        vec![lights_on_time, lights_off_time, pump_on_time, pump_off_time];

    //TODO pretty print upcoming schedule instead
    // manual display, but to show user would also require similar shinanigans
    println!("demo schedule init: {:#?}", demo_schedule);
    demo_schedule
        .sort_by(|a, b| Instruction::inspect(a.clone()).cmp(&Instruction::inspect(b.clone())));
    println!("demo schedule sorted: {:#?}", demo_schedule);

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
            Instruction::Lighting(Lights::LightsOn(_)) => "LightsOn",
            Instruction::Lighting(Lights::LightsOff(_)) => "LightsOff",
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
            Instruction::Pumping(Pumps::PumpsOn(_)) => "PumpsOn",
            Instruction::Pumping(Pumps::PumpsOff(_)) => "PumpsOff",
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
