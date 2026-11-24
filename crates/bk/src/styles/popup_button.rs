use ctdesign::{BODY, BUTTON, ControlSize, Font, LIST_GROUP, SYMBOL};

use crate::styles::button::sized_shape;
use crate::styles::{
    ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, pressed_palette,
};

/// States: hovered, pressed, focused. The menu it opens is a menu view
/// (`docs/barrierkit/07-menus-and-popovers.md`).
#[derive(Clone, Debug)]
pub struct PopUpButtonStyle {
    pub height: f32,
    pub gap: f32,
    /// Tengen: `BUTTON.padding_x`, before the icon or the label. After the indicator the padding
    /// is `(height - indicator_size) / 2`, so the indicator sits as far from the end as from
    /// the top.
    pub leading_padding: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Continuous }`; the focus ring
    /// takes its corners.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, as the bezel of a bordered button.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `SYMBOL.grid`, the SVG icon or swatch of the chosen item.
    pub icon_size: f32,
    /// Tengen: `BUTTON.popup.regular.label_lift`, how far the title sits above the center of the button.
    pub label_lift: f32,
    /// Tengen: `BUTTON.popup.regular.trailing`, after the indicator.
    pub trailing_padding: f32,
    /// Tengen: `BUTTON.popup.regular.indicator`.
    pub indicator_size: f32,
    /// Tengen: `BUTTON.popup.regular.glyph`.
    pub chevron_size: f32,
    /// Tengen: `BUTTON.popup.regular.glyph_stroke`, in points.
    pub chevron_stroke: f32,
    /// Tengen: `(BUTTON.popup.regular.pulldown_glyph, BUTTON.popup.regular.pulldown_drop)`: the size of the
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
    /// Tengen: `BODY`.
    pub font: Font,
}

impl PopUpButtonStyle {
    /// The pop-up button of a `Form` row, as SwiftUI draws it: no bezel, the label then the
    /// indicator in a `LIST_GROUP.popup_indicator` circle of the control fill, `LIST_GROUP.popup_height` high.
    pub(crate) fn plain(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Clear),
            indicator: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            height: LIST_GROUP.popup_height,
            indicator_size: LIST_GROUP.popup_indicator,
            gap: LIST_GROUP.popup_gap,
            leading_padding: 0.0,
            trailing_padding: LIST_GROUP.popup_trailing,
            ..self
        }
    }

    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let button = BUTTON.size(size);
        let popup = BUTTON.popup.size(size);
        Self {
            height: button.height,
            gap: popup.gap,
            leading_padding: button.padding_x,
            shape: sized_shape(button.radius, button.height),
            label_lift: popup.label_lift,
            trailing_padding: popup.trailing,
            indicator_size: popup.indicator,
            chevron_size: popup.glyph,
            chevron_stroke: popup.glyph_stroke,
            pulldown_chevron: (popup.pulldown_glyph, popup.pulldown_drop),
            font: Font {
                size: button.font_size as u32,
                ..self.font
            },
            ..self
        }
    }
}

impl ControlStyle for PopUpButtonStyle {
    fn tengen() -> Self {
        let popup = BUTTON.popup.regular;
        Self {
            height: BUTTON.height,
            gap: popup.gap,
            leading_padding: BUTTON.padding_x,
            shape: sized_shape(BUTTON.radius, BUTTON.height),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            icon_size: SYMBOL.grid,
            label_lift: popup.label_lift,
            trailing_padding: popup.trailing,
            indicator_size: popup.indicator,
            chevron_size: popup.glyph,
            chevron_stroke: popup.glyph_stroke,
            pulldown_chevron: (popup.pulldown_glyph, popup.pulldown_drop),
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
            font: BODY,
        }
    }
}
