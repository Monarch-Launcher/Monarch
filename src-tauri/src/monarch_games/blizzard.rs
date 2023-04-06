use log::{info, error};
use std::io;
use std::io::Error;
use std::process::{Command, Child};

use crate::monarch_utils::{winreg_searcher::is_installed, downloader::download_and_run};

/*
This is hopefully not a long time solution. For now running battlenet://<game> only opens battlenet page and doesn't run game.
Here are game codes:
    Hearthstone: WTCG
    Diablo 3: D3
    Starcraft 2: SC2
    World of Warcraft: WoW
    Heroes of the Storm: Hero
    Overwatch: Pro
    Destiny 2: DST2
*/

/*
---------- Public Blizzard related functions ----------
*/

/// Installs Battle.net launcher
pub async fn get_blizzard() {
    let is_installed: bool = blizzard_is_installed();

    if is_installed {
        info!("Battle.net already installed!");
    }
    else {
        let target_url: &str = "https://eu.battle.net/download/getInstaller?os=win&installer=Battle.net-Setup.exe";
        download_and_run(target_url).await;
    }
}

/// Attempts to run Blizzard game, returns Ok() or Err()
pub fn launch_game(name: &str) -> io::Result<()> {
    // Convert name to id, somehow

    let mut game_command: String = String::from("battlenet://");
    game_command.push_str(id);

    let exec_result: Result<Child, Error> = Command::new("PowerShell")
                                                    .arg("start")
                                                    .arg(&game_command)
                                                    .spawn(); // Run steam installer for specified game
    match exec_result {
        Ok(_) => {
            info!("Launching game: {}", game_command);
            Ok(())
        }
        Err(e) => {
            error!("Failed to launch game: {} | Message: {:?}", id, e);
            Err(e)
        }
    }
}

/*
----------- Private Blizzard related function -----------
*/

/// Sepcifically checks if Battle.net is installed
fn blizzard_is_installed() -> bool {
    return is_installed(r"Blizzard Entertainment\Battle.net");
}