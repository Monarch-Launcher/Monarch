use anyhow::{Context, Result};
use image::ImageFormat;
use image::ImageReader;
use reqwest::Response;
use std::io::Cursor;
use std::path::Path;

use super::monarch_http;

/*
---------- Download images for games ----------
*/

/// Tells Monarch to attempt to download url content as image
pub async fn download_image(url: &str, path: &Path) -> Result<()> {
    let response: Response = monarch_http::download_client()
        .get(url)
        .send()
        .await
        .with_context(|| {
            format!("monarch_download::download_image() Error while downloading: {url} | Err: ")
        })?;

    save_image_content(response, path)
        .await
        .with_context(|| "monarch_download::download_image() -> ")?;
    Ok(())
}

/// Saves the content from response to file
async fn save_image_content(response: Response, path: &Path) -> Result<()> {
    let bytes = response
        .bytes()
        .await
        .with_context(|| "monarch_download::save_image_content() Failed to read bytes! | Err")?;

    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .with_context(|| "monarch_download::save_image_content() Error guessing format! | Err: ")?
        .decode()
        .with_context(|| "monarch_download::save_image_content() Error decoding image! | Err: ")?;

    let file = std::fs::File::create(path)
        .with_context(|| "monarch_download::save_image_content() Error creating file! | Err: ")?;

    img.write_to(file, ImageFormat::Png)
        .with_context(|| "monarch_download::save_image_content() Error writing to file. | Err: ")?;
    Ok(())
}
