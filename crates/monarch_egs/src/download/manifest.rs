use reqwest::Client;

pub struct Manifest {}

static MANIFEST_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";

/// Returns a download manifest for Epic Games game of namespace
pub async fn get_game_manifest(
    namespace: &str,
    app_name: &str,
    catalog_id: &str,
    platform: Option<&str>,
    label: Option<&str>,
) -> Manifest {
    let platform_: &str = platform.unwrap_or("Windows");
    let label_: &str = label.unwrap_or("Live");

    let url: String = format!(
        "https://{MANIFEST_URL}/launcher/api/public/assets/v2/platform/{platform_}/namespace/{namespace}/catalogItem/{catalog_id}/app/{app_name}/label/{label_}",
    );

    let client: Client = Client::new();

    let response = client.get(&url).send().await.unwrap();

    println!("resp: {:?}", response);

    let text = response.text().await.unwrap();
    println!("text: {}", text);

    Manifest {}
}
