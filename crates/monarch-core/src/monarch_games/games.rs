use std::sync::{Arc, RwLock};

use super::stores::StoreType;
use crate::{
    monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame},
    monarch_utils::monarch_settings::Settings,
};
use anyhow::Result;
use async_trait::async_trait;

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

pub struct LaunchArgs {
    // Proton related options
    ProtonLog: bool,
    ProtonLogDir: String,
    ProtonCrashReportDir: String,
    ProtonWaitAttach: bool,
    ProtonUseWined3d: bool,
    ProtonNoD3D11: bool,
    ProtonNoD3D10: bool,
    ProtonDxvkD3D8: bool,
    ProtonNoFsync: bool,
    ProtonNoNtsync: bool,
    ProtonDisableNvapi: bool,
    ProtonForceLargeAddressAware: bool,
    ProtonHeapDelayFree: bool,
    ProtonUseXalia: bool,
    HostLcAll: String,
    Fna3dForceDriver: String,

    // Wine related options
    WinePrefix: String,
    WineArch: String,
    WineDebug: String,
    WineDllOverrides: String,
    WineServer: String,
    WineLoader: String,
    WineDllPath: String,
    WineEsync: bool,
    WineFsync: bool,
    WineFullscreenFsr: bool,
    WineFullscreenFsrStrength: u8,
    WineFullscreenIntegerScaling: bool,
    WineUseKwinHacks: bool,

    // DXVK
    DxvkHud: bool,
    DxvkHudCustom: String,
    DxvkLogLevel: String,
    DxvkLogPath: String,
    DxvkConfigFile: String,
    DxvkConfig: String,
    DxvkFilterDeviceName: String,
    DxvkFilterDeviceUuid: String,
    DxvkDebug: String,
    DxvkShaderCacheDisabled: bool,
    DxvkShaderCachePath: bool,

    // Gamescope/Gamemode
    GamemodeAuto: bool,
    GamescopeWsi: bool,
    GamescopeFsrStrength: String,

    //Misc
    MangoHud: bool,
}
