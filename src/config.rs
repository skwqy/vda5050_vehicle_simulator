use std::path::PathBuf;

use config_file::FromConfigFile;
use serde::Deserialize;

/// Prefer `config.toml` next to the executable (portable copy), else the file in the current
/// working directory (e.g. `cargo run` from the repo root).
fn resolve_config_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside = dir.join("config.toml");
            if beside.is_file() {
                return beside;
            }
        }
    }
    PathBuf::from("config.toml")
}

pub fn get_config() -> Config {
    let path = resolve_config_path();
    Config::from_config_file(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load config from {}: {e}",
            path.display()
        )
    })
}

#[derive(Deserialize, Clone)]
pub struct MqttBrokerConfig {
    pub host: String,
    pub port: String,
    pub vda_interface: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct VehicleConfig {
    pub manufacturer: String,
    pub serial_number: String,
    pub vda_version: String,
    pub vda_full_version: String,
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub action_time: f32,
    pub speed: f32,
    pub robot_count: u32,
    pub state_frequency: u64,
    pub visualization_frequency: u64,
    pub map_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mqtt_broker: MqttBrokerConfig,
    pub vehicle: VehicleConfig,
    pub settings: Settings
}
