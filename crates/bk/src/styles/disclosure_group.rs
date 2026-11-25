use ctdesign::{
    BODY, DISCLOSURE, DISCLOSURE_TRIANGLE_CONTRAST_DARK, DISCLOSURE_TRIANGLE_CONTRAST_DARK_TINT,
    DISCLOSURE_TRIANGLE_CONTRAST_LIGHT, DISCLOSURE_TRIANGLE_DARK, DISCLOSURE_TRIANGLE_DARK_TINT,
    DISCLOSURE_TRIANGLE_LIGHT, DURATION_DISCLOSURE_MS, Font, ease_default,
};

use crate::styles::{Animation, ControlStyle, Role, ShapeStyle, StateValue, per_look};

pub(super) fn disclosure_triangle() -> StateValue<ShapeStyle> {
    StateValue::new(ShapeStyle::Color(per_look(
        [
            DISCLOSURE_TRIANGLE_LIGHT,
            DISCLOSURE_TRIANGLE_DARK,
            DISCLOSURE_TRIANGLE_DARK_TINT,
        ],
        [
            DISCLOSURE_TRIANGLE_CONTRAST_LIGHT,
            DISCLOSURE_TRIANGLE_CONTRAST_DARK,
            DISCLOSURE_TRIANGLE_CONTRAST_DARK_TINT,
        ],
    )))
}

#[derive(Clone, Debug)]
pub struct DisclosureGroupStyle {
    /// Tengen: `DISCLOSURE.row_height`.
    pub row_height: f32,
    /// Tengen: `0`, the indent of the children.
    pub indent: f32,
    /// Tengen: `DISCLOSURE.group_lead`, the column of the chevron.
    pub lead_width: f32,
    /// Tengen: `(DISCLOSURE.group_chevron, DISCLOSURE.group_chevron_stroke)`.
    pub chevron_size: (f32, f32),
    /// Tengen: `(DISCLOSURE.group_chevron_x, 0)`, where the closed chevron sits from the center of its column.
    pub chevron_offset: (f32, f32),
    /// Tengen: `(DISCLOSURE.group_chevron_open_x, DISCLOSURE.group_chevron_open_y)`, the same for the open chevron.
    pub open_chevron_offset: (f32, f32),
    /// Tengen: `Color(DISCLOSURE_TRIANGLE)` (`DISCLOSURE_TRIANGLE_CONTRAST` under Increase
    /// Contrast).
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `label` `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Custom { duration_ms: DURATION_DISCLOSURE_MS, curve: ease_default }`, SwiftUI's `DisclosureGroup`: the content unrolls and the chevron turns on the same curve; `None` makes them jump.
    pub animation: Animation,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for DisclosureGroupStyle {
    fn tengen() -> Self {
        Self {
            row_height: DISCLOSURE.row_height,
            indent: 0.0,
            lead_width: DISCLOSURE.group_lead,
            chevron_size: (DISCLOSURE.group_chevron, DISCLOSURE.group_chevron_stroke),
            chevron_offset: (DISCLOSURE.group_chevron_x, 0.0),
            open_chevron_offset: (
                DISCLOSURE.group_chevron_open_x,
                DISCLOSURE.group_chevron_open_y,
            ),
            chevron: disclosure_triangle(),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            animation: Animation::Custom {
                duration_ms: DURATION_DISCLOSURE_MS,
                curve: ease_default,
            },
            font: BODY,
        }
    }
}
