use ctdesign::{BODY, BUTTON, Font, SETTINGS, SPACE_1};

use crate::styles::button::continuous_rounded;
use crate::styles::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, pressed_palette,
    system_effect,
};

/// States: pressed and focused; each half of a `ComboButton` on its own,
/// and its group only `normal`. `ShareLink` is a `MenuButton`: its `.style` takes
/// a `MenuButtonStyle`.
#[derive(Clone, Debug)]
pub struct MenuButtonStyle {
    /// Tengen: `BUTTON.height`.
    pub height: f32,
    /// Tengen: `BUTTON.padding_x`.
    pub padding_x: f32,
    /// Tengen: `SPACE_1`.
    pub chevron_gap: f32,
    /// Tengen: `SETTINGS.dropdown_chevron_glyph`.
    pub chevron_size: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Continuous }`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, pressed `Color(PressedPalette.bezel)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, pressed `Color(PressedPalette.bezel)`: the `IconOnly` button is a square of
    /// `height`, lit under the pointer like `PopUpButton`.
    pub icon_only_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for MenuButtonStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            padding_x: BUTTON.padding_x,
            chevron_gap: SPACE_1,
            chevron_size: SETTINGS.dropdown_chevron_glyph as f32,
            shape: continuous_rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel)),
            icon_only_background: StateValue::new(ShapeStyle::Clear)
                .pressed(ShapeStyle::Color(pressed_palette().bezel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}
