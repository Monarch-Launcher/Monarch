use log::info;
use crate::monarch_utils::monarch_fs::{get_app_data_path, create_dir, path_exists};

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

/// Initializes logger to ensure logs are written when running app.
/// To log to the monarch.log file you use the log macros as shown in the bottom with info!()
pub fn init_logger() {
    let log_path = get_log_dir();

    if !path_exists(&log_path) {
        create_dir(&log_path).unwrap();
    }

    let monarch_logs: String = get_log_file();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{l}] - {m}\n")))
        .build(monarch_logs).expect("Failed to build logfile during init_logger()");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
                        .appender("logfile")
                        .build(LevelFilter::Info))
                        .expect("Failed to build logger config during init_logger()");

    log4rs::init_config(config).unwrap();
    info!("Logger initialized!");
}

/// Creates path to log folder that should be located under %appdata%.
pub fn get_log_dir() -> String {
    let mut log_path: String = get_app_data_path().unwrap();
    log_path.push_str("\\logs");
    return log_path
}

/// Creates path to log file that should be located under %appdata%.
pub fn get_log_file() -> String {
    let mut log_path: String = get_app_data_path().unwrap();
    log_path.push_str("\\logs\\monarch.log");
    return log_path

}