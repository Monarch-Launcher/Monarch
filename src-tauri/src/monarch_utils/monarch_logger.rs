use std::fs::File;
use std::io;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::monarch_utils::monarch_fs::{create_dir, get_monarch_home, path_exists};

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
        Err(_) => tracing_subscriber::EnvFilter::new("info"),
    };

    let logfile = File::create(monarch_logs).unwrap();
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
}

/// Creates path to log folder that should be located under %appdata%.
pub fn get_log_dir() -> PathBuf {
    let mut log_path: PathBuf = get_monarch_home();
    log_path = log_path.join("logs");
    log_path
}

/// Creates path to log file that should be located under %appdata%.
pub fn get_log_file() -> PathBuf {
    let mut log_path: PathBuf = get_monarch_home();
    log_path = log_path.join("logs");
    log_path = log_path.join("monarch.log");
    log_path
}
