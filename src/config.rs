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

fn default_serial_suffix_start() -> u32 {
    1
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub action_time: f32,
    pub speed: f32,
    pub robot_count: u32,
    pub state_frequency: u64,
    pub visualization_frequency: u64,
    pub map_id: String,
    /// First robot uses this numeric suffix appended to `vehicle.serial_number`; each additional
    /// robot increments by 1 (e.g. start 2 → …2, …3, …).
    #[serde(default = "default_serial_suffix_start")]
    pub serial_suffix_start: u32,
}

/// OpenTCS plant XML and layout options. Omitted `[map]` in TOML → defaults (disabled).
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct MapConfig {
    pub enabled: bool,
    pub xml_path: String,
    pub layout_scale_mm: f64,
    pub layout_flip_y: bool,
    /// Stop when this close to target node (m).
    pub arrival_threshold_m: f32,
    /// Simulation time step for motion integration (s); should match publish tick (~0.05).
    pub sim_dt_seconds: f32,
    /// OpenTCS `point name` to place the AGV at startup (world coordinates from map). Same idea as
    /// Java `simulation.initialPointName`. Requires `enabled` and a successful map load.
    pub initial_point_name: Option<String>,
    /// Aligns with `aos-backend` `yeefungagv.plc.name-prefixes` + `aos.vda5050.map-name-prefixes`
    /// (see `YeefungAgvPlcPrefixProperties` / `application-dev.yml`): resolve VDA `nodeId` / `edgeId`
    /// that omit `Point_` / `Path_` to OpenTCS plant names.
    #[serde(default)]
    pub name_prefixes: MapNamePrefixes,
}

/// Same semantics as Java `YeefungAgvPlcPrefixProperties` / `AgvPlcNamePrefixHelper` for point and path
/// names (vehicle prefix is PLC-only; not used for map lookup here).
#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MapNamePrefixes {
    /// When true, map lookups accept stripped ids (e.g. `"3"` → `"Point_3"`) like backend
    /// `map-name-prefixes.apply-stripping` / YAML `applyStripping`.
    #[serde(alias = "apply-stripping")]
    pub apply_stripping: bool,
    /// Must end with `_` when `apply_stripping` is true (validated at map load).
    #[serde(alias = "pointPrefix")]
    pub point_prefix: String,
    #[serde(alias = "pathPrefix")]
    pub path_prefix: String,
}

impl Default for MapNamePrefixes {
    fn default() -> Self {
        Self {
            apply_stripping: false,
            point_prefix: "Point_".into(),
            path_prefix: "Path_".into(),
        }
    }
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            xml_path: "maps/youle-final-4.xml".into(),
            layout_scale_mm: 1.0,
            layout_flip_y: false,
            arrival_threshold_m: 0.08,
            sim_dt_seconds: 0.05,
            initial_point_name: None,
            name_prefixes: MapNamePrefixes::default(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mqtt_broker: MqttBrokerConfig,
    pub vehicle: VehicleConfig,
    pub settings: Settings,
    #[serde(default)]
    pub map: MapConfig,
}
