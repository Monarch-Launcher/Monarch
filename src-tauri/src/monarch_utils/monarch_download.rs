use reqwest;
use reqwest::Response;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use log::{info, error};
use image;
use core::result::Result;

use super::monarch_run::run_file;
use super::monarch_fs::create_dir;

/// Creates a tmp path name for file to install.
async fn create_file_path(response: &Response, tmp_dir: &PathBuf) -> PathBuf {
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    // If file doesn't have an extension, assume its an executable.
    if !fname.contains(".") {
        let mut string_fname: String = String::from(fname);
        string_fname.push_str(".exe");
        tmp_dir.join(string_fname)
    }
    else {
        tmp_dir.join(fname)
    }
    
}

/// Writes downloaded content to file, has to be it's own function to 
/// close file and avoid "file used by another process" error.
async fn write_content(installer_path: &PathBuf, content: Response) {
    match File::create(installer_path) {
        Ok(mut file) => {
            match &content.bytes().await {
                Ok(buf) => {
                    if let Err(e) = file.write_all(buf) {
                        error!("Failed to write_all to file: {} (write_content()) | Message: {:?}",  installer_path.display(), e);
                    }
                    if let Err(e) = file.sync_all() {
                        error!("Failed to sync_all to file: {} (write_content()) | Message: {:?}",  installer_path.display(), e);
                    }
                }
               Err(e) => {
                    error!("Failed to read content as bytes! (write_content()) | Message: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to create temporary file: {} (write_content()) | Message: {:?}", installer_path.display(), e)
        }
    }
}

/// Downloads and attempts to run the downloaded file.
pub async fn download_and_run(url: &str) -> Result<(), String> {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("monarch");
    tmp_dir.push("downloads");
    
    if let Err(e) = create_dir(tmp_dir.clone()) {
        error!("Failed to create new directory: {} (download_and_run()) | Message: {:?}", tmp_dir.display(), e);
        return Err("Failed to create temporary directory!".to_string())
    }

    match reqwest::get(url).await {
        Ok(response) => {
            
            let installer_path: PathBuf = create_file_path(&response, &tmp_dir).await;

            info!("Downloading to: {}", installer_path.display());
            write_content(&installer_path, response).await;

           return run_file(installer_path) // Returns Result<(), String> indicating how execution went
            
        }
        Err(e) => {
            error!("Failed to get response from: {} | Message: {:?}", url, e);
            return Err("Failed to get response from url!".to_string())
        }
    }
}

pub async fn download_file(url: &str) -> Result<PathBuf, String> {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("monarch");
    tmp_dir.push("downloads");
    
    if let Err(e) = create_dir(tmp_dir.clone()) {
        error!("Failed to create new directory: {} (download_file()) | Message: {:?}", tmp_dir.display(), e);
        return Err("Failed to create temporary directory!".to_string())
    }

    match reqwest::get(url).await {
        Ok(response) => {
            
            let installer_path: PathBuf = create_file_path(&response, &tmp_dir).await;

            info!("Downloading to: {}", installer_path.display());
            write_content(&installer_path, response).await;
            return Ok(installer_path)
            
        }
        Err(e) => {
            error!("Failed to get response from: {} | Message: {:?}", url, e);
            return Err("Failed to get response from url!".to_string())
        }
    }
}

/*
---------- Download images for games ----------
*/

/// Tells Monarch to attempt to download url content as image
pub async fn download_image(url: &str, path: PathBuf) {
    let request: Result<Response, reqwest::Error> = reqwest::get(url).await;
    let thumbnail_path: PathBuf = PathBuf::from(path);

    match request {
        Ok(response) => {
            get_image_content(response, thumbnail_path).await;
        }
        Err(e) => {
            error!("Failed to download image file! Url:{} (download_image()) | Message: {:?}", url, e);
        }
    }   
}

/// Saves the content from response to file
async fn get_image_content(response: Response, path: PathBuf) {
    match response.bytes().await {
        Ok(image_bytes) => {
            match image::load_from_memory(&image_bytes) {
                Ok(image) => {
                    if let Err(e) = image.save(path) {
                        error!("Failed to save image file! (get_image_content()) | Message: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to read image from bytes! (get_image_content()) | Message: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to read image as bytes! (get_image_content()) | Message: {:?}", e);
        }
    }
}