use log::info;

use crate::monarch_utils::{monarch_winreg::is_installed, monarch_download::download_and_run};

/// Installs Epic games launcher if not already installed
pub async fn get_epic() {
    let is_installed: bool = epic_is_installed();

    if is_installed {
        info!("Epic Games already installed!");
    }
    else {
        let target_url: &str = "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/installer/download/EpicGamesLauncherInstaller.msi";
        download_and_run(target_url).await;
    }
}

/// Returns whether or not Epic games launcehr is installed
fn epic_is_installed() -> bool {
    return is_installed(r"Epic Games\EpicGamesLauncher");
}
