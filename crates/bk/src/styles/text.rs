use ctappearance::Resolved;
use ctdesign::{BODY, Font};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

/// These views wrap `label()` or `rect()` and keep Freya's methods. They resolve
/// their style when they are built, against the theme and the `.control_style`
/// of the components around the one that builds them: a `.control_style` on a
/// stack does not reach the views built in the same function. `.style` builds
/// the view again, so it comes right after `new`, before Freya's methods.
///
/// States: only `normal`. `Text::font(Font)` changes the font of one text after
/// its style. `Image::size` and `Image::color` are instance overrides of `size`
/// and of `foreground` (`Color`). `VStack`, `HStack`, `Scrim` and `Canvas` have
/// no style: their look is the paint on views and Freya's
/// methods.
#[derive(Clone, Debug)]
pub struct TextStyle {
    /// Tengen: `foreground` `Role(Label)`.
    pub foreground: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `font` `BODY`.
    pub font: Font,
}

impl ControlStyle for TextStyle {
    fn tengen() -> Self {
        Self {
            foreground: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        Self {
            font: self.font.scaled(appearance.text_size),
            ..self
        }
    }
}
