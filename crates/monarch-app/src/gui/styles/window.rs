//! Custom window frame styling: rounded corners and a thin border.

use iced::widget::container;
use iced::{Border, Color, Theme};

/// Corner radius of the window frame.
///
/// Matches the OS corner radius used by [`crate::window::apply_rounded_corners`]
/// so the border arc lines up with the window silhouette.
pub const BORDER_RADIUS: f32 = 8.0;

/// Width of the thin window border.
pub const BORDER_WIDTH: f32 = 1.0;

/// Width of the invisible resize handle band around the window.
pub const RESIZE_HANDLE: f32 = 6.0;

/// Total padding reserved around the content for the border and resize band.
pub const FRAME_PADDING: f32 = BORDER_WIDTH + RESIZE_HANDLE;

/// Style of the main window frame: dark background, rounded corners, thin border.
pub fn frame(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(theme.palette().background.into()),
        border: Border {
            width: BORDER_WIDTH,
            radius: BORDER_RADIUS.into(),
            color: Color::from_rgb8(78, 78, 95),
        },
        ..Default::default()
    }
}