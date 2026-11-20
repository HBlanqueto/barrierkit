use ctdesign::{BODY, BUTTON, Font, SETTINGS, SYMBOL};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, pressed_palette,
    system_effect,
};

/// States: hovered, pressed, focused. The menu it opens is a menu view
/// (`docs/barrierkit/07-menus-and-popovers.md`).
#[derive(Clone, Debug)]
pub struct PopUpButtonStyle {
    /// Tengen, with `gap`: `SETTINGS.popup_height`, `SETTINGS.popup_text_gap` between the
    /// icon, the label and the indicator.
    pub height: f32,
    /// Tengen, with `height`: `SETTINGS.popup_height`, `SETTINGS.popup_text_gap` between the
    /// icon, the label and the indicator.
    pub gap: f32,
    /// Tengen: `BUTTON.padding_x`, before the icon or the label. After the indicator the padding
    /// is `(height - indicator_size) / 2`, so the indicator sits as far from the end as from
    /// the top.
    pub leading_padding: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Circular }`; the focus ring
    /// takes its corners.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, as the bezel of a bordered button.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `SYMBOL.grid`, the SVG icon or swatch of the chosen item.
    pub icon_size: f32,
    /// Tengen: `SETTINGS.popup_label_lift`, how far the title sits above the center of the button.
    pub label_lift: f32,
    /// Tengen: `SETTINGS.popup_trailing_padding`, after the indicator.
    pub trailing_padding: f32,
    /// Tengen: `SETTINGS.popup_chevron`.
    pub indicator_size: f32,
    /// Tengen: `SETTINGS.popup_glyph`.
    pub chevron_size: f32,
    /// Tengen: `SETTINGS.popup_glyph_stroke`, in points.
    pub chevron_stroke: f32,
    /// Tengen: `(SETTINGS.popup_pulldown_glyph, SETTINGS.popup_pulldown_drop)`: the size of the
    /// chevron of a button that pulls down and how far it sits below the center of its box.
    pub pulldown_chevron: (f32, f32),
    /// Tengen: `Circle`.
    pub indicator_shape: Shape,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Clear`.
    pub indicator: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub indicator_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub indicator_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `1.0`: a disabled pop-up button changes its colors, not its opacity.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for PopUpButtonStyle {
    fn tengen() -> Self {
        Self {
            height: SETTINGS.popup_height as f32,
            gap: SETTINGS.popup_text_gap as f32,
            leading_padding: BUTTON.padding_x,
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            icon_size: SYMBOL.grid,
            label_lift: SETTINGS.popup_label_lift,
            trailing_padding: SETTINGS.popup_trailing_padding as f32,
            indicator_size: SETTINGS.popup_chevron as f32,
            chevron_size: SETTINGS.popup_glyph as f32,
            chevron_stroke: SETTINGS.popup_glyph_stroke,
            pulldown_chevron: (
                SETTINGS.popup_pulldown_glyph as f32,
                SETTINGS.popup_pulldown_drop,
            ),
            indicator_shape: Shape::Circle,
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            indicator: StateValue::new(ShapeStyle::Clear),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            indicator_border: None,
            indicator_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}
