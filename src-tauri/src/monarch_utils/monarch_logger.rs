use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::PathBuf;

use crate::monarch_utils::monarch_fs::{create_dir, get_monarch_home, path_exists};

/// Initializes logger to ensure logs are written when running app.
/// To log to the monarch.log file you use the log macros as shown in the bottom with info!()
pub fn init_logger() {
    let log_path: PathBuf = get_log_dir();

    if !path_exists(&log_path) {
        create_dir(&log_path).unwrap();
    }

    let monarch_logs: PathBuf = get_log_file();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] - {m}\n",
        )))
        .build(monarch_logs)
        .expect("monarch_logger::init_logger() failed! Failed to build logfile!");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .expect("monarch_logger::init_logger() failed! Failed to build log config!");

    log4rs::init_config(config).unwrap();
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
