use ctdesign::{CAPTION, Font, HAIRLINE, SPACE_2, TOOLBAR_TABS};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

/// States: a tab of either view takes hovered, pressed and focused.
#[derive(Clone, Debug)]
pub struct TabViewStyle {
    /// Tengen, with `spacing`, `content_spacing`: `TOOLBAR_TABS.padding` inside the strip,
    /// `TOOLBAR_TABS.gap` between tabs, `SPACE_2` between the strip and the page.
    pub padding: f32,
    /// Tengen, with `padding`, `content_spacing`: `TOOLBAR_TABS.padding` inside the strip,
    /// `TOOLBAR_TABS.gap` between tabs, `SPACE_2` between the strip and the page.
    pub spacing: f32,
    /// Tengen, with `padding`, `spacing`: `TOOLBAR_TABS.padding` inside the strip,
    /// `TOOLBAR_TABS.gap` between tabs, `SPACE_2` between the strip and the page.
    pub content_spacing: f32,
    /// Tengen: `TOOLBAR_TABS.tab_min_width`.
    pub tab_min_width: f32,
    /// Tengen: `(tab_padding_top, tab_padding_bottom)`.
    pub tab_padding: (f32, f32),
    /// Tengen: `TOOLBAR_TABS.icon`.
    pub icon_size: f32,
    /// Tengen: `TOOLBAR_TABS.icon_gap`.
    pub icon_spacing: f32,
    /// Tengen: `RoundedRectangle` of `TOOLBAR_TABS.radius` and of `TOOLBAR_TABS.tab_radius`,
    /// `Circular`.
    pub shape: Shape,
    /// Tengen: a `Capsule` rounds by half the height of the strip or the tab.
    pub tab_shape: Shape,
    /// Tengen: `Role(SunkenBackground)`, the strip.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Role(ControlBackground)`.
    pub tab_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(ControlHover)`.
    pub selected_tab_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, `Accent`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: the icon takes the same paint.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn inside the strip.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION` at `TOOLBAR_TABS.font_size` with `TOOLBAR_TABS.font_weight`; one line,
    /// so `line_height` only sets the tab's height for a `Capsule`.
    pub font: Font,
}

impl ControlStyle for TabViewStyle {
    fn tengen() -> Self {
        Self {
            padding: TOOLBAR_TABS.padding,
            spacing: TOOLBAR_TABS.gap,
            content_spacing: SPACE_2,
            tab_min_width: TOOLBAR_TABS.tab_min_width,
            tab_padding: (
                TOOLBAR_TABS.tab_padding_top,
                TOOLBAR_TABS.tab_padding_bottom,
            ),
            icon_size: TOOLBAR_TABS.icon,
            icon_spacing: TOOLBAR_TABS.icon_gap,
            shape: rounded(TOOLBAR_TABS.radius),
            tab_shape: rounded(TOOLBAR_TABS.tab_radius),
            background: StateValue::new(ShapeStyle::Role(Role::SunkenBackground)),
            tab_background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlBackground)),
            selected_tab_background: StateValue::new(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            selected_label: StateValue::new(ShapeStyle::Accent),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            effect: system_effect,
            font: Font {
                size: TOOLBAR_TABS.font_size as u32,
                weight: TOOLBAR_TABS.font_weight as u16,
                ..CAPTION
            },
        }
    }
}
