use ctappearance::Resolved;
use ctdesign::{BUTTON, DURATION_FAST_MS, HAIRLINE, ICON_BUTTON, SYMBOL, TOOLGROUP};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    pressed_palette, reduce_motion, show_borders, system_effect,
};

/// States: pressed, focused, disabled. `ToolbarOverflow` is an
/// `IconButton`: its `.style` takes an `IconButtonStyle`.
#[derive(Clone, Debug)]
pub struct IconButtonStyle {
    /// Tengen: width and height, `ICON_BUTTON.size` square; in a unified toolbar `TOOLGROUP.height`
    /// square; in a compact one `TOOLGROUP.compact_width` less twice `TOOLGROUP.compact_inset` by
    /// `TOOLGROUP.compact_height`, in an expanded one the same with the `expanded_` metrics.
    pub size: (f32, f32),
    /// Tengen: `ICON_BUTTON.icon`, in a toolbar `ICON_BUTTON.toolbar_icon`.
    pub icon_size: f32,
    /// Tengen: `SYMBOL.stroke` scaled to `icon_size`, in a toolbar `ICON_BUTTON.toolbar_stroke`. In
    /// pixels, not grid units.
    pub stroke: f32,
    /// Tengen: `RoundedRectangle { radius: ICON_BUTTON.radius, style: Circular }`, in a toolbar
    /// `Capsule`.
    pub shape: Shape,
    /// Tengen: `Clear`, pressed `Color(PressedPalette.bezel)`, disabled `Clear`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, disabled `Role(TertiaryLabel)`; in a toolbar of an inactive window
    /// `Role(TertiaryLabel)`, as AppKit dims toolbar items.
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
    /// Tengen: `Ease { duration_ms: DURATION_FAST_MS }`: the pressed scale eases; `None` makes it
    /// jump.
    pub animation: Animation,
}

impl ControlStyle for IconButtonStyle {
    fn tengen() -> Self {
        let toolbar = crate::compact::toolbar_style();
        let dimmed = toolbar.is_some() && !crate::platform::window_activated();
        let (size, icon_size, stroke, shape) = match toolbar {
            Some(_) => {
                let height = TOOLGROUP.height as f32;
                let size = crate::compact::slim_toolbar_items()
                    .map_or((height, height), |(width, height, inset)| {
                        (width - 2.0 * inset, height)
                    });
                (
                    size,
                    ICON_BUTTON.toolbar_icon,
                    ICON_BUTTON.toolbar_stroke,
                    Shape::Capsule,
                )
            }
            None => (
                (ICON_BUTTON.size, ICON_BUTTON.size),
                ICON_BUTTON.icon,
                SYMBOL.stroke * ICON_BUTTON.icon / SYMBOL.grid,
                rounded(ICON_BUTTON.radius),
            ),
        };
        Self {
            size,
            icon_size,
            stroke,
            shape,
            background: StateValue::new(ShapeStyle::Clear)
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Clear),
            icon: StateValue::new(ShapeStyle::Role(if dimmed {
                Role::TertiaryLabel
            } else {
                Role::Label
            }))
            .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        let style = reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        });
        show_borders(style, appearance, |style| Self {
            border: Some((ShapeStyle::Role(Role::ControlBorder), HAIRLINE)),
            ..style
        })
    }
}
