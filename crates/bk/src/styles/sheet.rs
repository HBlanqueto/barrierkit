use ctappearance::Resolved;
use ctdesign::{DIALOG, DIALOG_PADDING, HAIRLINE};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct SheetStyle {
    pub padding: f32,
    pub shape: Shape,
    pub scrim: StateValue<ShapeStyle>,
    pub background: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}
