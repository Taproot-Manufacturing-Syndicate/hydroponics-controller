extern crate tokio;
use chrono::DateTime;
use chrono::Local;
use chrono::SubsecRound;
use chrono::TimeDelta;

#[tokio::main]
async fn main() -> () {
    // WARN while program is running, changing the clock, timezone, or daylight savings could cause problems

    // using local time as identified by system
    let current_datetime = chrono::Local::now();
    println!(
        "current local time according to chrono : {:?}",
        current_datetime
    );

    #[derive(Debug, Clone)]
    pub enum Command {
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

    impl Command {
        pub fn inspect(self) -> DateTime<Local> {
            match self {
                //Command::Pump::PumpOn(x) => x,
                Command::Pumping(Pump::PumpOn(x)) => x,
                Command::Pumping(Pump::PumpOff(x)) => x,
                Command::Lighting(Lights::LightsOn(x)) => x,
                Command::Lighting(Lights::LightsOff(x)) => x,
            }
        }
    }

    let mut _command_schedule: Vec<Command> = Vec::new();
    // TODO read-in YAML file for schedule (MVP)
    // see serde-yaml crate

    // for now, we will hard code a single day demo schedule
    // multi day schedules could be generated algorithmically, ie, same for 12 days or, reduce light by 5/min a day for 40 days, etc
    // would this be good as a test?
    let lights_off_time = Command::Lighting(Lights::LightsOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(15))
            .expect("pump off init to work"),
    ));
    let lights_on_time = Command::Lighting(Lights::LightsOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(5))
            .expect("pump off init to work"),
    ));
    let pump_on_time = Command::Pumping(Pump::PumpOn(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(10))
            .expect("pump off init to work"),
    ));
    let pump_off_time = Command::Pumping(Pump::PumpOff(
        current_datetime
            .round_subsecs(0)
            .checked_add_signed(TimeDelta::seconds(20))
            .expect("pump off init to work"),
    ));

    // manual display, but to show user would also require similar shinanigans
    println!("pump off! @ {:#?}", Command::inspect(pump_off_time.clone()));
    println!("pump on! @ {:#?}", Command::inspect(pump_on_time.clone()));

    println!(
        "lights off! @ {:#?}",
        Command::inspect(lights_off_time.clone())
    );
    println!(
        "lights on! @ {:#?}",
        Command::inspect(lights_on_time.clone())
    );

    let mut demo_schedule: Vec<Command> =
        vec![lights_on_time, lights_off_time, pump_on_time, pump_off_time];

    println!("demo schedule init: {:?}", demo_schedule);
    demo_schedule.sort_by(|a, b| Command::inspect(a.clone()).cmp(&Command::inspect(b.clone())));
    println!("demo schedule sorted: {:?}", demo_schedule);

    // TODO now sorted into temporal order, parse into device cues
    let mut light_schedule: Vec<Command> = Vec::new();
    let mut pump_schedule: Vec<Command> = Vec::new();

    for cmd in &demo_schedule {
        match cmd {
            Command::Lighting(_) => light_schedule.push(cmd.clone()),
            Command::Pumping(_) => pump_schedule.push(cmd.clone()),
        }
    }

    println!("light schedule : {:?}", light_schedule);
    println!("pump schedule : {:?}", pump_schedule);

    //TODO : actually send requests based on time
    //TODO : two async tasks: spawn, spawn + join
}
