# Monarch EGS lib

## Good URLs

### Getting "all assets"
launcher-public-service-prod06.ol.epicgames.com

```rust
```
/// An asset is essentially a game on the Epic Games Store
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "appName")]
    pub app_id: String,

    #[serde(rename = "buildVersion")]
    pub build_version: String,

    #[serde(rename = "catalogItemId")]
    pub catalog_id: String,

    pub namespace: String,

    #[serde(rename = "assetId")]
    pub asset_id: String,
}
```
```

### Getting game metadata
catalog-public-service-prod06.ol.epicgames.com
