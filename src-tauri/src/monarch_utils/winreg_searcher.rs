use winreg::enums::*;
use winreg::RegKey;
use std::io;

/// Return whether a program is found in windows registery.
/// Only checks under Software of each user in HKEY_USERS
pub fn is_installed(program_name: &str) -> bool {
    let hkurs: RegKey = RegKey::predef(HKEY_USERS); // Gets users in registery
    
    let mut path: String = String::from(r"\Software\"); // Specifies which path to look in
    path.push_str(program_name); // Adds program name to said path

    for user in hkurs.enum_keys() { // Loops through users in registery
        if let Ok(mut search_path) = user {
            search_path.push_str(path.as_str()); // Adds subpath of program to the users path

            if let Ok(_) = hkurs.open_subkey(&search_path) {
                return true // Returns true if path exits
            }
        }
    }
    return false; // False if path was never found
}

/// Returns content in a registry folder.
pub fn get_reg_folder_contents(dir_name: &str) -> io::Result<Vec<String>> {
    let hkurs: RegKey = RegKey::predef(HKEY_USERS); // Gets users in registery
    
    let mut path: String = String::from(r"\Software\"); // Specifies which path to look in
    path.push_str(dir_name); // Adds program name to said path

    for Ok(mut user_path) in hkurs.enum_keys() {
        user_path.push_str(path.as_str());

        if let Ok(content) = hkurs.open_subkey(&user_path) {
            let mut keys: Vec<String> = Vec::new();
            
            for key in content.enum_keys().enumerate() {
                keys.push(key.1.unwrap())
            }  
            return Ok(keys);
        }
    }

    let custom_error: io::Error = io::Error::new(io::ErrorKind::NotFound, "Steam library not found!");
    return Err(custom_error) 
}