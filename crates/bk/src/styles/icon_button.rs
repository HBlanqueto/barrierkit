use ctappearance::Resolved;
use ctdesign::{BUTTON, ICON_BUTTON, TOOLGROUP};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

/// States: hovered, pressed, focused, disabled. `ToolbarOverflow` is an
/// `IconButton`: its `.style` takes an `IconButtonStyle`.
#[derive(Clone, Debug)]
pub struct IconButtonStyle {
    /// Tengen: `ICON_BUTTON.size`, in a toolbar `TOOLGROUP.height`.
    pub size: f32,
    /// Tengen: `ICON_BUTTON.icon`.
    pub icon_size: f32,
    /// Tengen: `RoundedRectangle { radius: ICON_BUTTON.radius, style: Circular }`, in a toolbar
    /// `Capsule`.
    pub shape: Shape,
    /// Tengen: `Clear`, hovered `Role(ControlHover)`, disabled `Clear`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, disabled `Role(TertiaryLabel)`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `1.0`.
    pub disabled_opacity: f32,
    /// Tengen: `BUTTON.pressed_scale`.
    pub pressed_scale: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `None`: the pressed scale jumps.
    pub animation: Animation,
}

impl ControlStyle for IconButtonStyle {
    fn tengen() -> Self {
        let (size, shape) = if crate::compact::in_toolbar() {
            (TOOLGROUP.height as f32, Shape::Capsule)
        } else {
            (ICON_BUTTON.size, rounded(ICON_BUTTON.radius))
        };
        Self {
            size,
            icon_size: ICON_BUTTON.icon,
            shape,
            background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover))
                .disabled(ShapeStyle::Clear),
            icon: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            animation: Animation::None,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}
