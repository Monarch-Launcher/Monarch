use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::stores::StoreType;
use crate::{
    monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame},
    monarch_utils::monarch_state::MonarchState,
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
    async fn launch(&self, state_handle: Arc<RwLock<MonarchState>>) -> Result<()>;
    fn into_monarchgame(&self) -> MonarchGame;
}

#[async_trait]
pub trait SearchResult: Send + Sync {
    fn to_search_result(&self) -> MonarchWebApiGame;
    fn into_monarchgame(&self) -> MonarchGame;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatOptions {
    // Gamemode
    pub gamemoderun: bool,

    // Gamescope
    pub gamescope_enable: bool,
    pub gamescope_width: String,
    pub gamescope_height: String,
    pub gamescope_refresh: String,
    pub gamescope_hdr: bool,
    pub gamescope_expose_wayland: bool,
    pub gamescope_vrr: bool,
    pub gamescope_force_grab_cursor: bool,

    // Misc
    pub mango_hud: bool,

    // Proton
    pub proton_use_wined3d: bool,
    pub proton_no_d3d12: bool,
    pub proton_no_d3d11: bool,
    pub proton_no_d3d10: bool,
    pub proton_use_esync: bool,
    pub proton_use_fsync: bool,
    pub proton_use_ntsync: bool,
    pub proton_no_esync: bool,
    pub proton_no_fsync: bool,
    pub proton_no_ntsync: bool,
    pub proton_enable_nvapi: bool,
    pub proton_use_seccomp: bool,
    pub proton_use_sdl: bool,
    pub proton_use_wayland: bool,
    pub proton_enable_wayland: bool,
    pub proton_use_xalia: bool,

    // Wine
    pub wine_prefix: String,
    pub wine_dll_overrides: String,
    pub wine_esync: bool,
    pub wine_fsync: bool,
    pub wine_fullscreen_fsr: bool,
    pub wine_large_address_aware: bool,

    // DXVK
    pub dxvk_hud: bool,

    // User added arguments
    pub user_envs: HashMap<String, String>,
    pub user_args: Vec<UserLaunchArgs>,
}

// This struct allows the user to manipulate the launch command in the text field and
// for Monarch to still keep it in the correct order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserLaunchArgs {
    pub comes_after: String,
    pub value: String,
}
