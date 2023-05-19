use std::{process::exit, io, fs};
use serde_json::Value;
use log::error;
use std::env::VarError;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use regex::Regex;

/*
---------- General functions for filesystem tasks ----------
*/

/// Run on startup to ensure users filesystem is ready for Monarch launcher 
pub fn init_monarch_fs() {
    check_appdata_folder();
    check_resources_folder();
}

/// Create Monarch folder in users %appdata% directory
fn check_appdata_folder() {
    let appdata_path = get_app_data_path();

    match appdata_path {  
        Ok(path) =>  {
            if !path_exists(&path) {
                if let Err(e) = create_dir(&path) {
                    println!("Failed to create Monarch %appdata% folder! | Message: {:?}", e);
                } // Returns result of creating directory
            }
        }
        Err(e) => {
            // If Monarch fails to create its own %appdata% directory
            println!("Something went wrong looking for %appdata% folder! \nErr: {:?} \nExiting... ", e);
            exit(1); // Exit out of app!
        }
    }
}

/// Folder to store image resources for game thumbnails etc...
fn check_resources_folder() {
    let resources_dir = get_resources_path();
    let cache_dir = get_resources_cache();
    let lib_img_dir = get_resources_library();
        
    if !path_exists(&resources_dir) {
        if let Err(e) = create_dir(&resources_dir) {
            println!("Failed to create empty folder: {}! | Message: {:?}", resources_dir, e);
            exit(1);
        }
    }

    if !path_exists(&cache_dir) {
        if let Err(e) = create_dir(&cache_dir) {
            println!("Failed to create empty folder: {}! | Message: {:?}", cache_dir, e);
            exit(1);
        }
    }

    if !path_exists(&lib_img_dir) {
        if let Err(e) = create_dir(&lib_img_dir) {
            println!("Failed to create empty folder: {}! | Message: {:?}", lib_img_dir, e);
            exit(1);
        }
    }
}

/// Gets the users %appdata% directory and adds \Monarch to the end of it to generate Monarch path
pub fn get_app_data_path() -> Result<String, VarError> {
    let app_data_path_res = std::env::var("APPDATA");
    
    match app_data_path_res {
        Ok(mut app_data_path) => {
            app_data_path.push_str("\\Monarch");
            return Ok(app_data_path) 
        },
        Err(e) => Err(e)
    }
}

/// Returns path to library.json
pub fn get_library_json_path() -> String {
    if let Ok(mut path) = get_app_data_path() {
        path.push_str("\\library.json");
        return path
    }
    String::new()
}

/// Returns path to collections.json
pub fn get_collections_json_path() -> String {
    if let Ok(mut path) = get_app_data_path() {
        path.push_str("\\collections.json");
        return path
    }
    String::new()
}

/// Write JSON to file
pub fn write_json_content(content: Value, path: &str) -> io::Result<()> {
    if let Err(e) = fs::write(path, content.to_string()) {
        error!("Failed to write new library to: {} | Message: {:?}", path, e);
        return Err(e)
    }
    return Ok(())
}

/// Checks whether a given path exists already or not
pub fn path_exists(path: &str) -> bool {
    if Path::new(path).exists() {
        return true
    }
    return false
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: &str) -> io::Result<()> {
    let result = fs::create_dir(path);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

/*
---------- Functions related to storing in resources dir ----------
*/

/// Returns path to resources folder.
/// Should never fail during runtime because of init_monarch_fs,
/// but if it does it returns an empty string.
pub fn get_resources_path() -> String {
    if let Ok(mut path) = get_app_data_path() {
        path.push_str("\\resources");
        return path
    }
    return String::new()
}

/// Returns path to store temporary images
pub fn get_resources_cache() -> String {
    let mut cache_path = get_resources_path();
    cache_path.push_str("\\cache");
    return cache_path
}

/// Returns path to store thumbnails for games in library
pub fn get_resources_library() -> String {
    let mut lib_img_path = get_resources_path();
    lib_img_path.push_str("\\library");
    return lib_img_path
}

/// Clears out old cached thumbnails
pub fn clear_cached_thumbnails() {
    if let Ok(files) = fs::read_dir(get_resources_cache()) {
        for file in files {
            if let Ok(file_path) = file {
                remove_thumbnail(file_path);
            }       
        }
    }
}

/// Removes old cache file if old enough
fn remove_thumbnail(file: DirEntry) {
    if time_to_remove(file.path()) {
        if let Err(e) = fs::remove_file(file.path()) {
            error!("Failed to remove file from: {}! | Message: {:?}", get_resources_cache(), e);
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
    if let Ok(files) = fs::read_dir(get_resources_cache()) {
        for file in files {
            if let Ok(file_path) = file {
                force_remove_thumbnail(file_path);
            }       
        }
    }
}

/// Removes old cache file even if it's not old enough
fn force_remove_thumbnail(file: DirEntry) {
    if time_to_remove(file.path()) {
        if let Err(e) = fs::remove_file(file.path()) {
            error!("Failed to remove file from: {}! | Message: {:?}", get_resources_cache(), e);
        }
    }
}

/// Create a name for image file in cache directory
/// Can be used to download image and check if an image already exists
pub fn generate_cache_image_name(name: &str) -> String {
    let filename = generate_image_filename(name);
    let mut dir = get_resources_cache();
    
    dir.push_str("\\");
    dir.push_str(&filename);
    dir
}

/// Create a name for image file in cache directory
pub fn generate_library_image_name(name: &str) -> String {
    let filename = generate_image_filename(name);
    let mut dir = get_resources_library();
    
    dir.push_str("\\");
    dir.push_str(&filename);
    dir
}

/// Generates a filename without any special characters or spaces
fn generate_image_filename(name: &str) -> String {
    let mut filename: String = String::from(name);
    filename = filename.replace(" ", "_");

    let re: Regex = Regex::new(r"[^a-zA-Z0-9_]").unwrap();
    filename = re.replace_all(&filename, "").to_string();
    filename.push_str(".jpg");

    return filename
}