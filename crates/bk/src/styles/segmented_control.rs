use ctdesign::{BODY, ControlSize, Font, GLASS_CONTROL, SEGMENTED};

use super::button::continuous_rounded;
use super::button::sized_shape;
use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    disabled_palette, glass_hover, glass_ink, per_scheme, system_effect,
};

/// States: each segment is hovered, pressed and focused on its own; the group
/// has only `normal`. The arrow keys move the choice on both axes.
#[derive(Clone, Debug)]
pub struct SegmentedControlStyle {
    /// Tengen: `SEGMENTED.padding`.
    pub padding: f32,
    /// Tengen: `SEGMENTED.gap`.
    pub gap: f32,
    /// Tengen: `SEGMENTED.segment_height`.
    pub segment_height: f32,
    /// Tengen: `SEGMENTED.segment_padding_x`.
    pub segment_padding_x: f32,
    /// Tengen: `false`, each segment fits its title; a `Form` row sizes them all to the widest.
    pub equal_segments: bool,
    /// Tengen: `SEGMENTED.divider_height`, a 1 wide line centered between two segments that are not chosen.
    pub divider_height: f32,
    /// Tengen: `Role(SegmentDivider)`.
    pub divider: StateValue<ShapeStyle>,
    /// Tengen, with `segment_shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Continuous`.
    pub shape: Shape,
    /// Tengen, with `shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Continuous`.
    pub segment_shape: Shape,
    /// Tengen: `Role(ControlBackground)`, behind the segments.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `selected`: `Clear` for a segment that is not chosen,
    /// `Role(ToggleAccent)` for the chosen one.
    pub segment: StateValue<ShapeStyle>,
    /// Tengen, with `segment`: `Clear` for a segment that is not chosen,
    /// `Role(ToggleAccent)` for the chosen one.
    pub selected: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the group.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: none.
    pub selected_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`: the segments share the width; `Vertical` stacks them at full
    /// width.
    pub axis: Axis,
    /// Tengen: `BODY` with `SEGMENTED.font_size` and `SEGMENTED.font_weight`; `line_height` has
    /// no effect.
    pub font: Font,
}

impl SegmentedControlStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = SEGMENTED.size(size);
        let shape = sized_shape(row.radius, row.segment_height);
        Self {
            segment_height: row.segment_height,
            segment_padding_x: row.segment_padding_x,
            divider_height: row.divider_height,
            shape,
            segment_shape: shape,
            font: Font {
                size: row.font_size as u32,
                ..self.font
            },
            ..self
        }
    }
}

impl ControlStyle for SegmentedControlStyle {
    fn tengen() -> Self {
        let style = Self {
            padding: SEGMENTED.padding,
            gap: SEGMENTED.gap,
            segment_height: SEGMENTED.segment_height,
            segment_padding_x: SEGMENTED.segment_padding_x,
            equal_segments: false,
            divider_height: SEGMENTED.divider_height,
            divider: StateValue::new(ShapeStyle::Role(Role::SegmentDivider)),
            shape: continuous_rounded(SEGMENTED.radius),
            segment_shape: continuous_rounded(SEGMENTED.segment_radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            segment: StateValue::new(ShapeStyle::Clear),
            selected: StateValue::new(ShapeStyle::Role(Role::ToggleAccent))
                .disabled(ShapeStyle::Color(disabled_palette().segment)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent))
                .disabled(ShapeStyle::Color(disabled_palette().segment_label)),
            border: None,
            selected_shadows: vec![],
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
            font: Font {
                size: SEGMENTED.font_size as u32,
                weight: SEGMENTED.font_weight as u16,
                ..BODY
            },
        };
        if crate::compact::toolbar_glass() {
            style.on_glass()
        } else {
            style
        }
    }
}

impl SegmentedControlStyle {
    // SwiftUI's toolbar segmented control on its own glass capsule: veil thumbs, full-strength labels, no dividers.
    fn on_glass(self) -> Self {
        let selected = per_scheme([
            GLASS_CONTROL.selected_light,
            GLASS_CONTROL.selected_dark,
            GLASS_CONTROL.selected_dark,
        ]);
        let ink = StateValue::new(ShapeStyle::Color(glass_ink()))
            .disabled(ShapeStyle::Role(Role::TertiaryLabel));
        Self {
            padding: GLASS_CONTROL.inset,
            gap: GLASS_CONTROL.gap,
            segment_height: GLASS_CONTROL.height - 2.0 * GLASS_CONTROL.inset,
            segment_padding_x: GLASS_CONTROL.segment_padding_x,
            divider: StateValue::new(ShapeStyle::Clear),
            shape: Shape::Capsule,
            segment_shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Clear),
            segment: StateValue::new(ShapeStyle::Clear).hovered(ShapeStyle::Color(glass_hover())),
            selected: StateValue::new(ShapeStyle::Color(selected)),
            label: ink.clone(),
            selected_label: ink,
            ..self
        }
    }
}
