use anyhow::{bail, Context, Result};
use reqwest;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tar::Archive;
use tracing::{debug, error, info, warn};

use crate::{
    monarch_games::{
        games::{CompatOptions, GameType},
        monarchgame::MonarchGame,
        LaunchCommand,
    },
    monarch_library::library,
    monarch_utils::{
        monarch_fs::{self, get_monarch_bins_path, get_monarch_home},
        monarch_settings::Settings,
        monarch_state::MonarchState,
        monarch_terminal,
    },
};

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Returns path to directory where Monarch stores its copy of the umu-launcher binary.
fn get_umu_dir() -> PathBuf {
    get_monarch_bins_path()
        .expect("Don't expect to crash")
        .join("umu")
}

/// Returns path to umu-launcher binary.
pub fn get_umu_exe() -> PathBuf {
    if let Some(p) = monarch_fs::find_linux_binary("umu-run") {
        return p;
    }
    get_umu_dir().join("umu-run")
}

/// For now a simple check to verify that umu-launcher exists.
pub fn umu_is_installed() -> bool {
    if monarch_fs::find_linux_binary("umu-run").is_some() {
        return true;
    }

    let umu_path = get_umu_dir();
    if !umu_path.exists() {
        return false;
    }
    get_umu_exe().exists()
}

pub fn remove_umu() -> Result<()> {
    if !umu_is_installed() {
        warn!("linux::remove_umu() Umu not found!");
        bail!("Umu not found!")
    }

    std::fs::remove_dir_all(&get_umu_dir())
        .with_context(|| "linux::remove_umu() Failed to remove_dir_all() | Err: ")
}

/// Installs the umu-launcher by downloading the binary to $MONARCH_HOME/umu/umu-run
pub fn install_umu(settings_handle: Arc<RwLock<Settings>>) -> Result<()> {
    if umu_is_installed() {
        bail!("linux::umu::install_umu() Failed to install umu-launcher! | Err: Umu path already exists.")
    }

    info!("Getting umu-launcher releases...");
    let umu_release_url: &str =
        "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest";

    let client = reqwest::blocking::Client::new();
    let release_response = client
        .get(umu_release_url)
        .header("User-Agent", "Monarch/1.0")
        .send()?;

    let release_text: String = release_response.text()
        .with_context(|| "linux::umu::install_umu() Failed to get response text from umu-launcher release page! | Err: ")?;

    let release_data: Release = serde_json::from_str(&release_text).with_context(|| {
        "linux::umu::install_umu() Failed to parse response from umu-launcher release page! | Err: "
    })?;

    info!("Using release: {}", release_data.tag_name);

    let asset = release_data
        .assets
        .into_iter()
        .find(|a| a.name.contains("zipapp") && a.name.ends_with(".tar"))
        .ok_or("No matching asset found")
        .unwrap();

    info!("Downloading : {}", &asset.browser_download_url);

    let mut download_response =
        reqwest::blocking::get(&asset.browser_download_url).with_context(|| {
            format!(
                "linux::umu::install_umu() Failed to get response from {} | Err: ",
                &asset.browser_download_url
            )
        })?;
    let dest_path: PathBuf = get_monarch_home(settings_handle.clone()).join(asset.name);
    let mut dest = std::fs::File::create(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to create: {} | Err: ",
            get_umu_dir().display()
        )
    })?;

    info!("Writing umu-launcher to: {}...", dest_path.display());
    std::io::copy(&mut download_response, &mut dest)
        .with_context(|| "linux::umu::install_umu() Failed to copy response to file! | Err: ")?;

    info!("Unpacking: {}...", dest_path.display());
    let mut archive = Archive::new(std::fs::File::open(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to open {} | Err: ",
            dest_path.display()
        )
    })?);
    archive
        .unpack(get_monarch_home(settings_handle))
        .with_context(|| {
            format!(
                "linux::umu::install_umu() Failed to unpack {}! | Err: ",
                dest_path.display()
            )
        })?;

    info!("Finished downloading umu-launcher.");

    info!("Removing: {}...", dest_path.display());
    std::fs::remove_file(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to remove: {} | Err: ",
            dest_path.display()
        )
    })?;

    Ok(())
}

