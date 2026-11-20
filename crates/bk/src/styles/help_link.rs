use ctappearance::Resolved;
use ctdesign::{BUTTON, DURATION_FAST_MS, HELP_LINK};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    pressed_palette, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct HelpLinkStyle {
    /// Tengen: `size` `BUTTON.height`, a square.
    pub size: f32,
    /// Tengen: `Circle`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, pressed `Color(PressedPalette.bezel)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, the `?`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BUTTON.pressed_scale`.
    pub pressed_scale: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `(HELP_LINK.glyph, HELP_LINK.glyph_stroke)`, the size and line width of the `?`.
    pub glyph: (f32, f32),
    /// Tengen: `Ease { duration_ms: DURATION_FAST_MS }`: the pressed scale eases; `None` makes it
    /// jump.
    pub animation: Animation,
}

impl ControlStyle for HelpLinkStyle {
    fn tengen() -> Self {
        Self {
            size: BUTTON.height,
            shape: Shape::Circle,
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            glyph: (HELP_LINK.glyph, HELP_LINK.glyph_stroke),
            animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}
