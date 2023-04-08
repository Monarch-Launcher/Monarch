use std::env::VarError;
use std::{fs, io};
use std::path::Path;
use std::process::exit;

/// Run on startup to ensure users filesystem is ready for Monarch launcher 
pub fn init_monarch_fs() {
    check_appdata_folder();
    check_resources_folder();
}

/// Create Monarch folder in users %appdata% directory
fn check_appdata_folder() {
    if let Ok(path) = get_app_data_path() {
        if !path_exists(&path) {
            create_dir(&path); // Returns result of creating directory
        }
    }
    // If Monarch fails to create its own %appdata% directory
    println!("Something went wrong looking for %appdata% folder! \nErr: {} \nExiting...", e);
    exit(1); // Exit out of app!
}

/// Folder to store image resources for game thumbnails etc...
fn check_resources_folder() {
    if let Ok(mut path) = get_app_data_path() {
        path.push_str("\\resources");
        
        if !path_exists(&path) {
            if let Err(e) = create_dir(&path) {
                println!("Failed to create empty resources folder! | Message: {:?}", e);
            }
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