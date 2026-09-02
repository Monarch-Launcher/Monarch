use iced::widget::{image, svg};
use std::sync::LazyLock;

pub static LOGO: LazyLock<image::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Logo/Square71x71Logo.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static LOGO_LARGE: LazyLock<image::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Logo/Square310x310Logo.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static MONARCH: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Logo/monarch.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static STEAM: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Stores/steam_icon_135152.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static EPIC: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Stores/epic_games_logo_icon_145306.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static GOG: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Stores/gog_icon_135545.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static ITCH: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Stores/itch_io_icon_198115.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static PLAY: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/play_icon.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static REFRESH: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/refresh.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static ADD_FOLDER: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/folder-plus-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static _SEARCH_FOLDER: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/folder-search-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static FOLDER: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/folder-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static VIEW: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/view.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static HIDE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/hide.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static DOWNLOAD: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/download-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static PAUSE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/pause-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static TRASH: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!(
        "../../icons/Actions/bin-cancel-delete-remove-trash-garbage-svgrepo-com.svg"
    );
    svg::Handle::from_memory(bytes.to_vec())
});

pub static FAVORITE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/star-alt-3-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static FAVORITE_OUTLINE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/star-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static UPDATE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/update-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static FILTER: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/filter-svgrepo-com.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static WINDOW_MINIMIZE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/window-minimize.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static WINDOW_MAXIMIZE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/window-maximize.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static WINDOW_RESTORE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/window-restore.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static WINDOW_CLOSE: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Actions/window-close.svg");
    svg::Handle::from_memory(bytes.to_vec())
});