/// Executes the game using umu-launcher to run in proton.
pub async fn umu_run(game: &MonarchGame, state_handle: Arc<RwLock<MonarchState>>) -> Result<()> {
    let mut launch_command: LaunchCommand =
        generate_launch_command(&game, state_handle.clone(), None)
            .await
            .with_context(|| "linux::umu::umu_run() -> ")?;

    // Add some extra env vars that help launching the game
    let store_arg = match game.get_store_name().as_str() {
        "epic" | "epicgames" => "egs".to_string(),
        _ => "none".to_string(),
    };

    let settings_handle = match state_handle.read() {
        Ok(state) => state.get_settings_ptr(),
        Err(e) => {
            bail!("linux::umu::umu_run() Failed to acquire read lock on state_handle! | Err: {e}")
        }
    };

    let gameid_arg = format!("umu-{}", game.get_store_id());
    let prefix = monarch_fs::wine_prefix_dir(settings_handle, &gameid_arg);
    std::fs::create_dir_all(&prefix).with_context(|| {
        format!(
            "linux::umu::umu_run() Failed to create wine prefix {} | Err: ",
            prefix.display()
        )
    })?;

    let extra_env: [(String, String); 7] = [
        (
            "PROTONPATH".to_string(),
            game.compatibility.as_ref().unwrap().to_string(),
        ),
        ("GAMEID".to_string(), gameid_arg.clone()),
        ("STORE".to_string(), store_arg),
        (
            "WINEPREFIX".to_string(),
            prefix.to_string_lossy().to_string(),
        ),
        (
            "STEAM_COMPAT_DATA_PATH".to_string(),
            prefix.to_string_lossy().to_string(),
        ),
        (
            "STEAM_COMPAT_INSTALL_PATH".to_string(),
            launch_command.work_dir.clone(),
        ),
        (
            "WINEDLLOVERRIDES".to_string(),
            "winemenubuilder.exe=d".to_string(),
        ),
    ];

    for (k, v) in extra_env {
        // Don't override user overriden vars
        if launch_command.env.contains_key(&k) {
            continue;
        }
        launch_command.env.insert(k, v);
    }

    launch_command
        .env
        .entry("LD_PRELOAD".to_string())
        .or_default();

    debug!("Working dir: {}", &launch_command.work_dir);
    debug!("Env vars: {:?}", launch_command.env);
    debug!("Launch command: {}", &launch_command.command);

    let rx = monarch_terminal::spawn_terminal(
        launch_command.command,
        launch_command.env,
        Some(launch_command.work_dir),
    );

    if let Err(e) = rx.await {
        error!("linux::umu::umu_run() Terminal command failed! | Err: {e}");
    }

    Ok(())
}

