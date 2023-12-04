use keyring::Entry;
use log::error;
use core::result::Result;

/// Save credentials to OS's secure store.
pub fn set_credentials(platform: &str, username: &str, password: &str) -> Result<(), String> {
    match Entry::new(platform, username) {
        Ok(entry) => {
            if let Err(e) = entry.set_password(password) {
                error!("monarch_credentials::set_credentials() failed! Something went wrong while setting password! | Error: {e}");
                return Err("Failed to set password!".to_string())
            }
            Ok(())
        }
        Err(e) => {
            error!("monarch_credentials::set_credentials() failed! Failed to create new entry in secure store! | Error: {e}");
            return Err("Failed to set credentials!".to_string())
        }
    }
}

/// Retrieve password from OS's secure store.
pub fn get_password(platform: &str, username: &str) -> Result<String, String> {
    match Entry::new(platform, username) {
        Ok(entry) => {
            match entry.get_password() {
                Ok(password) => { return Ok(password) }
                Err(e) => {
                    error!("monarch_credentials::get_password() failed! Could not get password! | Error: {e}");
                    return Err("Failed to retrieve password!".to_string())
                }
            }
        }
        Err(e) => {
            error!("monarch_credentials::get_password() failed! Failed to create new entry in secure store! | Error: {e}");
            return Err("Failed to get credentials!".to_string())
        }
    }
}

/// Delete credentials from OS's secure store.
pub fn delete_credentials(platform: &str, username: &str) -> Result<(), String> {
    match Entry::new(platform, username) {
        Ok(entry) => {
            if let Err(e) = entry.delete_password() {
                error!("monarch_credentials::delete_credentials() failed! Something went wrong while deleting password from secure store! | Error: {e}");
                return Err("Failed to delete credentials!".to_string())
            }
            Ok(())
        }
        Err(e) => {
            error!("monarch_credentials::delete_credentials() failed! Failed to create new entry in secure store! | Error: {e}");
            return Err("Failed to set credentials!".to_string())
        }
    }
}