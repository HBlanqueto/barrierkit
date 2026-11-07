use ctappearance::Resolved;
use ctdesign::{CAPTION, FLOATING_PANE, Font};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct FloatingPaneStyle {
    pub title_height: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub title_bar: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub outline: StateValue<ShapeStyle>,
    pub outline_width: f32,
    pub shadows: Vec<ShadowStyle>,
    pub effect: ColorEffect,
    pub title_font: Font,
}

impl ControlStyle for FloatingPaneStyle {
    fn tengen() -> Self {
        Self {
            title_height: FLOATING_PANE.title_height,
            shape: Shape::Rectangle,
            background: StateValue::new(ShapeStyle::Role(Role::WindowBackground)),
            title_bar: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            title: StateValue::new(ShapeStyle::Role(Role::Label)),
            outline: StateValue::new(ShapeStyle::Role(Role::Separator)).focused(ShapeStyle::Accent),
            outline_width: FLOATING_PANE.outline,
            shadows: vec![ShadowStyle {
                color: FLOATING_PANE.shadow,
                x: 0.0,
                y: FLOATING_PANE.shadow_y,
                blur: FLOATING_PANE.shadow_blur,
                spread: 0.0,
                inner: false,
            }],
            effect: system_effect,
            title_font: CAPTION,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}
