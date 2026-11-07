use ctdesign::Rgba;
use freya_core::prelude::*;

use crate::menu::MenuItem;
use crate::styles::Painted;
use crate::styles::{
    ControlState, ControlStyle, Paints, ShadowStyle, ShapeStyle, StateValue, StyleScope, Theme,
    system_effect,
};

pub trait ViewModifiers: EventHandlersExt + Sized {
    fn context_menu(
        self,
        items: Vec<MenuItem>,
        on_activate: impl Into<Callback<String, ()>>,
    ) -> Self {
        crate::context_menu::context_menu(self, items, on_activate)
    }

    fn help(self, text: &'static str) -> Self {
        crate::tooltip::use_tooltip(text).attach(self)
    }

    fn focus_ring(self, corner_radius: f32, color: Rgba) -> Self
    where
        Self: StyleExt,
    {
        crate::interaction::ring(self, corner_radius, color)
    }

    fn foreground_style(self, style: ShapeStyle) -> Self
    where
        Self: TextStyleExt,
    {
        let paints = Paints::current(ControlState::default(), system_effect);
        self.color(paints.fill(&StateValue::new(style)))
    }

    fn background_style(self, style: ShapeStyle) -> Self
    where
        Self: StyleExt + EffectExt,
    {
        self.painted(
            &Paints::current(ControlState::default(), system_effect),
            &StateValue::new(style),
        )
    }

    fn shadows(self, shadows: Vec<ShadowStyle>) -> Self
    where
        Self: StyleExt,
    {
        shadows
            .into_iter()
            .fold(self, |view, shadow| view.shadow(Shadow::from(shadow)))
    }
}

impl<T: EventHandlersExt> ViewModifiers for T {}

pub trait StyleModifiers: IntoElement + Sized {
    fn control_style<S: ControlStyle>(self, restyle: impl Fn(S) -> S + 'static) -> StyleScope {
        StyleScope::new(Theme::tengen().style(restyle), self)
    }
}

impl<T: IntoElement> StyleModifiers for T {}
