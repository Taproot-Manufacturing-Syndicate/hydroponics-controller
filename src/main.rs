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
struct LightConfig {
    tasmota_hostname: String,
    start: chrono::NaiveTime,
    end: chrono::NaiveTime,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Config {
    pump: Option<PumpConfig>,
    light: Option<LightConfig>,
}

fn notify(msg: &str) {
    println!("{msg}");
    let _r = std::process::Command::new("/home/seb/bin/text-me")
        .arg(format!("Subject:\n\n{msg}"))
        .output()
        .expect("failed to run command");
}

// Wait for `start`, turn the tasmota device on, wait for `end`, turn
// it back off.
//
// Reports how much energy it used while on.
//
// Handles getting called at any time, not just before `start`.
//
// If we're past `end`, the behavior depends on `wait_for_day_rollover`:
//     - If it's true, wait for the `start` tomorrow.
//     - If it's false, the on-period is handled and it returns immediately.
async fn handle_power_on_period(
    name: &str,
    tasmota_device: &tasmor_lib::Device<tasmor_lib::protocol::HttpClient>,
    mut start: chrono::NaiveTime,
    end: chrono::NaiveTime,
    wait_for_day_rollover: bool,
) {
    let now = chrono::Local::now().time();
    println!("handling {name} power-on period:");
    println!("    now: {now:#?}");
    println!("    start: {start:#?}");
    println!("    end: {end:#?}");
    println!("    wait_for_day_rollover: {wait_for_day_rollover:#?}");

    let mut time_until_start = start - now;
    let time_until_start = if time_until_start > chrono::TimeDelta::zero() {
        // This is the happy path: `start` is in the future, just wait
        // until then.
        time_until_start
    } else {
        println!("oops, we missed the start!");
        if now < end {
            println!("we're in the middle of the on-period! start right away!");
            start = now;
            chrono::TimeDelta::zero()
        } else {
            println!("we missed the end too");
            if wait_for_day_rollover {
                println!("waiting for the start tomorrow");
                while time_until_start < chrono::TimeDelta::zero() {
                    time_until_start += chrono::TimeDelta::days(1);
                }
                time_until_start
            } else {
                println!("this period's done");
                return;
            }
        }
    };
    println!("start is in {}", timedelta_to_str(&time_until_start));
    tokio::time::sleep(time_until_start.to_std().unwrap()).await;

    // It's time to start!

    // FIXME: This takes a couple of seconds and we actually miss the
    // `start` instant.
    let start_energy = match tasmota_device.energy().await {
        Ok(energy) => energy.total_energy(),
        Err(e) => {
            println!("failed to read energy from {name} tasmota device: {}", e);
            None
        }
    };

    tasmota_device.power_on().await.unwrap();
    notify(&format!("{name} on"));

    // TODO: Instead of just sleeping, monitor power and shut
    // down if it does something weird.
    let sleep_diration = (end - chrono::Local::now().time()).to_std().unwrap();
    println!("sleep duration is {sleep_diration:#?}");
    tokio::time::sleep(sleep_diration).await;

    println!("{name} off");
    tasmota_device.power_off().await.unwrap();

    let on_time = end - start;
    notify(&format!(
        "{name} off, was on for {}",
        timedelta_to_str(&on_time)
    ));

    let final_energy = match tasmota_device.energy().await {
        Ok(energy) => energy.total_energy(),
        Err(e) => {
            println!("failed to read energy from {name} tasmota device: {}", e);
            None
        }
    };

    let on_hours = on_time.num_milliseconds() as f32 / (1_000.0 * 60.0 * 60.0);
    match (start_energy, final_energy) {
        (Some(start_energy), Some(final_energy)) => {
            let energy = final_energy - start_energy;
            let avg_power = 1000.0 * energy / on_hours;
            println!("{name} consumed {:.3} kWh during this on-period", energy);
            println!("{name} averaged {:.3} W during this on-period", avg_power);
            notify(&format!(
                "{name} consumed {energy:.3} kWh (avg {avg_power:.3} W)"
            ));
        }
        (_, _) => {
            println!("failed to read energy from {name} tasmota");
        }
    }
}

async fn run_light(light_config: LightConfig) {
    // unwrap as panic
    let (tasmota_device, _initial_state) = tasmor_lib::Device::http(&light_config.tasmota_hostname)
        .build()
        .await
        .unwrap();

    loop {
        println!("light: top of loop");
        handle_power_on_period(
            "light",
            &tasmota_device,
            light_config.start,
            light_config.end,
            true,
        )
        .await;
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
        println!("pump: top of loop");
        // We assume that the flood configs are sorted by time, so
        // earlier comes before later.
        for flood in &pump_config.floods {
            handle_power_on_period(
                "pump",
                &tasmota_device,
                flood.start,
                flood.start + flood.duration,
                wait_for_day_rollover,
            )
            .await;
        }

        // We're done with the *last* flood of the day, wait for the
        // start of the first one tomorrow.
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

    if let Some(pump_config) = config.pump {
        set.spawn(run_pump(pump_config));
    }

    if let Some(light_config) = config.light {
        set.spawn(run_light(light_config));
    }

    set.join_all().await;
}
