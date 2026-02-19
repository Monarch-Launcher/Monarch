use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{fmt, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::monarch_utils::monarch_fs::{self, create_dir, get_monarch_home, path_exists};
use crate::monarch_utils::monarch_settings::{self, Settings};

/// Initializes logger to ensure logs are written when running app.
/// To log to the monarch.log file you use the log macros as shown in the bottom with info!()
pub fn init_logger() {
    let log_path: PathBuf = get_log_dir();
    if !path_exists(&log_path) {
        create_dir(&log_path).unwrap();
    }
    let monarch_logs: PathBuf = get_log_file();

    let filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(_) => {
            if cfg!(debug_assertions) {
                tracing_subscriber::EnvFilter::new("info,monarch=debug")
            } else {
                tracing_subscriber::EnvFilter::new("info")
            }
        }
    };

    let logfile: File = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&monarch_logs)
        .unwrap();

    let file_layer = layer()
        .with_ansi(false)
        .with_writer(logfile)
        .with_target(true)
        .with_level(true);

    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_writer(io::stdout)
        .with_target(true) // optional: omit target
        .with_level(true); // optional: show log level

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    info!("Logger initialized");
    debug!("Logging to: {}", monarch_logs.display());
}

/// Creates path to log folder that should be located under %appdata%.
/// Logger gets initialised before MONARCH_STATE, which uses logging,
/// therefore we manually read the home path from settings.
pub fn get_log_dir() -> PathBuf {
    // Try reading settings
    if let Ok(settings_table) = monarch_settings::read_settings() {
        if let Ok(settings) = settings_table.try_into::<Settings>() {
            return PathBuf::from(settings.monarch.monarch_home.clone()).join("logs");
        }
    }

    // Else default
    monarch_fs::generate_monarch_home()
        .expect(
            "monarch_logger::get_log_dir() Failed to generate monarch home path! Unrecoverable.",
        )
        .join("logs")
}

/// Creates path to log file that should be located under %appdata%.
pub fn get_log_file() -> PathBuf {
    get_log_dir().join("monarch.log")
}
