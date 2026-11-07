use ctdesign::{BODY, DIALOG_ALERT_WIDTH, Font, HEADLINE, SPACE_2, SPACE_4};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct AlertStyle {
    pub width: f32,
    pub spacing: f32,
    pub buttons_spacing: f32,
    pub buttons_margin_top: f32,
    pub title: StateValue<ShapeStyle>,
    pub message: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub title_font: Font,
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
