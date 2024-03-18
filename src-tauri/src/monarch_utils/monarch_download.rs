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
    let request: Result<Response, reqwest::Error> = reqwest::get(url).await;
    let response = request.with_context(|| -> String {
        format!("monarch_download::download_image() failed! Error while downloading: {url} | Err: ")
    })?;

    save_image_content(response, path).await?;
    Ok(())
}

/// Saves the content from response to file
async fn save_image_content(response: Response, path: &Path) -> Result<()> {
    let bytes = response.bytes().await.with_context(|| -> String {
        "monarch_download::save_image_content() failed! Failed to read bytes! | Err".to_string()
    })?;
    let mut file = std::fs::File::create(path).with_context(|| -> String {
        "monarch_download::save_image_content() failed! Failed to create new file! | Err: "
            .to_string()
    })?;
    file.write_all(&bytes)?;
    Ok(())
}
