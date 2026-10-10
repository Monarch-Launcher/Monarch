use std::sync::{Arc, RwLock};

use super::stores::StoreType;
use crate::{
    monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame},
    monarch_utils::monarch_settings::Settings,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait GameType: Send + Sync {
    fn get_name(&self) -> String;
    fn get_store(&self) -> Box<dyn StoreType>;
    fn get_store_name(&self) -> String;
    fn get_store_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> f64;
    async fn launch(&self, settings_handle: Arc<RwLock<Settings>>) -> Result<()>;
    fn into_monarchgame(&self) -> MonarchGame;
}

#[async_trait]
pub trait SearchResult: Send + Sync {
    fn to_search_result(&self) -> MonarchWebApiGame;
    fn into_monarchgame(&self) -> MonarchGame;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatOptions {
    // Proton
    pub proton_use_wined3d: bool,
    pub proton_no_d3d11: bool,
    pub proton_no_d3d10: bool,
    pub proton_dxvk_d3d8: bool,
    pub proton_no_fsync: bool,
    pub proton_no_ntsync: bool,
    pub proton_disable_nvapi: bool,
    pub proton_enable_nvapi: bool,
    pub proton_use_seccomp: bool,
    pub proton_use_sdl: bool,
    pub proton_prefer_sdl: bool,
    pub proton_use_wayland: bool,
    pub proton_enable_wayland: bool,
    pub proton_use_xalia: bool,

    // Wine
    pub wine_prefix: String,
    pub wine_arch: String,
    pub wine_debug: String,
    pub wine_dll_overrides: String,
    pub wine_server: String,
    pub wine_loader: String,
    pub wine_dll_path: String,
    pub wine_esync: bool,
    pub wine_fsync: bool,
    pub wine_fullscreen_fsr: bool,
    pub wine_fullscreen_fsr_strength: String,
    pub wine_fullscreen_integer_scaling: bool,
    pub wine_use_kwin_hacks: bool,

    // DXVK
    pub dxvk_hud: bool,
    pub dxvk_hud_custom: String,
    pub dxvk_config: String,
    pub dxvk_shader_cache_disabled: bool,
    pub dxvk_shader_cache_path: String,

    // Gamemode
    pub gamemode_auto: bool,

    // Gamescope
    pub gamescope_enable: bool,
    pub gamescope_width: String,
    pub gamescope_height: String,
    pub gamescope_refresh: String,
    pub gamescope_hdr: bool,
    pub gamescope_expose_wayland: bool,
    pub gamescope_vrr: bool,
    pub gamescope_force_grab_cursor: bool,
    pub gamescope_wsi: bool,
    pub gamescope_fsr_strength: String,

    // Misc
    pub mango_hud: bool,
}
