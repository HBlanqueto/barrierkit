use ctdesign::{BODY, CAPTION, DIALOG_PADDING, Font, SPACE_1, SPACE_3, TITLE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct AboutViewStyle {
    /// Tengen, with `spacing`, `name_spacing`: `DIALOG_PADDING`; `SPACE_3` between the icon,

    /// the name block and the copyright; `SPACE_1` between the name and the version.
    pub padding: f32,
    /// Tengen, with `padding`, `name_spacing`: `DIALOG_PADDING`; `SPACE_3` between the icon,
    /// the name block and the copyright; `SPACE_1` between the name and the version.
    pub spacing: f32,
    /// Tengen, with `padding`, `spacing`: `DIALOG_PADDING`; `SPACE_3` between the icon, the
    /// name block and the copyright; `SPACE_1` between the name and the version.
    pub name_spacing: f32,
    /// Tengen: `Role(Label)`.
    pub name: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub version: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`.
    pub copyright: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `TITLE`.
    pub name_font: Font,
    /// Tengen: `BODY`.
    pub version_font: Font,
    /// Tengen: `CAPTION`.
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
