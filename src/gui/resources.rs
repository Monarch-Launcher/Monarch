use iced::widget::{image, svg};
use std::sync::LazyLock;

pub static LOGO: LazyLock<image::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/Square71x71Logo.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static ICON: LazyLock<image::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/icon.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static MONARCH: LazyLock<svg::Handle> = LazyLock::new(|| {
    let bytes = include_bytes!("../../icons/monarch.svg");
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
