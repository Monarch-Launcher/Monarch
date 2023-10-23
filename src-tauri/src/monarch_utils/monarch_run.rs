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
            Ok(_) => { info!("Executing '{file}' via PowerShell...", file = path.display()) }
            Err(e) => { 
                error!("monarch_run::run_file() failed! Error while running '{file}' in PowerShell! | Error: {e}", file = path.display());
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
            Ok(_) => { info!("Executing '{file}'...", file = path.display()) }
            Err(e) => { 
                error!("monarch_run::run_file() failed! Error while running '{file}' with sh! | Error: {e}", file = path.display());
                return Err("Failed to execute file!".to_string())
            }
        }
        return Ok(())
    }
    warn!("monarch_run::run_file() not executed! No matching OS!");
    return Err("No matching OS! Don't know how to execute file!".to_string())
}

/// Exectutes a file of given path under the translation layer Wine
pub fn run_file_wine(path: PathBuf) -> Result<(), String> {
    if cfg!(windows) {
        warn!("monarch_run::run_file_wine() not executed! Attempting to run a file in wine in windows is not allowed!");
        return Err("Cannot run a file in wine in Windows!".to_string())
    }

    let result: Result<Child, Error> = Command::new("wine")
                                               .arg(path.clone())
                                               .spawn();
    
    match result {
        Ok(_) => { info!("Executing '{file}' in Wine...", file = path.display()) }
        Err(e) => { 
            error!("monarch_run::run_file_wine() failed! Error while executing: '{file}' in Wine! | Error: {e}", file = path.display());
            return Err("Failed to execute file!".to_string())
        }
    }

    return Ok(())
}
