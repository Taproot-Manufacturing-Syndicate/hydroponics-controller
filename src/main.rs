extern crate tokio;
use chrono::DateTime;
use chrono::Local;
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

    impl Pump {
        pub fn inspect(value: Pump) -> DateTime<Local> {
            match value {
                Pump::PumpOn(x) => x,
                Pump::PumpOff(x) => x,
            }
        }
    }

    impl Lights {
        pub fn inspect(value: Lights) -> DateTime<Local> {
            match value {
                Lights::LightsOn(x) => x,
                Lights::LightsOff(x) => x,
            }
        }
    }

    let mut _command_schedule: Vec<Command> = Vec::new();
    // TODO read-in YAML file for schedule (MVP)
    // see serde-yaml crate

    // for now, we will hard code a demo schedule
    // would this be good as a test?
    let lights_on_time = Lights::LightsOn(
        current_datetime
            .checked_add_signed(TimeDelta::seconds(5))
            .expect("lights on to work"),
    );
    let lights_off_time = Lights::LightsOff(
        current_datetime
            .checked_add_signed(TimeDelta::seconds(15))
            .expect("lights off to work"),
    );
    let pump_on_time = Pump::PumpOn(
        current_datetime
            .checked_add_signed(TimeDelta::seconds(10))
            .expect("pump to work"),
    );
    let pump_off_time = Pump::PumpOff(
        current_datetime
            .checked_add_signed(TimeDelta::seconds(20))
            .expect("pump to work"),
    );

    // manual display, but to show user would also require similar shinanigans
    println!("pump off! @ {:#?}", Pump::inspect(pump_off_time.clone()));
    println!("pump on! @ {:#?}", Pump::inspect(pump_on_time.clone()));
    println!(
        "lights off! @ {:#?}",
        Lights::inspect(lights_off_time.clone())
    );
    println!(
        "lights on! @ {:#?}",
        Lights::inspect(lights_on_time.clone())
    );

    let demo_schedule: Vec<Command> = vec![
        Command::Lighting(lights_on_time),
        Command::Lighting(lights_off_time),
        Command::Pumping(pump_on_time),
        Command::Pumping(pump_off_time),
    ];

    println!("demo schedule: {:?}", demo_schedule);
    // multiday schedules would also fit into a single schedule given the systemtime's calandar
    // and could be generated algorithmically, ie, same for 12 days or, reduce light by 5/min a day for 40 days, etc

    //TODO NEXT : two async tasks: spawn, spawn + join
}
