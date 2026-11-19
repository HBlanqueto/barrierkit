use ctappearance::ColorScheme;
use ctdesign::{DIALOG, DIALOG_PADDING, HAIRLINE, SHADOW, SHADOW_DARK};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

/// `Alert` draws on a sheet: it takes `SheetStyle` from the theme and the
/// containers, and `AlertStyle` for its own parts. Its buttons take
/// `ButtonStyle` and its checkbox `CheckboxStyle`.
///
/// States: only `normal`. Like `Badge`, `Sheet::style` and
/// `ContentUnavailableView::style` build the view again, so they come right
/// after `new`.
#[derive(Clone, Debug)]
pub struct SheetStyle {
    /// Tengen: `padding`: `DIALOG_PADDING`.
    pub padding: f32,
    /// Tengen: `RoundedRectangle { radius: DIALOG.corner_radius, style: Continuous }`; a
    /// `Capsule` or `Circle` stays square.
    pub shape: Shape,
    /// Tengen: `scrim` `Role(Scrim)` over the window.
    pub scrim: StateValue<ShapeStyle>,
    /// Tengen: `background` `Role(RaisedBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn outside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: the window shadow, `SHADOW` (`SHADOW_DARK` in dark): the sheet floats like a window.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for SheetStyle {
    fn tengen() -> Self {
        let shadow = if crate::theme::resolved().scheme == ColorScheme::Light {
            SHADOW
        } else {
            SHADOW_DARK
        };
        Self {
            padding: DIALOG_PADDING,
            shape: Shape::RoundedRectangle {
                radius: DIALOG.corner_radius as f32,
                style: CornerStyle::Continuous,
            },
            scrim: StateValue::new(ShapeStyle::Role(Role::Scrim)),
            background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            shadows: [shadow.ambient, shadow.contact]
                .into_iter()
                .map(|layer| ShadowStyle {
                    color: ctdesign::Rgba {
                        a: layer.opacity,
                        ..layer.color
                    },
                    ..layer.into()
                })
                .collect(),
            effect: system_effect,
        }
    }
}
