use ctdesign::TOOLBAR_SEARCH_WIDTH;

use super::ControlStyle;

/// The open field is a `SearchField` and draws from the `SearchFieldStyle` in
/// context; the collapsed button is an `IconButton` and draws from the
/// `IconButtonStyle` in context. Change them with `.control_style` around the
/// toolbar search:
#[derive(Clone, Debug)]
pub struct ToolbarSearchStyle {
    /// Tengen: `TOOLBAR_SEARCH_WIDTH`, the width of the open field.
    pub width: f32,
}

impl ControlStyle for ToolbarSearchStyle {
    fn tengen() -> Self {
        Self {
            width: TOOLBAR_SEARCH_WIDTH,
        }
    }
}
