use std::process::Command;
use std::path::PathBuf;
use log::{info, error, warn};
use core::result::Result;
use std::process::Child;
use std::io::Error;

/// Executes the file of given path in either PowerShell or normal sh shell.
pub fn run_file(path: PathBuf) -> Result<(), String> {
    if cfg!(windows) {
        let result: Result<Child, Error> = Command::new("PowerShell")
                .arg(path.clone())
                .spawn();
    
        match result {
            Ok(_) => { info!("Executing '{}' via PowerShell...", path.display()) }
            Err(e) => { 
                error!("Failed to run '{}' in PowerShell! | Message: {:?}", path.display(), e);
                return Err("Failed to execute file!".to_string())
            }
        }
        return Ok(())
    }
    if cfg!(linux) {
        let result: Result<Child, Error> = Command::new("sh")
                .arg(path.clone())
                .spawn();
    
        match result {
            Ok(_) => { info!("Executing '{}'...", path.display()) }
            Err(e) => { 
                error!("Failed to run '{}' | Message: {:?}", path.display(), e);
                return Err("Failed to execute file!".to_string())
            }
        }
        return Ok(())
    }
    return Err("No matching OS! Don't know how to execute file!".to_string())
}

/// Exectutes a file of given path under the translation layer Wine
pub fn run_file_wine(path: PathBuf) -> Result<(), String> {
    if cfg!(windows) {
        warn!("Attempting to run a file in wine in windows is not allowed! Cancelling...");
        return Err("Cannot run a file in wine in Windows!".to_string())
    }

    let result: Result<Child, Error> = Command::new("sh")
                                               .arg("wine")
                                               .arg(path.clone())
                                               .spawn();
    
    match result {
        Ok(_) => { info!("Executing '{}' in Wine...", path.display()) }
        Err(e) => { 
            error!("Failed to run '{}' in Wine! | Message: {:?}", path.display(), e);
            return Err("Failed to execute file!".to_string())
        }
    }

    return Ok(())
}
