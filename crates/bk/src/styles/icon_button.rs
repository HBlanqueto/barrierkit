use ctappearance::{ColorScheme, Resolved};
use ctdesign::{
    BUTTON, DURATION_FAST_MS, HAIRLINE, ICON_BUTTON, SYMBOL, TOOLBAR_ITEM_HIGHLIGHT, TOOLGROUP,
};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    pressed_palette, reduce_motion, show_borders, system_effect,
};

/// States: pressed, focused, disabled; hovered in a toolbar on native glass. `ToolbarOverflow` is an
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
        // shortcut: assumes interactive glass (macOS 27+), so a press on macOS 26 shows nothing; pass the platform's answer if 26 matters.
        let on_glass = crate::compact::toolbar_glass();
        let (size, icon_size, stroke, shape) = match toolbar {
            Some(_) => {
                let height = TOOLGROUP.height as f32;
                let item = if on_glass {
                    TOOLBAR_ITEM_HIGHLIGHT.size
                } else {
                    height
                };
                let size = crate::compact::slim_toolbar_items()
                    .map_or((item, item), |(width, height, inset)| {
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
            // On native glass the glass lights a press itself, as in SwiftUI, and the item lays its own circle over that light.
            background: if on_glass {
                let (hover, pressed) = match crate::theme::resolved().scheme {
                    ColorScheme::Light => (
                        TOOLBAR_ITEM_HIGHLIGHT.hover_light,
                        TOOLBAR_ITEM_HIGHLIGHT.pressed_light,
                    ),
                    ColorScheme::Dark | ColorScheme::DarkTint => (
                        TOOLBAR_ITEM_HIGHLIGHT.hover_dark,
                        TOOLBAR_ITEM_HIGHLIGHT.pressed_dark,
                    ),
                };
                StateValue::new(ShapeStyle::Clear)
                    .hovered(ShapeStyle::Color(hover))
                    .pressed(ShapeStyle::Color(pressed))
                    .disabled(ShapeStyle::Clear)
            } else {
                StateValue::new(ShapeStyle::Clear)
                    .pressed(ShapeStyle::Color(pressed_palette().bezel))
                    .disabled(ShapeStyle::Clear)
            },
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
