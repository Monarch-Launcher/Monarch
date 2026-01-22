use iced::widget::{image, svg};
use once_cell::sync::Lazy;

pub static LOGO: Lazy<image::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Square71x71Logo.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static ICON: Lazy<image::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/icon.png");
    image::Handle::from_bytes(bytes.to_vec())
});

pub static MONARCH: Lazy<svg::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/monarch.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static STEAM: Lazy<svg::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Stores/steam_icon_135152.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static EPIC: Lazy<svg::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Stores/epic_games_logo_icon_145306.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static GOG: Lazy<svg::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Stores/gog_icon_135545.svg");
    svg::Handle::from_memory(bytes.to_vec())
});

pub static ITCH: Lazy<svg::Handle> = Lazy::new(|| {
    let bytes = include_bytes!("../../icons/Stores/itch_io_icon_198115.svg");
    svg::Handle::from_memory(bytes.to_vec())
});
