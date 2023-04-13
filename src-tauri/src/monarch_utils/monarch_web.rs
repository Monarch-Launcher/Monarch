use reqwest;
use reqwest::{Response, Result};

/// Gets data from url.
pub async fn request_data(url: &str) -> Result<Response> {
    return reqwest::get(url).await;
}