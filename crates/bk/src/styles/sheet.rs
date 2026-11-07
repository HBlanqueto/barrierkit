use ctdesign::{DIALOG, DIALOG_PADDING, HAIRLINE};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
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
    /// Tengen, with `background`: `scrim` `Role(Scrim)` over the window; `background`
    /// `Role(RaisedBackground)`.
    pub scrim: StateValue<ShapeStyle>,
    /// Tengen, with `scrim`: `scrim` `Role(Scrim)` over the window; `background`
    /// `Role(RaisedBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn outside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for SheetStyle {
    fn tengen() -> Self {
        Self {
            padding: DIALOG_PADDING,
            shape: Shape::RoundedRectangle {
                radius: DIALOG.corner_radius as f32,
                style: CornerStyle::Continuous,
            },
            scrim: StateValue::new(ShapeStyle::Role(Role::Scrim)),
            background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            effect: system_effect,
        }
    }
}
