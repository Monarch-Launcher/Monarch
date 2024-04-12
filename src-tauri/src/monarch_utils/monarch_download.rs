use anyhow::{Context, Result};
use reqwest;
use reqwest::Response;
use std::io::Write;
use std::path::Path;

/*
---------- Download images for games ----------
*/

/// Tells Monarch to attempt to download url content as image
pub async fn download_image(url: &str, path: &Path) -> Result<()> {
    let response: Response = reqwest::get(url).await.with_context(|| {
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
    let mut file = std::fs::File::create(path).with_context(|| {
        "monarch_download::save_image_content() Failed to create new file! | Err: "
    })?;
    file.write_all(&bytes)
        .with_context(|| "monarch_download::save_image_content() Error writing to file. | Err: ")?;
    Ok(())
}
