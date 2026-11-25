use ctdesign::{BODY, SYMBOL};

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct ImageStyle {
    /// Tengen: `BODY.size` in SF points, as SwiftUI draws a symbol at the body font; a symbol lays out in its own SF frame.
    pub size: f32,
    /// Tengen: `SYMBOL.stroke`, the line width of a symbol on its grid.
    pub stroke: f32,
    /// Tengen: `foreground` `Role(Label)`, the symbol.
    pub foreground: StateValue<ShapeStyle>,
}

impl ControlStyle for ImageStyle {
    fn tengen() -> Self {
        Self {
            size: BODY.size as f32 * crate::symbol::SF_POINT,
            stroke: SYMBOL.stroke,
            foreground: StateValue::new(ShapeStyle::Role(Role::Label)),
        }
    }
}
