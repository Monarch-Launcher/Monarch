/*
    This file is for routines that just keep Monarch fast and clean, such as
    clearing old cached images, temporary downloads, etc...
 */

use log::{info, error};
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::time::SystemTime;

use super::monarch_fs::{get_resources_cache};

/// Clears out old cached thumbnails (Don't like the indentaion level, will come back to rework later)
pub fn clear_cached_thumbnails() {
    match get_resources_cache() {
        Ok(cache) => {
            if let Ok(files) = fs::read_dir(cache) {
                for file in files {
                    if let Ok(file_path) = file {
                        remove_thumbnail(file_path);
                    }       
                }
            }
        }
        Err(e) => {
            error!("Failed to get cache directory! | Message: {}", e);
        }
    }
    
}

/// Removes old cache file if old enough
fn remove_thumbnail(file: DirEntry) {
    if time_to_remove(file.path()) {
        info!("Clearing cached images...");
        if let Err(e) = fs::remove_file(file.path()) {
            error!("Failed to remove file: {}! | Message: {:?}", file.path().display(), e);
        }
    }
}

/// Checks if it's time to remove cached thumbnail
fn time_to_remove(file: PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(file) {
        if let Ok(time) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(time) {
                return age.as_secs() >= 1209600 // Return if file is older than 14 days
            }
        }
    }
    false
}

/// Removes all files in \resources\cache, meant for UI so that user can clear folder if wanted
pub fn clear_all_cache() {
    match get_resources_cache() {
        Ok(resources) => {
            match fs::read_dir(resources.clone()) {
                Ok(files) => {
                    for file in files {
                        match file {
                            Ok(f) => { remove_thumbnail(f); }
                            Err(e) => { error!("Failed to read file! | Message: {:?}", e); }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get files from folder: {} | Message: {}", resources.display(), e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get resources path! | Message: {}", e);
        }
    }
}