use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tracing::{error, info};

use monarch_egs::{CompatLayer, Manifest, get_game_manifest, build_egs_launch_command};

use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::games::GameType;
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{
    self, ensure_wine_safe_install_dir, get_monarch_home, wine_prefix_dir,
};
use crate::monarch_utils::monarch_terminal;

/// Platform identifier used when fetching the manifest for a Monarch-managed
/// install. Games are downloaded as Windows build manifests regardless of the
/// host OS (they run through Proton/Wine on Linux).
const MANAGED_PLATFORM: &str = "Windows";

/// Launches an Epic Games Store game installed and managed by Monarch.
///
/// Loads the Epic session, fetches the game manifest so the exact launch
/// executable and Epic-provided launch arguments are known, builds the full
/// launch command (compatibility layer + auth args + user args) via
/// [`build_egs_launch_command`], and spawns it in a terminal.
pub async fn egs_run(game: &MonarchGame) -> Result<()> {
    let mut client = EgsClient::new();
    if client.credentials_exist() {
        client
            .load_existing_user()
            .await
            .with_context(|| "linux::egs::egs_run() -> ")?;
    } else {
        // Without a session we cannot fetch a manifest or provide Epic auth
        // args, so fall back to launching the recorded executable directly
        // through umu (no online authentication).
        info!("linux::egs::egs_run() No Epic credentials found, launching '{}' without online auth", game.name);
        return launch_without_auth(game).await;
    }

    let (app_name, catalog_id, namespace) = epic_metadata(game)?;

    let install_dir = resolve_install_dir(game)?;

    let token = client.user().session().get_access_token().await;
    let manifest: Manifest = get_game_manifest(
        &token,
        MANAGED_PLATFORM,
        &namespace,
        &catalog_id,
        &app_name,
    )
    .await
    .with_context(|| "linux::egs::egs_run() Failed to fetch game manifest! | Err: ")?;

    let compat = resolve_compat(game)?;
    let prefix = wine_prefix_dir(&format!("umu-{}", game.get_store_id()));

    let mut session = client.user().session();
    let launch = build_egs_launch_command(
        &mut session,
        client.user(),
        &app_name,
        &install_dir,
        Some(&manifest),
        compat,
        Some(&prefix),
        &extra_args(game),
    )
    .await
    .with_context(|| "linux::egs::egs_run() Failed to build launch command! | Err: ")?;

    spawn(launch.executable, launch.args, &install_dir, launch.environment)
        .await
}

/// Builds the compatibility layer from the game's recorded Proton path.
fn resolve_compat(game: &MonarchGame) -> Result<CompatLayer> {
    let compatibility = game
        .compatibility
        .clone()
        .ok_or_else(|| anyhow::anyhow!("linux::egs::resolve_compat() No compatibility layer set for {}", game.name))?;
    Ok(CompatLayer::Proton(PathBuf::from(compatibility)))
}

/// Returns the caller-supplied launch arguments for the game.
fn extra_args(game: &MonarchGame) -> Vec<String> {
    game.launch_args
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(String::from)
        .collect()
}

/// Collects the Epic metadata (app name, catalog id, namespace) required to
/// fetch the game manifest.
fn epic_metadata(game: &MonarchGame) -> Result<(String, String, String)> {
    let catalog_id = game
        .properties
        .other
        .get("catalog_id")
        .cloned()
        .unwrap_or_default();
    let app_name = game
        .properties
        .other
        .get("app_name")
        .cloned()
        .unwrap_or_default();
    let namespace = game.get_store_id();

    if catalog_id.is_empty() || app_name.is_empty() || namespace.is_empty() {
        bail!("linux::egs::epic_metadata() Missing Epic Games metadata (app_name/catalog_id/namespace) for '{}'", game.name)
    }

    Ok((app_name, catalog_id, namespace))
}

/// Resolves the on-disk install directory, preferring the recorded path.
fn resolve_install_dir(game: &MonarchGame) -> Result<PathBuf> {
    let recorded = game.properties.install_dir.trim();
    if !recorded.is_empty() && recorded != "Error" {
        let path = PathBuf::from(recorded);
        if path.is_dir() {
            return Ok(path);
        }
    }
    if let Some(exe) = &game.executable_path {
        if let Some(parent) = Path::new(exe).parent() {
            if parent.is_dir() {
                return Ok(parent.to_path_buf());
            }
        }
    }
    bail!("linux::egs::resolve_install_dir() Unable to locate install directory for '{}'", game.name)
}

/// Spawns the built launch command in a terminal window.
async fn spawn(
    executable: String,
    args: Vec<String>,
    workdir: &Path,
    environment: std::collections::HashMap<String, String>,
) -> Result<()> {
    let mut command = String::from(&executable);
    for arg in &args {
        command.push(' ');
        command.push_str(&quote_arg(arg));
    }

    info!("linux::egs::spawn() Launch command: {}", command);
    info!("linux::egs::spawn() Working directory: {}", workdir.display());

    let rx = monarch_terminal::spawn_terminal(
        command,
        environment,
        Some(workdir.to_string_lossy().to_string()),
    );
    if let Err(e) = rx.await {
        error!("linux::egs::spawn() Terminal command failed! | Err: {e}");
    }
    Ok(())
}

