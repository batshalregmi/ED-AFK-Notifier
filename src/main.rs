use std::fs::{File, OpenOptions};
use std::io::Read;
use ed_journals::journal::auto_detect_journal_path;
use std::path::{Path, PathBuf};
use std::{process, thread};
use std::time::Duration;
use chrono::Utc;
use ed_journals::logs::asynchronous::LiveLogDirReader;
use ed_journals::logs::LogEventContent;
use tokio::task;
use pushover::API;
use pushover::requests::message::SendMessage;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_writer_pretty};

const FILE_PATH: &str = "./ed-afk-config.json";
const DEFAULT_TOKEN: &str = "add_your_token_here";
const DEFAULT_USER_KEY: &str = "add_your_user_key_here";

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    user_key: String
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut missions_count: u8 = 0;
    let path = PathBuf::from(auto_detect_journal_path().unwrap());
    let mut live_dir_reader = LiveLogDirReader::open(path).unwrap();

    let config = setup_config()?;

    send_notification("ED AFK Notifier has started!".parse().unwrap(), &config.token, &config.user_key);

    // Get the current time
    let start_time = Utc::now();

    // Spawn a new task to run the loop
    let handle = task::spawn(async move {
        while let Some(entry) = live_dir_reader.next().await {
            let entry = entry.as_ref().unwrap();

            // Check if the entry is a mission event before we filter out the old events
            match &entry.content {
                LogEventContent::Missions(missions) => {
                    missions_count = missions.active.len() as u8;
                    println!("Initial Missions count: {}", missions_count);
                }
                _ => {}
            }
            // Ignore old events
            if entry.timestamp < start_time {
                continue;
            }
            match &entry.content {
                LogEventContent::ShieldState(shield_state) => {
                    if !shield_state.shields_up {
                        println!("Shields down");
                        send_notification("Shields are down".parse().unwrap(), &config.token, &config.user_key);
                    }
                }
                LogEventContent::HullDamage(hull_damage) => {
                    if hull_damage.player_pilot {
                        let hull_percentage = hull_damage.health * 100.0;
                        if hull_percentage < 75f32 || hull_percentage < 50f32 || hull_percentage <  25f32 || hull_percentage < 5f32 {
                            println!("Hull damage detected: Hull at {}%", hull_percentage.to_string());
                            send_notification(format!("Taking damage: Hull at {}%", hull_percentage.to_string()), &config.token, &config.user_key);
                        }
                    }
                }
                LogEventContent::FighterDestroyed(_) => {
                    println!("Fighter destroyed");
                    send_notification("Fighter destroyed".parse().unwrap(), &config.token, &config.user_key);
                }
                LogEventContent::CollectCargo(collect_cargo) => {
                    if collect_cargo.stolen {
                        println!("Stolen cargo collected");
                        send_notification("Stolen cargo collected".parse().unwrap(), &config.token, &config.user_key);
                    }
                }
                LogEventContent::Died(_) => {
                    println!("Your commander has died");
                    send_notification("Your commander has died".parse().unwrap(), &config.token, &config.user_key);
                }
                LogEventContent::Missions(missions) => {
                    let mut temp_missions_count = 0;
                    for mission in &missions.active {
                        if mission.expires > 0 {
                            temp_missions_count += 1;
                        }
                    }
                    missions_count = temp_missions_count;
                    println!("Active Missions: {}", missions_count);
                }
                LogEventContent::MissionRedirected(_) => {
                    missions_count -= 1;
                    handle_missions_count(missions_count, &config.token, &config.user_key);
                }
                LogEventContent::MissionFailed(_) => {
                    missions_count -= 1;
                    handle_missions_count(missions_count, &config.token, &config.user_key);
                }
                LogEventContent::MissionAbandoned(_) => {
                    missions_count -= 1;
                    handle_missions_count(missions_count, &config.token, &config.user_key);
                }
                LogEventContent::MissionAccepted(_) => {
                    missions_count += 1;
                    println!("Missions count: {}", missions_count);
                }

                _ => {}
            }
        }
    });
    handle.await.unwrap();

    Ok(())
}

fn handle_missions_count(missions_count: u8, token: &str, user_key: &str) {
    if missions_count == 0 {
        println!("No active missions");
        send_notification("No active missions".parse().unwrap(), token, user_key);
    } else {
        println!("Missions count: {}", missions_count);
    }
}

fn send_notification(message: String, token: &str, user_key: &str) {
    println!("Sending notification: {}", message);
    let api = API::new();
    let resp = api.send(&SendMessage::new(token, user_key, message));
    if resp.is_err() {
        println!("Failed to send notification: {:?}", resp.err());
    }
}

fn setup_config() -> Result<Config, Box<dyn std::error::Error>> {
    if !Path::new(FILE_PATH).exists() {
        create_config_file()?;
        println!("Config file not found. A new one has been created at {}. Please fill in the json and run the program again.", FILE_PATH);
        thread::sleep(Duration::from_secs(10));
        process::exit(1);
    }

    let config = read_config_file()?;

    if config.token == DEFAULT_TOKEN || config.user_key == DEFAULT_USER_KEY {
        println!("Please fill in the config file at {} and run the program again.", FILE_PATH);
        thread::sleep(Duration::from_secs(10));
        process::exit(1);
    }

    Ok(config)
}

fn create_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(FILE_PATH)?;

    let config = Config {
        token: String::from(DEFAULT_TOKEN),
        user_key: String::from(DEFAULT_USER_KEY)
    };

    to_writer_pretty(file, &config)?;
    Ok(())
}

fn read_config_file() -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(FILE_PATH)?;
    let mut buff = String::new();
    file.read_to_string(&mut buff)?;

    let config: Config = from_str(&buff)?;
    Ok(config)
}