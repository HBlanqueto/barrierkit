use ctappearance::ColorScheme;
use ctdesign::{
    CORNER_ALERT, CORNER_SHEET, DIALOG_PADDING, DIALOG_SOLID_DARK, DIALOG_SOLID_LIGHT,
    DIALOG_SOLID_SCRIM_LIGHT, HAIRLINE, SHADOW, SHADOW_DARK,
};

use crate::styles::{ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

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
    /// Tengen: `RoundedRectangle { radius: CORNER_SHEET.fitted, style: Continuous }`, the
    /// silhouette of a sheet; an `Alert` takes `CORNER_ALERT.fitted`. A `Capsule` or `Circle` stays
    /// square.
    pub shape: Shape,
    /// Tengen: `scrim` `Role(Scrim)` over the window; `DIALOG_SOLID_SCRIM_LIGHT` in light with
    /// Increase Contrast, which implies Reduce Transparency.
    pub scrim: StateValue<ShapeStyle>,
    /// Tengen: `background` `Role(WindowBackground)`; `DIALOG_SOLID_LIGHT` or `DIALOG_SOLID_DARK`
    /// with Increase Contrast, which implies Reduce Transparency.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Outline), HAIRLINE))`, drawn outside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: the window shadow, `SHADOW` (`SHADOW_DARK` in dark): the sheet floats like a window.
    pub shadows: Vec<ShadowStyle>,
}

impl SheetStyle {
    pub(crate) fn alert(self) -> Self {
        Self {
            shape: Shape::RoundedRectangle {
                radius: CORNER_ALERT.fitted,
                style: CornerStyle::Continuous,
            },
            ..self
        }
    }
}

impl ControlStyle for SheetStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let light = resolved.scheme == ColorScheme::Light;
        let shadow = if light { SHADOW } else { SHADOW_DARK };
        let solid = resolved.contrast;
        let background = match (solid, light) {
            (false, _) => ShapeStyle::Role(Role::WindowBackground),
            (true, true) => ShapeStyle::Color(DIALOG_SOLID_LIGHT),
            (true, false) => ShapeStyle::Color(DIALOG_SOLID_DARK),
        };
        let scrim = if solid && light {
            ShapeStyle::Color(DIALOG_SOLID_SCRIM_LIGHT)
        } else {
            ShapeStyle::Role(Role::Scrim)
        };
        Self {
            padding: DIALOG_PADDING,
            shape: Shape::RoundedRectangle {
                radius: CORNER_SHEET.fitted,
                style: CornerStyle::Continuous,
            },
            scrim: StateValue::new(scrim),
            background: StateValue::new(background),
            border: Some((ShapeStyle::Role(Role::Outline), HAIRLINE)),
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
        }
    }
}
