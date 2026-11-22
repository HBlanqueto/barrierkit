use ctappearance::ColorScheme;
use ctdesign::{
    CORNER_ALERT, CORNER_SHEET, DIALOG_PADDING, DIALOG_SOLID_DARK, DIALOG_SOLID_LIGHT,
    DIALOG_SOLID_SCRIM_LIGHT, HAIRLINE, SHADOW, SHADOW_DARK,
};

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
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
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
            effect: system_effect,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;

    use super::*;
    use crate::styles::{Theme, resolve};

    struct Probe {
        contrast: bool,
        reduce_transparency: bool,
        dark: bool,
        seen: Rc<RefCell<Option<SheetStyle>>>,
    }

    impl App for Probe {
        fn render(&self) -> impl IntoElement {
            let (contrast, reduce_transparency, dark) =
                (self.contrast, self.reduce_transparency, self.dark);
            crate::theme::use_appearance(move |config| {
                config.contrast = contrast;
                config.reduce_transparency = reduce_transparency;
                config.color_scheme = if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                };
            });
            *self.seen.borrow_mut() = Some(resolve(&Theme::tengen()));
            rect()
        }
    }

    fn sheet_style(contrast: bool, reduce_transparency: bool, dark: bool) -> SheetStyle {
        let seen = Rc::new(RefCell::new(None));
        let app = AppComponent::new(Probe {
            contrast,
            reduce_transparency,
            dark,
            seen: seen.clone(),
        });
        let (mut runner, _) = freya_testing::TestingRunner::new(
            app,
            torin::prelude::Size2D::new(10.0, 10.0),
            |_| {},
            1.0,
        );
        runner.sync_and_update();
        seen.take().expect("the probe rendered")
    }

    #[test]
    fn the_sheet_is_solid_with_increase_contrast_which_implies_reduce_transparency() {
        for (contrast, reduce_transparency, dark, solid) in [
            (true, true, false, true),
            (true, true, true, true),
            (true, false, false, true),
            (false, true, true, false),
        ] {
            let style = sheet_style(contrast, reduce_transparency, dark);
            let fill = matches!(
                style.background.resolve(Default::default()),
                ShapeStyle::Color(_)
            );
            assert_eq!(fill, solid, "{contrast} {reduce_transparency} {dark}");
        }
        let light = sheet_style(true, true, false);
        let dark = sheet_style(true, true, true);
        assert_eq!(
            light.background.resolve(Default::default()),
            &ShapeStyle::Color(DIALOG_SOLID_LIGHT)
        );
        assert_eq!(
            dark.background.resolve(Default::default()),
            &ShapeStyle::Color(DIALOG_SOLID_DARK)
        );
        assert_eq!(
            light.scrim.resolve(Default::default()),
            &ShapeStyle::Color(DIALOG_SOLID_SCRIM_LIGHT)
        );
        assert_eq!(
            dark.scrim.resolve(Default::default()),
            &ShapeStyle::Role(Role::Scrim)
        );
    }
}