/// Generate the launch command for a given MonarchGame.
/// Abstracted to be used both internally for launching
/// as well as displaying the launch command to the user.
pub async fn generate_launch_command(
    game_ref: &MonarchGame,
    state_handle: Arc<RwLock<MonarchState>>,
    exe_path: Option<&str>,
) -> Result<LaunchCommand> {
    // Clone to not do any permanent changes to game, even when just running
    // this function to generate a user-readable launch command.
    let mut game: MonarchGame = game_ref.clone();

    // Use user specified compat layer *or* default to Proton-GE
    let compatibility: &str = match &game_ref.compatibility {
        Some(compat) => compat,
        None => "Proton-GE-Latest",
    };
    info!("Compatibility layer set: {compatibility}");

    let exe = PathBuf::from(game.executable_path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "linux::umu::umu_run() No executable path set for {}",
            game.name
        )
    })?);
    if !exe.is_file() {
        bail!("Executable not found: {}", exe.display());
    }

    let db_pool: Arc<SqlitePool>;

    match state_handle.read() {
        Ok(state) => {
            db_pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("linux::umu::umu_run() Failed to acquire lock on state_handle! | Err: {e}");
        }
    }

    let work_dir = resolve_install_dir(&game, &exe);
    let (work_dir, exe) = ensure_wine_safe_game_paths(&mut game, work_dir, exe, db_pool).await?;

    // Mount the game directory inside the Steam Runtime container and give
    let env: HashMap<String, String> = HashMap::new();
    let exe_arg: String = match exe_path {
        Some(path) => path.to_string(),
        None => monarch_fs::relative_launch_exe_arg(&exe, &work_dir),
    };
    let umu: PathBuf = get_umu_exe();

    let command: String = format!("{} {}", umu.display(), exe_arg);
    let mut launch_command: LaunchCommand = LaunchCommand {
        command,
        work_dir: work_dir.to_string_lossy().to_string(),
        env,
    };

    if game.compatibility_opts.is_some() {
        apply_compat_options_to_launch(&game, &mut launch_command);
    }

    /*
     * --------------------- Game specific options ---------------------
     *
     * These apply regardless of whether compatibility options are set.
     */
    // Env args are prepended to the very front of the command, e.g.
    // "PROTON_ENABLE_WAYLAND=1 ..." (Steam passes them the same way).
    apply_env_args(&mut launch_command.env, game.env_args.as_deref());

    // Game specific launch args are appended at the very end of the launch
    // command. For Steam games this matches whatever comes after %command%.
    if let Some(args) = game.launch_args.as_deref() {
        let args = args.trim();
        if !args.is_empty() {
            launch_command.command = format!("{} {}", launch_command.command, args);
        }
    }

    Ok(launch_command)
}

