use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

/// The name of the Oxanium family bundled under `crates/monarch-app/fonts/`.
///
/// Oxanium is sourced from Google Fonts (SIL Open Font License). Bundling the
/// font keeps text rendering identical on every device, instead of falling
/// back to whatever system fonts happen to be installed.
///
/// NOTE: We bundle *static* weight instances rather than the upstream
/// variable font (`Oxanium[wght].ttf`). cosmic-text (iced 0.14's text engine)
/// only matches a face when its registered weight equals the requested weight
/// exactly, and fontdb registers a variable font at its *default instance*
/// weight — which for Oxanium is ExtraLight (200). The result was that every
/// request for Normal/Medium/SemiBold/Bold silently fell back to a system
/// font. Regenerate the statics from the variable font with:
///
/// `fonttools varLib.instancer --update-name-table -o Oxanium-Regular.ttf "Oxanium[wght].ttf" wght=400`
///
/// (repeat for 500/Medium, 600/SemiBold, 700/Bold, ...).
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

/// The raw bytes of the bundled Oxanium static instances, one per weight.
///
/// Each is passed to the daemon via [`iced::daemon::Daemon::font`] so they are
/// loaded into the renderer when the compositor is created, before any window
/// content is rendered. The [`Font`] constants above can then resolve on
/// systems without Oxanium installed.
pub const BYTES: &[&[u8]] = &[
    include_bytes!("../../../fonts/Oxanium-Regular.ttf"),
    include_bytes!("../../../fonts/Oxanium-Medium.ttf"),
    include_bytes!("../../../fonts/Oxanium-SemiBold.ttf"),
    include_bytes!("../../../fonts/Oxanium-Bold.ttf"),
];
