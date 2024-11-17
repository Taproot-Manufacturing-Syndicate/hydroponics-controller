extern crate tokio;

use chrono::DateTime;
use chrono::Local;
use chrono::SubsecRound;
use chrono::TimeDelta;
use serde_json::Value;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub enum Instruction {
    Lighting(Lights),
    Pumping(Pumps),
}

#[derive(Debug, Clone)]
pub enum Lights {
    LightsOn(DateTime<Local>),
    LightsOff(DateTime<Local>),
}

#[derive(Debug, Clone)]
pub enum Pumps {
    PumpsOn(DateTime<Local>),
    PumpsOff(DateTime<Local>),
}

impl Instruction {
    pub fn inspect(self) -> DateTime<Local> {
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
    // using local time as identified by system
    let current_datetime = chrono::Local::now();

    let mut _command_schedule: Vec<Instruction> = Vec::new();

    // TODO JSON file for schedule (MVP)

    let demo_json_contents: Value = serde_json::from_str(
        &(tokio::fs::read_to_string("demo.json")
            .await
            .expect("reading in JSON to work")),
    )
    .unwrap();
    println!(
        "JSON is working inside main, here are some untyped values,  {:?}, cool? {:?}",
        demo_json_contents["devices"], demo_json_contents["cool"]
    );

    // for now, we will hard code a single day demo schedule
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

    tokio::spawn(async move { light_process(lights_schedule).await });
    tokio::spawn(async move { pump_process(pumps_schedule).await });

    // TODO replace with join
    sleep(Duration::from_secs(15)).await;

    return ();
}

async fn light_process(l_schedule: Vec<Instruction>) -> () {
    println!("inside light process");
    for l in l_schedule {
        sleep(
            (l.clone().inspect() - chrono::Local::now())
                .to_std()
                .expect("to_std to work"),
        )
        .await;
        println!("l in l_sch");
        println!("{:?}", l)
    }
}
async fn pump_process(p_schedule: Vec<Instruction>) -> () {
    println!("inside pump process");
    for p in p_schedule {
        sleep(
            (p.clone().inspect() - chrono::Local::now())
                .to_std()
                .expect("to_std to work"),
        )
        .await;
        println!("p in p_sch");
        println!("{:?}", p)
    }
}
