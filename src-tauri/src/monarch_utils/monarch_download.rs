use reqwest;
use reqwest::Response;
use std::env;
use std::fs::{File, create_dir};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use log::{info, error};
use image;

use super::monarch_web::request_data;

/// Creates a tmp path name for file to install.
async fn create_file_path(response: &Response, tmp_dir: &PathBuf) -> PathBuf {
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    // If file doesnt have an extension, assume its an executable.
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
    let mut f = File::create(installer_path).unwrap();
    f.write_all(&content.bytes().await.unwrap()).unwrap();
    f.sync_all().unwrap();
}

/// Downloads and attempts to run the downloaded file.
pub async fn download_and_run(url: &str) {
    let system_tmp_dir = env::temp_dir();
    let tmp_dir: PathBuf = [system_tmp_dir.to_str().unwrap(), r"moose_launcher_download\"].iter().collect();
    let _ = create_dir(&tmp_dir);

    if let Ok(response) = request_data(url).await {
        let installer_path = create_file_path(&response, &tmp_dir).await;

        info!("Downloading to: {}", installer_path.display());
        write_content(&installer_path, response).await;

        let result = Command::new("PowerShell")
            .arg(&installer_path.to_str().unwrap())
            .spawn();

        match result {
            Ok(_) => { info!("Executing '{}'", installer_path.display()) }
            Err(err) => { error!("Failed to run '{}' | Message: {:?}", installer_path.display(), err) }
        }
    }
}

/*
---------- Download images for games ----------
*/

pub async fn download_image(url: &str, path: &str) {
    let response = request_data(url).await;

    match response{
        Ok(content) => {
            let thumbnail_path = PathBuf::from(path);
            println!("{} : {}", path, url);
            if let Ok(img_bytes) = content.bytes().await {
                let image = image::load_from_memory(&img_bytes).unwrap();
                image.save(thumbnail_path).unwrap();
            }
        }
        Err(e) => {
            error!("Failed to download image file! Url:{} | Message: {:?}", url, e);
        }
    }
    
}