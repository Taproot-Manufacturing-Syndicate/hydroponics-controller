/// Turn pumps and light on and off according to a schedule, to run a
/// hydroponics setup.
use clap::Parser;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// The config file name.
    config_file_name: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct FloodConfig {
    start: chrono::NaiveTime,
    #[serde(deserialize_with = "duration_str::deserialize_duration")]
    duration: std::time::Duration,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct PumpConfig {
    tasmota_hostname: String,
    floods: Vec<FloodConfig>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Config {
    pump: PumpConfig,
}

async fn sleep_until(flood: &FloodConfig) {
    let mut time_until_start = flood.start - chrono::Local::now().time();
    println!(
        "waiting for the flood, {}",
        timedelta_to_str(&time_until_start)
    );

    while time_until_start < chrono::TimeDelta::zero() {
        time_until_start += chrono::TimeDelta::days(1);
        println!(
            "adjusted sleep duration is {}",
            timedelta_to_str(&time_until_start)
        );
    }

    let sleep_duration = time_until_start.to_std().unwrap();
    println!("sleep duration is {sleep_duration:#?}");
    tokio::time::sleep(sleep_duration).await;
}

async fn handle_flood(
    flood: &FloodConfig,
    pump_tasmota_device: &tasmor_lib::Device<tasmor_lib::protocol::HttpClient>,
    wait_for_day_rollover: bool,
) {
    let now = chrono::Local::now().time();
    println!("now: {now:#?}");

    println!("considering flood {:#?}", flood);

    if now < flood.start {
        // Sleep until this flood starts.
        sleep_until(flood).await;
    } else if now < flood.start + flood.duration {
        println!("we're in the middle of this flood, better start right away");
    } else {
        // This flood is in the past, or tomorrow.
        if wait_for_day_rollover {
            sleep_until(flood).await;
        } else {
            println!("we missed this flood");
            return;
        }
    }

    println!("starting flood");
    let start_time = chrono::Local::now().time();

    let start_energy = match pump_tasmota_device.energy().await {
        Ok(energy) => energy.total_energy(),
        Err(e) => {
            println!("failed to read energy from pump tasmota device: {}", e);
            None
        }
    };
    // println!("start energy: {start_energy:#?}");

    pump_tasmota_device.power_on().await.unwrap();

    let flood_end = flood.start + flood.duration;
    let time_until_end = flood_end - chrono::Local::now().time();
    println!(
        "sleeping until {} ({})",
        flood_end,
        timedelta_to_str(&time_until_end)
    );

    // TODO: Instead of just sleeping, monitor pump power and
    // shut down if it does something weird.
    tokio::time::sleep(time_until_end.to_std().unwrap()).await;

    println!("ending flood");
    pump_tasmota_device.power_off().await.unwrap();

    let end_time = chrono::Local::now().time();

    let final_energy = match pump_tasmota_device.energy().await {
        Ok(energy) => energy.total_energy(),
        Err(e) => {
            println!("failed to read energy from pump tasmota device: {}", e);
            None
        }
    };
    // println!("final energy: {final_energy:#?}");

    let flood_time = end_time - start_time;
    let flood_time_hours = flood_time.num_milliseconds() as f32 / (1_000.0 * 60.0 * 60.0);

    match (start_energy, final_energy) {
        (Some(start_energy), Some(final_energy)) => {
            let energy = final_energy - start_energy;
            let avg_power = 1000.0 * energy / flood_time_hours;
            println!("pump consumed {:.3} kWh during this flood", energy);
            println!("pump averaged {:.3} W during this flood", avg_power);
        }
        (_, _) => {
            println!("failed to read energy from tasmota");
        }
    }
}

async fn run_pump(pump_config: PumpConfig) {
    // unwrap as panic
    let (tasmota_device, _initial_state) = tasmor_lib::Device::http(&pump_config.tasmota_hostname)
        .build()
        .await
        .unwrap();

    // This flag is false when the program first starts, the first time
    // through the flood loop below.  This makes it *skip over* floods
    // that are in the past (aka tomorrow), to handle the situation
    // where we start the program in the middle of a bunch of floods
    // and we want to skip ahead to the next flood of the day.
    //
    // The first time we get to the end of the list of floods, we set
    // this flag to true. At that point the program always waits for the
    // next flood. This is to handle the rollover at the end of the day.
    let mut wait_for_day_rollover = false;

    loop {
        println!("top of loop");
        // We assume that the flood configs are sorted by time, so
        // earlier comes before later.
        for flood in &pump_config.floods {
            handle_flood(flood, &tasmota_device, wait_for_day_rollover).await;
        }
        wait_for_day_rollover = true;
    }
}

fn timedelta_to_str(timedelta: &chrono::TimeDelta) -> String {
    let total_seconds = timedelta.abs().num_seconds();
    let s = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let m = total_minutes % 60;
    let h = timedelta.num_hours();
    format!("{}:{:02}:{:02}", h, m, s)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config: Config = serde_json::from_str(
        &(tokio::fs::read_to_string(args.config_file_name)
            .await
            .expect("reading in JSON to work")),
    )
    .expect("JSON resolves to Config");
    println!("config: {config:#?}");

    let mut set = tokio::task::JoinSet::<()>::new();

    set.spawn(run_pump(config.pump));

    set.join_all().await;
}
