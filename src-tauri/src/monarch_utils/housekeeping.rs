/*
    This file is for routines that just keep Monarch fast and clean, such as
    clearing old cached images, temporary downloads, etc...

    Also meant to help maintain a smaller footprint on users OS.
*/

use log::{error, info};
use std::fs::ReadDir;
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::sleep;
use std::time::SystemTime;
use std::{fs, time::Duration};
use sysinfo::{System, SystemExt};

use super::monarch_fs::get_resources_cache;

/// Runs HouseKeeper loop on seperate thread
pub fn start() {
    thread::spawn(move || {
        let mut sys: System = System::new();

        loop {
            sys.refresh_cpu();

            if low_system_usage(&sys) {
                clear_cached_thumbnails();

                break; // For now assume that program will be restarted at some point within next few days.
                       // Can therefor stop the housekeeping service
                       // Housekeeping also doesn't do anything rn except clear images. Can implement more logic later
                       // as it's needed.
            }

            sleep(Duration::new(3600, 0));
        }
    });
}

/// Checks if system usage is sufficiently low to clear resources.
/// Currently only checks a certain level of CPU usage, will possibly update later
/// to check more metrics such as disk usage, memory, etc...
fn low_system_usage(system: &System) -> bool {
    system.load_average().one < 15.0 // Check that system CPU usage 1 min ago is below 15%
}

/*
    Clearing images
*/

/// Clears out old cached thumbnails (Don't like the indentaion level, will come back to rework later)
pub fn clear_cached_thumbnails() {
    let path: PathBuf = get_resources_cache();
    match fs::read_dir(path) {
        Ok(files) => {
            clear_dir(files);
        }
        Err(e) => {
            error!("housekeeping::clear_cached_thumbnails() failed! Encountered error while running fs::read_dir() | Err {e}");
        }
    }
}

/// Helper function to remove some indentation levels from clear_cached_thumbnails().
fn clear_dir(files: ReadDir) {
    let mut logged_event: bool = false;

    // Only iterate over Ok()
    for file_ in files.into_iter().flatten() {
        let file_path: PathBuf = file_.path();

        if time_to_remove(&file_path) {
            if !logged_event {
                info!("Monarch Housekeeper: Clearing cached images...");
                logged_event = true;
            }
            remove_thumbnail(&file_path);
        }
    }
}

/// Removes old cache file if old enough
fn remove_thumbnail(file: &Path) {
    if let Err(e) = fs::remove_file(file) {
        error!(
            "housekeeping::remove_thumbnail() failed! Error while removing: {path} | Error: {e}",
            path = file.display()
        );
    }
}

/// Checks if it's time to remove cached thumbnail
fn time_to_remove(file: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(file) {
        if let Ok(time) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(time) {
                return age.as_secs() >= 1209600; // Return if file is older than 14 days
                                                 // TODO: REPLACE WITH CUSTOM SETTING USER CAN CAHNGE FOR HOW LONG TO STORE IMAGES
            }
        }
    }
    false
}

/// Removes all files in /resources/cache, meant for UI so that user can clear folder if wanted
pub fn clear_all_cache() {
    info!("Manually clearing all cached images...");
    let path: PathBuf = get_resources_cache();

    match fs::read_dir(&path) {
        Ok(files) => {
            for file in files {
                match file {
                    Ok(f) => {
                        remove_thumbnail(&f.path());
                    }
                    Err(e) => {
                        error!("housekeeping::clear_all_cache() failed! Could not read file! | Error: {e}");
                    }
                }
            }
        }
        Err(e) => {
            error!("housekeeping::clear_all_cache() failed! Error while reading files from: {dir} | Error: {e}", dir = path.display());
        }
    }
}

/*
    Clearing temporary files
*/
