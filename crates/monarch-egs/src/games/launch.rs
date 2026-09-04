use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::GameToken;
use crate::auth::{Session, User};
use crate::utils::err::MonarchEgsError;

/// Which compatibility layer to use when running a Windows game on Linux.
pub enum CompatLayer {
    Proton(PathBuf),
    Wine(PathBuf),
    None,
}

/// Assembled launch information for an EGS game.
pub struct EgsLaunchCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    /// Environment variables required by the compatibility layer.
    pub environment: HashMap<String, String>,
}

/// Build the complete launch command for an Epic Games Store game on Linux.
pub async fn build_egs_launch_command(
    user: &User,
    app_name: &str,
    namespace: &str,
    exe_path: &Path,
    install_dir: &Path,
    compat: CompatLayer,
    wine_prefix: Option<&Path>,
    egs_launch_commands: &str,
    extra_args: &[String],
    ot_path: Option<String>,
) -> Result<EgsLaunchCommand, MonarchEgsError> {
    let mut environment: HashMap<String, String> = HashMap::new();
    let mut compat_args: Vec<String> = Vec::new();
    let exe_command: String;

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
            environment.insert("WINEDLLOVERRIDES".into(), "winemenubuilder.exe=d".into());
            environment.entry("LD_PRELOAD".into()).or_default();

            // umu-run expects the game exe as a relative path from install_dir
            let rel_exe = exe_path.strip_prefix(install_dir).unwrap_or(&exe_path);
            exe_command = "umu-run".into();
            compat_args.push(rel_exe.to_string_lossy().to_string());
        }
        CompatLayer::Wine(wine_bin) => {
            let prefix = resolve_prefix(wine_prefix, app_name);
            exe_command = wine_bin.to_string_lossy().to_string();
            environment.insert("WINEPREFIX".into(), prefix);
            compat_args.push(exe_path.to_string_lossy().to_string());
        }
        CompatLayer::None => {
            exe_command = exe_path.to_string_lossy().to_string();
        }
    }

    // build Epic Games Store auth arguments
    let auth_args: Vec<String> = build_egs_auth_args(user, app_name, namespace, ot_path).await?;

    // manifest launch_command
    let mut manifest_args: Vec<String> = Vec::new();
    if !egs_launch_commands.is_empty() {
        manifest_args.extend(egs_launch_commands.split_whitespace().map(String::from));
    }

    // assemble final arg list
    let mut args: Vec<String> = Vec::new();
    args.extend(compat_args);
    args.extend(auth_args);
    args.extend(manifest_args);
    args.extend(extra_args.iter().cloned());

    Ok(EgsLaunchCommand {
        executable: exe_command,
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
async fn build_egs_auth_args(
    user: &User,
    app_name: &str,
    namespace: &str,
    ot_path: Option<String>,
) -> Result<Vec<String>, MonarchEgsError> {
    let mut session: Session = user.session();

    let token: GameToken = session.get_game_token().await?;
    let account_id: String = session.get_account_id();
    let display_name: String = user.display_name();

    let mut auth_args: Vec<String> = vec![
        "-AUTH_LOGIN=unused".into(),
        format!("-AUTH_PASSWORD={}", token.code),
        "-AUTH_TYPE=exchangecode".into(),
        format!("-epicapp={app_name}"),
        "-epicenv=Prod".into(),
        format!("-epicusername={display_name}"),
        format!("-epicuserid={account_id}"),
        "-epiclocale=en-US".into(),
        format!("-epicsandboxid={namespace}"),
        "-EpicPortal".into(),
    ];

    if let Some(path) = ot_path {
        auth_args.push(format!("-epicovt={path}"));
    }

    Ok(auth_args)
}
