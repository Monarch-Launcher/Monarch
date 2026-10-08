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
    proton_use_wined3d: bool,
    proton_no_d3d11: bool,
    proton_no_d3d10: bool,
    proton_dxvk_d3d8: bool,
    proton_no_fsync: bool,
    proton_no_ntsync: bool,
    proton_disable_nvapi: bool,
    proton_enable_nvapi: bool,
    proton_use_seccomp: bool,
    proton_use_sdl: bool,
    proton_prefer_sdl: bool,
    proton_use_wayland: bool,
    proton_enable_wayland: bool,
    proton_use_xalia: bool,

    // Wine
    wine_prefix: String,
    wine_arch: String,
    wine_debug: String,
    wine_dll_overrides: String,
    wine_server: String,
    wine_loader: String,
    wine_dll_path: String,
    wine_esync: bool,
    wine_fsync: bool,
    wine_fullscreen_fsr: bool,
    wine_fullscreen_fsr_strength: String,
    wine_fullscreen_integer_scaling: bool,
    wine_use_kwin_hacks: bool,

    // DXVK
    dxvk_hud: bool,
    dxvk_hud_custom: String,
    dxvk_config: String,
    dxvk_shader_cache_disabled: bool,
    dxvk_shader_cache_path: String,

    // Gamemode
    gamemode_auto: bool,

    // Gamescope
    gamescope_enable: bool,
    gamescope_width: String,
    gamescope_height: String,
    gamescope_refresh: String,
    gamescope_hdr: bool,
    gamescope_expose_wayland: bool,
    gamescope_vrr: bool,
    gamescope_force_grab_cursor: bool,
    gamescope_wsi: bool,
    gamescope_fsr_strength: String,

    // Misc
    mango_hud: bool,
}
