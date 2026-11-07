use ctappearance::Resolved;
use ctdesign::{CAPTION, Font, HAIRLINE, SPACE_2, TOOLBAR_TABS};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TabViewStyle {
    pub padding: f32,
    pub spacing: f32,
    pub content_spacing: f32,
    pub tab_min_width: f32,
    pub tab_padding: (f32, f32),
    pub icon_size: f32,
    pub icon_spacing: f32,
    pub shape: Shape,
    pub tab_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub tab_background: StateValue<ShapeStyle>,
    pub selected_tab_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}
