use reqwest;
use reqwest::Response;

/// Gets data from url.
pub async fn request_data(url: &str) -> Response {
    let target = url;
    return reqwest::get(target).await.unwrap()
}