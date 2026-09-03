use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tracing::{error, info};

use monarch_egs::{CompatLayer, Session, build_egs_launch_command};

use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::games::GameType;
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::wine_prefix_dir;
use crate::monarch_utils::monarch_terminal;

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
        info!("linux::egs::egs_run() No Epic credentials found, launching '{}' without online auth", game.name);
        return launch_without_auth(game).await;
    }

    let app_name: String = game.properties.other.get("app_name").cloned().unwrap_or("".to_string());
    let install_dir: PathBuf = resolve_install_dir(game)?;
    let prefix: PathBuf = wine_prefix_dir(&format!("umu-{}", game.get_store_id()));

    let mut session: Session = client.user().session();
    let launch = build_egs_launch_command(
        &mut session,
        client.user(),
        &app_name,
        &Path::new(&game.executable_path.as_ref().unwrap_or(&"".to_string())),
        &install_dir,
        CompatLayer::None,
        Some(&prefix),
        game.properties.other.get("egs_manifest_launch_command").unwrap_or(&"".to_string()),
        &extra_args(game),
    )
    .await
    .with_context(|| "linux::egs::egs_run() Failed to build launch command! | Err: ")?;

    spawn(launch.executable, launch.args, &install_dir, launch.environment)
        .await
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

#[cfg(not(target_os = "linux"))]
async fn launch_without_auth(_game: &MonarchGame) -> Result<()> {
    bail!("linux::egs::launch_without_auth() Only supported on Linux")
}
