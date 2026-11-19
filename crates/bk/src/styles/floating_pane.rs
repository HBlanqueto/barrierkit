use ctdesign::{CAPTION, FLOATING_PANE, Font};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct FloatingPaneStyle {
    /// Tengen, with `outline_width`: `title_height`, `outline_width`:
    /// `FLOATING_PANE.title_height`, `FLOATING_PANE.outline`, drawn outside.
    pub title_height: f32,
    /// Tengen: `Rectangle`; the title bar takes its top corners.
    pub shape: Shape,
    /// Tengen, with `title_bar`, `title`, `outline`: `background` `Role(WindowBackground)`,
    /// `title_bar` `Role(ControlBackground)`, `title` `Role(Label)`, `outline`
    /// `Role(Separator)`, focused `Accent`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `title`, `outline`: `background` `Role(WindowBackground)`,
    /// `title_bar` `Role(ControlBackground)`, `title` `Role(Label)`, `outline`
    /// `Role(Separator)`, focused `Accent`.
    pub title_bar: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `title_bar`, `outline`: `background`
    /// `Role(WindowBackground)`, `title_bar` `Role(ControlBackground)`, `title` `Role(Label)`,
    /// `outline` `Role(Separator)`, focused `Accent`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `title_bar`, `title`: `background` `Role(WindowBackground)`,
    /// `title_bar` `Role(ControlBackground)`, `title` `Role(Label)`, `outline`
    /// `Role(Separator)`, focused `Accent`.
    pub outline: StateValue<ShapeStyle>,
    /// Tengen, with `title_height`: `title_height`, `outline_width`:
    /// `FLOATING_PANE.title_height`, `FLOATING_PANE.outline`, drawn outside.
    pub outline_width: f32,
    /// Tengen: `[FLOATING_PANE.shadow]` with `shadow_y` and `shadow_blur`.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`.
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
}
