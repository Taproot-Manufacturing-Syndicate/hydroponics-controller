extern crate tokio;
use chrono::DateTime;
use chrono::Local;
use chrono::SubsecRound;
use chrono::TimeDelta;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Instruction {
    Lighting(Lights),
    Pumping(Pump),
}

#[derive(Debug, Clone)]
pub enum Lights {
    LightsOn(DateTime<Local>),
    LightsOff(DateTime<Local>),
}

#[derive(Debug, Clone)]
pub enum Pump {
    PumpOn(DateTime<Local>),
    PumpOff(DateTime<Local>),
}

impl Instruction {
    pub fn inspect(self) -> DateTime<Local> {
        match self {
            Instruction::Pumping(Pump::PumpOn(x)) => x,
            Instruction::Pumping(Pump::PumpOff(x)) => x,
            Instruction::Lighting(Lights::LightsOn(x)) => x,
            Instruction::Lighting(Lights::LightsOff(x)) => x,
        }
    }
}
#[tokio::main]
async fn main() -> () {
    // WARN while program is running, changing the clock, timezone, or daylight savings could cause problems

    // using local time as identified by system
    let current_datetime = chrono::Local::now();
    println!(
        "current local time according to chrono : {:?}",
        current_datetime
    );

    let mut _command_schedule: Vec<Instruction> = Vec::new();

    // TODO JSON file for schedule (MVP)
    let contents = tokio::fs::read_to_string("demo.json")
        .await
        .expect("reading in JSON to work");
    println!("JSON File has {} lines.", contents.lines().count());
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
            .checked_add_signed(TimeDelta::seconds(25))
            .expect("pump off init to work"),
    ));
    let lights_on_time = Instruction::Lighting(Lights::LightsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(15))
            .expect("pump off init to work"),
    ));
    let pump_on_time = Instruction::Pumping(Pump::PumpOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(20))
            .expect("pump off init to work"),
    ));
    let pump_off_time = Instruction::Pumping(Pump::PumpOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(30))
            .expect("pump off init to work"),
    ));

    // manual display, but to show user would also require similar shinanigans
    println!(
        "pump off! @ {:#?}",
        Instruction::inspect(pump_off_time.clone())
    );
    println!(
        "pump on! @ {:#?}",
        Instruction::inspect(pump_on_time.clone())
    );
    println!(
        "lights off! @ {:#?}",
        Instruction::inspect(lights_off_time.clone())
    );
    println!(
        "lights on! @ {:#?}",
        Instruction::inspect(lights_on_time.clone())
    );
    //TODO pretty print upcoming schedule instead

    let mut demo_schedule: Vec<Instruction> =
        vec![lights_on_time, lights_off_time, pump_on_time, pump_off_time];

    println!("demo schedule init: {:?}", demo_schedule);
    demo_schedule
        .sort_by(|a, b| Instruction::inspect(a.clone()).cmp(&Instruction::inspect(b.clone())));
    println!("demo schedule sorted: {:?}", demo_schedule);

    // now sorted into temporal order, parse into device cues
    let mut light_schedule: Vec<Instruction> = Vec::new();
    let mut pump_schedule: Vec<Instruction> = Vec::new();

    for cmd in &demo_schedule {
        match cmd {
            Instruction::Lighting(_) => light_schedule.push(cmd.clone()),
            Instruction::Pumping(_) => pump_schedule.push(cmd.clone()),
        }
    }

    println!("light schedule : {:?}", light_schedule);
    println!("pump schedule : {:?}", pump_schedule);

    //TODO : two async tasks: spawn, spawn + join
    tokio::spawn(async move {
        // Process each socket concurrently.
        light_process(light_schedule).await
    });

    //calculate time within task?
    //TODO : actually send requests based on time
}

async fn light_process(schedule: Vec<Instruction>) -> () {
    // TODO add duration to systemttime
    println!("inside light process")
}
