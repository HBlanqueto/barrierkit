use ctdesign::{BODY, CAPTION, DIALOG_PADDING, Font, SPACE_1, SPACE_3, TITLE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct AboutViewStyle {
    pub padding: f32,
    pub spacing: f32,
    pub name_spacing: f32,
    pub name: StateValue<ShapeStyle>,
    pub version: StateValue<ShapeStyle>,
    pub copyright: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub name_font: Font,
    pub version_font: Font,
    pub copyright_font: Font,
}

impl ControlStyle for AboutViewStyle {
    fn tengen() -> Self {
        Self {
            padding: DIALOG_PADDING,
            spacing: SPACE_3,
            name_spacing: SPACE_1,
            name: StateValue::new(ShapeStyle::Role(Role::Label)),
            version: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            copyright: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            effect: system_effect,
            name_font: TITLE,
            version_font: BODY,
            copyright_font: CAPTION,
        }
    }
}
