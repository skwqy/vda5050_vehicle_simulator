//! Per-vehicle file logging: `tracing` + `tracing-subscriber` + `tracing-appender` (non-blocking).
//!
//! - Main traffic (MC→ / AGV→ except visualization when configured separately): `logs/{serial}/vehicle.log*`
//! - Optional visualization payloads: `logs/{serial}/visualization.log*` (see `log_visualization_messages`)
//!
//! `log` records are bridged with metadata `target: "log"`; the real target is in `log.target`.
//!
//! Visualization uses targets `vehicle_viz_{serial}` so they can be routed to a separate file.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::config::Config;
use crate::size_rotating::SizeRotatingFile;

/// Keeps background flush workers alive for non-blocking appenders (must not drop until shutdown).
pub struct LogGuards {
    _guards: Vec<tracing_appender::non_blocking::WorkerGuard>,
}

/// Build `vehicle_{serial}` target string (must match directory name under `logs/`).
pub fn vehicle_log_target(serial: &str) -> String {
    format!("vehicle_{}", serial)
}

/// Target for visualization-only logs (separate rolling file under `logs/{serial}/`).
pub fn vehicle_viz_log_target(serial: &str) -> String {
    format!("vehicle_viz_{}", serial)
}

/// Collects `message` and `log.target` from a tracing event (covers `log` crate bridge records).
#[derive(Default)]
struct VehicleEventExtract {
    message: String,
    log_target: Option<String>,
}

impl Visit for VehicleEventExtract {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => self.message = value.to_string(),
            "log.target" => self.log_target = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}

/// Writes one line per matching event to the non-blocking appender for `vehicle_*` / `vehicle_viz_*` targets.
struct VehicleFileLayer {
    by_target: Arc<HashMap<String, tracing_appender::non_blocking::NonBlocking>>,
}

impl<S> Layer<S> for VehicleFileLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let mut ex = VehicleEventExtract::default();
        event.record(&mut ex);

        let route = if meta.target().starts_with("vehicle_viz_") {
            Some(meta.target().to_string())
        } else if meta.target().starts_with("vehicle_") {
            Some(meta.target().to_string())
        } else if meta.target() == "log" {
            match ex.log_target.take() {
                Some(t) if t.starts_with("vehicle_viz_") => Some(t),
                Some(t) if t.starts_with("vehicle_") => Some(t),
                _ => None,
            }
        } else {
            None
        };

        let Some(key) = route else {
            return;
        };
        let Some(nb) = self.by_target.get(&key) else {
            return;
        };

        let ts = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let line = format!(
            "{} {:5} {} {}\n",
            ts,
            meta.level(),
            key,
            ex.message
        );
        let _ = nb.clone().write_all(line.as_bytes());
    }
}

/// Initialize global subscriber: stderr for all filtered logs; rolling files per vehicle / viz targets.
pub fn init_from_config(config: &Config) -> Result<LogGuards, String> {
    let root: PathBuf = ["logs"].iter().collect();

    let mut guards: Vec<tracing_appender::non_blocking::WorkerGuard> = Vec::new();
    let mut by_target: HashMap<String, tracing_appender::non_blocking::NonBlocking> = HashMap::new();

    let max_bytes = config.settings.log_max_file_bytes;
    let max_files = config.settings.log_max_files;

    for i in 0..config.settings.robot_count {
        let serial = format!(
            "{}{}",
            config.vehicle.serial_number,
            config.settings.serial_suffix_start + i
        );
        let dir = root.join(&serial);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("create log dir {}: {e}", dir.display()))?;

        let main_rot = SizeRotatingFile::new(dir.join("vehicle.log"), max_bytes, max_files)
            .map_err(|e| format!("vehicle log {}: {e}", dir.display()))?;
        let (non_blocking, guard) = tracing_appender::non_blocking(main_rot);
        guards.push(guard);

        let target = vehicle_log_target(&serial);
        by_target.insert(target, non_blocking);

        if config.settings.log_visualization_messages {
            let viz_rot = SizeRotatingFile::new(dir.join("visualization.log"), max_bytes, max_files)
                .map_err(|e| format!("visualization log {}: {e}", dir.display()))?;
            let (viz_nb, viz_guard) = tracing_appender::non_blocking(viz_rot);
            guards.push(viz_guard);
            by_target.insert(vehicle_viz_log_target(&serial), viz_nb);
        }
    }

    let stderr_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_filter(stderr_filter);

    let map = Arc::new(by_target);
    let vehicle_filter = filter_fn(|meta: &tracing::Metadata| {
        meta.target().starts_with("vehicle_") || meta.target() == "log"
    });
    let vehicle_layer = VehicleFileLayer {
        by_target: Arc::clone(&map),
    }
    .with_filter(vehicle_filter);

    Registry::default()
        .with(stderr_layer)
        .with(vehicle_layer)
        .try_init()
        .map_err(|e| format!("tracing init: {e}"))?;

    Ok(LogGuards { _guards: guards })
}
