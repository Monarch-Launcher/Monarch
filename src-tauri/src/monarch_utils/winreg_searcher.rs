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
        let mut search_path: String = user.unwrap();
        search_path.push_str(path.as_str()); // Adds subpath of program to the users path

        let result = hkurs.open_subkey(&search_path); // Tests if path exists

        match result {
            Ok(_) => { return true } // Returns true if path exits
            Err(_) => { }
        }
    }
    return false; // False if path was never found
}

/// Returns content in a registry folder.
pub fn get_reg_folder_contents(dir_name: &str) -> io::Result<Vec<String>> {
    let hkurs: RegKey = RegKey::predef(HKEY_USERS); // Gets users in registery
    
    let mut path: String = String::from(r"\Software\"); // Specifies which path to look in
    path.push_str(dir_name); // Adds program name to said path

    for user in hkurs.enum_keys() {
        let mut search_path: String = user.unwrap();
        search_path.push_str(path.as_str());

        let result = hkurs.open_subkey(&search_path);

        match result {
            Ok(content) => {
                let mut keys: Vec<String> = Vec::new();
                
                for key in content.enum_keys().enumerate() {
                    keys.push(key.1.unwrap())
                }  
                
                return Ok(keys);
            }
            Err(e) => {
                // Do nothing, check next "user" in registry 
            }
        }
    }

    let custom_error: io::Error = io::Error::new(io::ErrorKind::NotFound, "Steam library not found!");
    return Err(custom_error) 
}