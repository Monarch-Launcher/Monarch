use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::Manifest;
use crate::auth::{Session, User};

use crate::utils::err::MonarchEgsError;

/// Which compatibility layer to use when running a Windows game on Linux.
pub enum CompatLayer {
    /// Proton — path to the Proton distribution directory
    /// (e.g. `~/.steam/steam/steamapps/common/Proton 8.0`).
    Proton(PathBuf),
    /// Wine — path to the `wine` / `wine64` binary.
    Wine(PathBuf),
    /// No compatibility layer (native Windows or caller handles it).
    None,
}

/// Assembled launch information for an EGS game.
pub struct EgsLaunchCommand {
    /// The executable to invoke (umu-run, wine64, or the game .exe directly).
    pub executable: String,
    /// Full argument list: compat layer args + exe + auth args + launch_command + extra.
    pub args: Vec<String>,
    /// Working directory — the game install directory.
    pub working_directory: PathBuf,
    /// Environment variables required by the compatibility layer.
    pub environment: HashMap<String, String>,
}

/// Build the complete launch command for an Epic Games Store game on Linux.
///
/// Combines the compatibility layer (Proton/Wine), the game executable,
/// Epic authentication parameters, manifest launch arguments, and any
/// caller-supplied extra arguments into a single [`EgsLaunchCommand`].
pub async fn build_egs_launch_command(
    session: &mut Session,
    user: &User,
    app_name: &str,
    install_dir: &Path,
    manifest: Option<&Manifest>,
    compat: CompatLayer,
    wine_prefix: Option<&Path>,
    extra_args: &[String],
) -> Result<EgsLaunchCommand, MonarchEgsError> {
    // --- resolve the game executable path -----------------------------------
    let game_exe = match manifest {
        Some(m) => {
            let rel = m.launch_exe();
            if rel.is_empty() {
                return Err(MonarchEgsError::ParsingError(
                    "Manifest has no launch_exe".into(),
                ));
            }
            install_dir.join(rel)
        }
        None => {
            return Err(MonarchEgsError::ParsingError(
                "A manifest is required to resolve the game executable".into(),
            ));
        }
    };

    let mut environment: HashMap<String, String> = HashMap::new();
    let exe_path: String;
    let mut compat_args: Vec<String> = Vec::new();

    match &compat {
        CompatLayer::Proton(proton_path) => {
            let proton_dir = proton_path.to_string_lossy().to_string();
            let prefix = resolve_prefix(wine_prefix, app_name);

            environment.insert("PROTONPATH".into(), proton_dir);
            environment.insert("GAMEID".into(), format!("umu-{app_name}"));
            environment.insert("STORE".into(), "egs".into());
            environment.insert("WINEPREFIX".into(), prefix.clone());
            environment.insert("STEAM_COMPAT_DATA_PATH".into(), prefix);
            environment.insert(
                "STEAM_COMPAT_INSTALL_PATH".into(),
                install_dir.to_string_lossy().to_string(),
            );
            environment.insert(
                "WINEDLLOVERRIDES".into(),
                "winemenubuilder.exe=d".into(),
            );
            environment
                .entry("LD_PRELOAD".into())
                .or_default();

            // umu-run expects the game exe as a relative path from install_dir
            let rel_exe = game_exe
                .strip_prefix(install_dir)
                .unwrap_or(&game_exe);
            exe_path = "umu-run".into();
            compat_args.push(rel_exe.to_string_lossy().to_string());
        }
        CompatLayer::Wine(wine_bin) => {
            let prefix = resolve_prefix(wine_prefix, app_name);
            exe_path = wine_bin.to_string_lossy().to_string();
            environment.insert("WINEPREFIX".into(), prefix);
            compat_args.push(game_exe.to_string_lossy().to_string());
        }
        CompatLayer::None => {
            exe_path = game_exe.to_string_lossy().to_string();
        }
    }

    // --- build Epic Games Store auth arguments ------------------------------
    let auth_args = build_egs_auth_args(session, user, app_name).await;

    // --- manifest launch_command (extra exe args from Epic metadata) ---------
    let mut manifest_args: Vec<String> = Vec::new();
    if let Some(m) = manifest {
        let cmd = m.launch_command();
        if !cmd.is_empty() {
            manifest_args.extend(cmd.split_whitespace().map(String::from));
        }
    }

    // --- assemble final arg list --------------------------------------------
    let mut args: Vec<String> = Vec::new();
    args.extend(compat_args);
    args.extend(auth_args);
    args.extend(manifest_args);
    args.extend(extra_args.iter().cloned());

    Ok(EgsLaunchCommand {
        executable: exe_path,
        args,
        working_directory: install_dir.to_path_buf(),
        environment,
    })
}

/// Resolve the Wine/Proton prefix path.
///
/// Falls back to `~/.local/share/Monarch/wine_prefixes/umu-{app_name}` when
/// no explicit prefix is provided.
fn resolve_prefix(wine_prefix: Option<&Path>, app_name: &str) -> String {
    if let Some(p) = wine_prefix {
        return p.to_string_lossy().to_string();
    }
    let data_home = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".local").join("share")
        });
    data_home
        .join("Monarch")
        .join("wine_prefixes")
        .join(format!("umu-{app_name}"))
        .to_string_lossy()
        .to_string()
}

/// Build the Epic authentication command-line arguments.
///
/// These are the same flags the official Epic Games Launcher and Legendary
/// pass to game executables for online authentication.
async fn build_egs_auth_args(
    session: &mut Session,
    user: &User,
    app_name: &str,
) -> Vec<String> {
    let token = session.get_access_token().await;
    let account_id = session.get_account_id();
    let display_name = user.display_name();

    vec![
        "-AUTH_LOGIN=unused".into(),
        format!("-AUTH_PASSWORD={token}"),
        "-AUTH_TYPE=exchangecode".into(),
        format!("-epicapp={app_name}"),
        "-epicenv=Prod".into(),
        format!("-epicusername={display_name}"),
        format!("-epicuserid={account_id}"),
        "-epiclocale=en-US".into(),
        "-EpicPortal".into(),
    ]
}
