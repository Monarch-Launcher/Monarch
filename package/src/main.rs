use cargo_packager::{
    config::{Binary, ConfigBuilder},
    Config, PackageFormat,
};
use std::path::PathBuf;

const MONARCH_BIN_PATH: &'static str = "target/release/monarch";

fn main() {
    // Prevents 'strip' errors on Arch Linux for AppImage packaging
    std::env::set_var("NO_STRIP", "true");

    println!("Packaging Monarch...");

    println!("Configuring...");
    let mut config_builder = create_config();

    if cfg!(target_os = "windows") {
        config_builder = config_windows(config_builder);
    }

    if cfg!(target_os = "macos") {
        config_builder = config_macos(config_builder);
    }

    if cfg!(target_os = "linux") {
        config_builder = config_linux(config_builder);
    }

    println!("Packaging...");
    package(config_builder.config());
}

fn create_config() -> ConfigBuilder {
    // We hardcode the version here or read it from ../Cargo.toml if we want to be dynamic.
    // For now, let's just use "0.1.2" to match the current app version.
    let version = "0.1.2";
    let monarch_bin = Binary::new(MONARCH_BIN_PATH).main(true);

    cargo_packager::config::ConfigBuilder::new()
        .product_name("Monarch")
        .version(version.to_string())
        .identifier("com.monarchlauncher.monarch")
        .binaries(vec![monarch_bin])
        .icons(vec![
            "icons/32x32.png".to_string(),
            "icons/128x128.png".to_string(),
            "icons/128x128@2x.png".to_string(),
            "icons/icon.icns".to_string(),
            "icons/icon.ico".to_string(),
        ])
        .formats(
            PackageFormat::platform_default()
                .iter()
                .cloned()
                .collect::<Vec<PackageFormat>>(),
        )
}

fn config_windows(mut config_builder: ConfigBuilder) -> ConfigBuilder {
    println!("    Windows...");

    let windows_config = cargo_packager::config::WindowsConfig::new();

    config_builder = config_builder.windows(windows_config);

    config_builder
}

fn config_macos(mut config_builder: ConfigBuilder) -> ConfigBuilder {
    println!("    MacOS...");

    let macos_config = cargo_packager::config::MacOsConfig::new();

    config_builder = config_builder.macos(macos_config);

    config_builder
}

fn config_linux(mut config_builder: ConfigBuilder) -> ConfigBuilder {
    println!("    Linux...");

    let pacman_config = cargo_packager::config::PacmanConfig::new();
    let deb_config = cargo_packager::config::DebianConfig::new();
    let appimage_config = cargo_packager::config::AppImageConfig::new();

    config_builder = config_builder.pacman(pacman_config);
    config_builder = config_builder.deb(deb_config);
    config_builder = config_builder.appimage(appimage_config);

    config_builder
}

fn package(config: &Config) {
    match cargo_packager::package(config) {
        Ok(pkgs) => {
            for pkg in pkgs {
                println!(
                    "Packaged: {} at {}",
                    pkg.format.short_name(),
                    pkg.paths
                        .first()
                        .unwrap_or(&PathBuf::from("Unknown"))
                        .display()
                );
            }
        }
        Err(e) => {
            println!("Failed to package! Err: {:?}", e);
            return;
        }
    }
}