/// Quote an argument for a shell / terminal command string.
fn quote_arg(arg: &str) -> String {
    if arg.chars().any(|c| c.is_whitespace() || c == '\'') {
        format!("'{}'", arg.replace('\'', "'\\''"))
    } else {
        arg.to_string()
    }
}

/// Launch path used when no Epic session is available: runs the recorded
/// executable through umu-launcher without online authentication.
#[cfg(target_os = "linux")]
async fn launch_without_auth(game: &MonarchGame) -> Result<()> {
    let compatibility = game
        .compatibility
        .clone()
        .ok_or_else(|| anyhow::anyhow!("linux::egs::launch_without_auth() No compatibility layer set for {}", game.name))?;

    let exe = PathBuf::from(
        game.executable_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("linux::egs::launch_without_auth() No executable path set for {}", game.name))?,
    );
    if !exe.is_file() {
        bail!("linux::egs::launch_without_auth() Executable not found: {}", exe.display());
    }

    let install_dir = resolve_install_dir(game)?;
    let (install_dir, safe_exe) = ensure_safe_paths(game, install_dir, exe).await?;

    let gameid_arg = format!("umu-{}", game.get_store_id());
    let prefix = wine_prefix_dir(&gameid_arg);
    std::fs::create_dir_all(&prefix).with_context(|| {
        format!(
            "linux::egs::launch_without_auth() Failed to create wine prefix {} | Err: ",
            prefix.display()
        )
    })?;

    let mut env_vars: std::collections::HashMap<String, String> = std::collections::HashMap::from([
        ("PROTONPATH".to_string(), compatibility),
        ("GAMEID".to_string(), gameid_arg.clone()),
        ("STORE".to_string(), "egs".to_string()),
        ("WINEPREFIX".to_string(), prefix.to_string_lossy().to_string()),
        (
            "STEAM_COMPAT_DATA_PATH".to_string(),
            prefix.to_string_lossy().to_string(),
        ),
        (
            "STEAM_COMPAT_INSTALL_PATH".to_string(),
            install_dir.to_string_lossy().to_string(),
        ),
        (
            "WINEDLLOVERRIDES".to_string(),
            "winemenubuilder.exe=d".to_string(),
        ),
    ]);
    env_vars
        .entry("LD_PRELOAD".to_string())
        .or_default();

    let exe_arg = monarch_fs::relative_launch_exe_arg(&safe_exe, &install_dir);
    let umu: PathBuf = monarch_fs::find_linux_binary("umu-run").unwrap_or(
        get_monarch_home().join("umu").join("umu-run"),
    );
    let launch_command: String = format!("{} {}", umu.display(), exe_arg);

    let launch_args = game.launch_args.as_deref().unwrap_or_default().trim();
    let full_command: String = if launch_args.contains("%command%") {
        launch_args.replace("%command%", &launch_command)
    } else if launch_args.is_empty() {
        launch_command
    } else {
        format!("{launch_command} {launch_args}")
    };

    info!("linux::egs::launch_without_auth() Launch command: {}", full_command);

    let rx = monarch_terminal::spawn_terminal(
        full_command,
        env_vars,
        Some(install_dir.to_string_lossy().to_string()),
    );
    if let Err(e) = rx.await {
        error!("linux::egs::launch_without_auth() Terminal command failed! | Err: {e}");
    }
    Ok(())
}

/// Renames the install folder when needed for Wine and returns safe paths.
async fn ensure_safe_paths(
    game: &MonarchGame,
    install_dir: PathBuf,
    exe: PathBuf,
) -> Result<(PathBuf, PathBuf)> {
    let Some(safe_dir) = ensure_wine_safe_install_dir(&install_dir)? else {
        return Ok((install_dir, exe));
    };

    let rel_exe = exe
        .strip_prefix(&install_dir)
        .unwrap_or(Path::new(exe.file_name().unwrap_or_default()));
    let safe_exe = safe_dir.join(rel_exe);

    let mut game = game.clone();
    game.properties.install_dir = safe_dir.to_string_lossy().to_string();
    game.executable_path = Some(safe_exe.to_string_lossy().to_string());

    if let Err(e) = crate::monarch_library::library::update_game_properties(&game).await {
        error!("linux::egs::ensure_safe_paths() Failed to persist renamed install paths | Err: {e}");
    }

    Ok((safe_dir, safe_exe))
}

#[cfg(not(target_os = "linux"))]
async fn launch_without_auth(_game: &MonarchGame) -> Result<()> {
    bail!("linux::egs::launch_without_auth() Only supported on Linux")
}
