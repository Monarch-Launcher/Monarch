use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

/// The name of the Oxanium family bundled under `crates/monarch-app/fonts/`.
///
/// Oxanium is a variable font (weights 200-800) sourced from Google Fonts
/// (SIL Open Font License); the renderer picks the requested weight from its
/// `wght` axis, so bundling a single file covers every weight used below.
/// Bundling the font keeps text rendering identical on every device,
/// instead of falling back to whatever system fonts happen to be installed.
pub const FAMILY: &str = "Oxanium";

pub const REGULAR: Font = Font {
    family: Family::Name(FAMILY),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const BOLD: Font = Font {
    family: Family::Name(FAMILY),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const SEMIBOLD: Font = Font {
    family: Family::Name(FAMILY),
    weight: Weight::Semibold,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const MEDIUM: Font = Font {
    family: Family::Name(FAMILY),
    weight: Weight::Medium,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const _MONOSPACE: Font = Font {
    family: Family::Monospace,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// The raw bytes of the bundled Oxanium variable font.
///
/// Passed to the daemon via [`iced::daemon::Daemon::font`] so it is loaded
/// into the renderer when the compositor is created, before any window
/// content is rendered. [`Font`] constants above can then resolve on systems
/// without Oxanium installed.
pub const BYTES: &[u8] = include_bytes!("../../../fonts/Oxanium[wght].ttf");
