use anyhow::{Context, Result};
use image::ImageFormat;
use image::ImageReader;
use reqwest::Response;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

use crate::monarch_utils::monarch_fs;
use crate::monarch_utils::monarch_settings::Settings;

use super::monarch_http;

/*
---------- Download images for games ----------
*/

/// Tells Monarch to attempt to download url content as image
pub async fn download_image(
    settings_handle: Arc<RwLock<Settings>>,
    url: &str,
    path: &Path,
) -> Result<()> {
    let response: Response = monarch_http::download_client()
        .get(url)
        .send()
        .await
        .with_context(|| {
            format!("monarch_download::download_image() Error while downloading: {url} | Err: ")
        })?;

    save_image_content(settings_handle, response, path)
        .await
        .with_context(|| "monarch_download::download_image() -> ")?;
    Ok(())
}

/// Saves the content from response to file
async fn save_image_content(
    settings_handle: Arc<RwLock<Settings>>,
    response: Response,
    path: &Path,
) -> Result<()> {
    let temp_dir = monarch_fs::get_temp_dir(settings_handle);
    if !temp_dir.exists() {
        monarch_fs::create_dir(&temp_dir)
            .with_context(|| "monarch_download::save_image_content() -> ")?;
    }
    let temp_file = temp_dir.join(path.file_name().unwrap());

    // Download image content
    let bytes = response
        .bytes()
        .await
        .with_context(|| "monarch_download::save_image_content() Failed to read bytes! | Err")?;

    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .with_context(|| "monarch_download::save_image_content() Error guessing format! | Err: ")?
        .decode()
        .with_context(|| "monarch_download::save_image_content() Error decoding image! | Err: ")?;

    // Write to a temporary file
    let file = std::fs::File::create(&temp_file)
        .with_context(|| "monarch_download::save_image_content() Error creating file! | Err: ")?;

    img.write_to(&file, ImageFormat::Png)
        .with_context(|| "monarch_download::save_image_content() Error writing to file. | Err: ")?;

    // Atomically generate the file in the correct location
    // by copying.
    std::fs::rename(temp_file, path)
        .with_context(|| "monarch_download::save_image_content() std::fs::rename failed! | Err: ")
}
