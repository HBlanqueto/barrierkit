use ctdesign::TOOLBAR_SEARCH_WIDTH;

use super::ControlStyle;

#[derive(Clone, Debug)]
pub struct ToolbarSearchStyle {
    pub width: f32,
}

impl ControlStyle for ToolbarSearchStyle {
    fn tengen() -> Self {
        Self {
            width: TOOLBAR_SEARCH_WIDTH,
        }
    }
}
