use anyhow::{Context, Result};
use image;
use log::{error, info};
use reqwest;
use reqwest::Response;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::monarch_fs::create_dir;

/// Creates a tmp path name for the downloaded file.
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
    } else {
        tmp_dir.join(fname)
    }
}

/// Writes downloaded content to file, has to be it's own function to
/// close file and avoid "file used by another process" error.
// Also I am aware that this might look ugly due to the nesting, might come back and fix later...
async fn write_content(installer_path: &PathBuf, content: Response) -> Result<()> {
    let mut file: File = File::create(installer_path).with_context(||
        -> String {format!("monarch_download::write_content() failed! Error while creating temporary file: {file} | Err", file = installer_path.display())})?;

    let bytes = content.bytes().await.with_context(|| -> String {
        format!("monarch_download::write_content() failed! Error while reading bytes! | Err")
    })?;

    file.write_all(&bytes).context(
        "monarch_download::write_content() failed! Error while writing bytes to file! | Err",
    ) // Wrap return in extra context for logs
}

pub async fn download_file(url: &str) -> Result<PathBuf> {
    let mut tmp_dir: PathBuf = env::temp_dir();
    tmp_dir.push("monarch");
    tmp_dir.push("downloads");

    create_dir(&tmp_dir).context(format!(
        "monarch_download::download_file() failed! Error while creating new directory: {dir} | Err",
        dir = tmp_dir.display()
    ))?;

    let response: Response = reqwest::get(url).await.with_context(|| -> String {
        format!("monarch_download::download_file() failed! No/bad response from: {url} | Err")
    })?;

    let installer_path: PathBuf = create_file_path(&response, &tmp_dir).await;

    info!("Downloading to: {}", installer_path.display());
    write_content(&installer_path, response).await.context(format!("monarch_download::download_file() failed! Error returned when writing content to: {path} | Err", path = installer_path.display()))?;
    return Ok(installer_path);
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
            get_image_content(response, &thumbnail_path).await;
        }
        Err(e) => {
            error!("monarch_download::download_image() failed! Error while downloading: {url} | Error: {e}");
        }
    }
}

/// Saves the content from response to file
async fn get_image_content(response: Response, path: &Path) {
    match response.bytes().await {
        Ok(image_bytes) => match image::load_from_memory(&image_bytes) {
            Ok(image) => {
                if let Err(e) = image.save(path) {
                    error!("monarch_download::get_image_content() failed! Failed to save image: {file} | Error: {e}", file = path.display());
                }
            }
            Err(e) => {
                error!("monarch_download::get_image_content() failed! Failed to parse bytes as image! | Error: {e}");
            }
        },
        Err(e) => {
            error!(
                "monarch_download::get_image_content() failed! Failed to read bytes! | Error: {e}"
            );
        }
    }
}