/// Resolve the on-disk install directory for a game.
fn resolve_install_dir(game: &MonarchGame, exe: &Path) -> PathBuf {
    let recorded = game.properties.install_dir.trim();
    if !recorded.is_empty() && recorded != "Error" {
        let path = PathBuf::from(recorded);
        if path.is_dir() {
            return path;
        }
    }
    exe.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Rename the install folder when needed for Wine, then update the game record.
async fn ensure_wine_safe_game_paths(
    game: &mut MonarchGame,
    install_dir: PathBuf,
    exe: PathBuf,
    db_pool: Arc<SqlitePool>,
) -> Result<(PathBuf, PathBuf)> {
    let Some(safe_dir) = monarch_fs::ensure_wine_safe_install_dir(&install_dir)? else {
        return Ok((install_dir, exe));
    };

    let rel_exe = exe
        .strip_prefix(&install_dir)
        .unwrap_or(Path::new(exe.file_name().unwrap_or_default()));
    let safe_exe = safe_dir.join(rel_exe);

    game.properties.install_dir = safe_dir.to_string_lossy().to_string();
    game.executable_path = Some(safe_exe.to_string_lossy().to_string());

    if let Err(e) = library::update_game_properties_in_db(db_pool, game).await {
        error!("linux::umu::ensure_wine_safe_game_paths() Failed to persist renamed install paths | Err: {e}");
    }

    Ok((safe_dir, safe_exe))
}

/// Reads the game.compatibility_options field and applies all of the
/// options to the correct places before launch.
///
/// NOTE: She ain't pretty, but she gets the job done
///
/// It might look stupid at first, but doing the command this way enables
/// Monarch to simply perform quick and easy bool/string checks and appends
/// instead of having to do more complex string manipulations to add and remove
/// snippets when needed.
fn apply_compat_options_to_launch(game: &MonarchGame, launch_command: &mut LaunchCommand) {
    let compat_options: &CompatOptions = game.compatibility_opts.as_ref().unwrap();

    /*
     * --------------------- Gamemode options ---------------------
     */
    if compat_options.gamemoderun {
        // Arbitrary user-supplied arguments for the gamemoderun wrapper
        // itself, e.g. "-a <FAST_GAMEMODE_SECONDS>", go right after the
        // wrapper and before the actual command.
        let args = compat_options.gamemode_args.trim();
        if args.is_empty() {
            launch_command.command = format!("gamemoderun {}", launch_command.command);
        } else {
            launch_command.command = format!("gamemoderun {} {}", args, launch_command.command);
        }
    }

    /*
     * --------------------- Gamescope options ---------------------
     */
    if compat_options.gamescope_enable {
        let mut gamescope_args: String = String::new();

        if !compat_options.gamescope_width.is_empty() {
            gamescope_args.push_str(&format!("-W {} ", compat_options.gamescope_width));
        }

        if !compat_options.gamescope_height.is_empty() {
            gamescope_args.push_str(&format!("-H {} ", compat_options.gamescope_height));
        }

        if !compat_options.gamescope_refresh.is_empty() {
            gamescope_args.push_str(&format!("-r {} ", compat_options.gamescope_refresh));
        }

        if compat_options.gamescope_hdr {
            gamescope_args.push_str("--hdr-enabled ");
        }

        if compat_options.gamescope_expose_wayland {
            gamescope_args.push_str("--expose-wayland ");
        }

        if compat_options.gamescope_vrr {
            gamescope_args.push_str("--adaptive-sync ");
        }

        if compat_options.gamescope_force_grab_cursor {
            gamescope_args.push_str("--force-grab-cursor ");
        }

        if compat_options.mango_hud {
            gamescope_args.push_str("--mangoapp ");
        }

        // Arbitrary user-supplied gamescope arguments, appended after the
        // built-in flags and before the "-- <command>" part.
        let user_gamescope_args = compat_options.gamescope_args.trim();
        if !user_gamescope_args.is_empty() {
            gamescope_args.push_str(user_gamescope_args);
            gamescope_args.push(' ');
        }

        launch_command.command =
            format!("gamescope {}-- {}", gamescope_args, launch_command.command)
    }

    /*
     * --------------------- Misc options ---------------------
     */
    // Mangohud can only be started this way if not running under gamescope
    // according to: https://wiki.archlinux.org/title/Gamescope
    if compat_options.mango_hud && !compat_options.gamescope_enable {
        launch_command.command = format!("mangohud {}", launch_command.command);
    }

    // Arbitrary user-supplied misc arguments, e.g. MANGOHUD_CONFIG=fps,cpu.
    apply_arbitrary_args(launch_command, &compat_options.misc_args);

    /*
     * --------------------- Proton options ---------------------
     */
    if compat_options.proton_use_wined3d {
        launch_command
            .env
            .insert("PROTON_USE_WINED3D".to_string(), "1".to_string());
    }

    if compat_options.proton_no_d3d12 {
        launch_command
            .env
            .insert("PROTON_NO_D3D12".to_string(), "1".to_string());
    }

    if compat_options.proton_no_d3d11 {
        launch_command
            .env
            .insert("PROTON_NO_D3D11".to_string(), "1".to_string());
    }

    if compat_options.proton_no_d3d10 {
        launch_command
            .env
            .insert("PROTON_NO_D3D10".to_string(), "1".to_string());
    }

    if compat_options.proton_use_esync {
        launch_command
            .env
            .insert("PROTON_USE_ESYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_use_fsync {
        launch_command
            .env
            .insert("PROTON_USE_FSYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_use_ntsync {
        launch_command
            .env
            .insert("PROTON_USE_NTSYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_no_esync {
        launch_command
            .env
            .insert("PROTON_NO_ESYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_no_fsync {
        launch_command
            .env
            .insert("PROTON_NO_FSYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_no_ntsync {
        launch_command
            .env
            .insert("PROTON_NO_NTSYNC".to_string(), "1".to_string());
    }

    if compat_options.proton_enable_nvapi {
        launch_command
            .env
            .insert("PROTON_ENABLE_NVAPI".to_string(), "1".to_string());
    }

    if compat_options.proton_use_seccomp {
        launch_command
            .env
            .insert("PROTON_USE_SECCOMP".to_string(), "1".to_string());
    }

    if compat_options.proton_use_sdl {
        launch_command
            .env
            .insert("PROTON_USE_SDL".to_string(), "1".to_string());
    }

    if compat_options.proton_enable_wayland {
        launch_command
            .env
            .insert("PROTON_ENABLE_WAYLAND".to_string(), "1".to_string());
    }

    if compat_options.proton_use_wayland {
        launch_command
            .env
            .insert("PROTON_USE_WAYLAND".to_string(), "1".to_string());
    }

    if compat_options.proton_use_xalia {
        launch_command
            .env
            .insert("PROTON_USE_XALIA".to_string(), "1".to_string());
    }

    /*
     * --------------------- Wine options ---------------------
     */
    if !compat_options.wine_prefix.is_empty() {
        launch_command
            .env
            .insert("WINEPREFIX".to_string(), compat_options.wine_prefix.clone());
    }

    if !compat_options.wine_dll_overrides.is_empty() {
        launch_command.env.insert(
            "WINEDLLOVERRIDES".to_string(),
            compat_options.wine_dll_overrides.clone(),
        );
    }

    if compat_options.wine_esync {
        launch_command
            .env
            .insert("WINEESYNC".to_string(), "1".to_string());
    }

    if compat_options.wine_fsync {
        launch_command
            .env
            .insert("WINEFSYNC".to_string(), "1".to_string());
    }

    if compat_options.wine_fullscreen_fsr {
        launch_command
            .env
            .insert("WINE_FULLSCREEN_FSR".to_string(), "1".to_string());
    }

    if compat_options.wine_large_address_aware {
        launch_command
            .env
            .insert("WINE_LARGE_ADDRESS_AWARE".to_string(), "1".to_string());
    }

    /*
     * --------------------- DXVK options ---------------------
     */
    if compat_options.dxvk_hud {
        launch_command
            .env
            .insert("DXVK_HUD".to_string(), "1".to_string());
    }

    /*
     * --------------------- Arbitrary per-category options ---------------------
     *
     * Each category has a "Misc Args" field where the user can put any
     * arbitrary text: KEY=VALUE tokens become environment variables, and
     * anything else is appended to the command itself.
     */
    apply_arbitrary_args(launch_command, &compat_options.proton_args);
    apply_arbitrary_args(launch_command, &compat_options.wine_args);
    apply_arbitrary_args(launch_command, &compat_options.dxvk_args);
}

/// Splits `text` on whitespace and puts every token in its place:
/// tokens containing '=' become environment variables, everything else is
/// appended to the command itself.
fn apply_arbitrary_args(launch_command: &mut LaunchCommand, text: &str) {
    if text.trim().is_empty() {
        return;
    }

    let mut command_args: Vec<String> = Vec::new();

    for token in text.split_whitespace() {
        if token.contains('=') {
            let (key, value) = token.split_once('=').unwrap_or((token, ""));
            launch_command
                .env
                .insert(key.to_string(), value.to_string());
        } else {
            command_args.push(token.to_string());
        }
    }

    if !command_args.is_empty() {
        launch_command.command = format!("{} {}", launch_command.command, command_args.join(" "));
    }
}

/// Parses an env-args string like "PROTON_ENABLE_WAYLAND=1 MANGOHUD_CONFIG=fps"
/// and inserts every assignment into `env`.
fn apply_env_args(env: &mut HashMap<String, String>, env_args: Option<&str>) {
    let Some(env_args) = env_args else {
        return;
    };

    for token in env_args.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            env.insert(key.to_string(), value.to_string());
        }
    }
}
