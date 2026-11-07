use ctdesign::{BODY, DIALOG_ALERT_WIDTH, Font, HEADLINE, SPACE_2, SPACE_4};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct AlertStyle {
    /// Tengen: `DIALOG_ALERT_WIDTH` with the sheet's `padding`.
    pub width: f32,
    /// Tengen: `SPACE_2`.
    pub spacing: f32,
    /// Tengen: `SPACE_2`.
    pub buttons_spacing: f32,
    /// Tengen: `SPACE_4`.
    pub buttons_margin_top: f32,
    /// Tengen: `title` `Role(Label)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `message` `Role(SecondaryLabel)`.
    pub message: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen, with `message_font`: `title_font` `HEADLINE`, `message_font` `BODY`.
    pub title_font: Font,
    /// Tengen, with `title_font`: `title_font` `HEADLINE`, `message_font` `BODY`.
    pub message_font: Font,
}

impl ControlStyle for AlertStyle {
    fn tengen() -> Self {
        Self {
            width: DIALOG_ALERT_WIDTH,
            spacing: SPACE_2,
            buttons_spacing: SPACE_2,
            buttons_margin_top: SPACE_4,
            title: StateValue::new(ShapeStyle::Role(Role::Label)),
            message: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            effect: system_effect,
            title_font: HEADLINE,
            message_font: BODY,
        }
    }
}
