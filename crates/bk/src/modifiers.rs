use ctdesign::{ControlSize, Rgba};
use freya_core::prelude::*;
use std::borrow::Cow;

use crate::environment::{DisabledScope, EnvironmentScope};
use crate::focus::{DefaultFocus, FocusState, Focused};
use crate::layout::Frame;
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

    fn sheet(self, is_presented: State<bool>, content: impl Fn() -> Element) -> Self
    where
        Self: ChildrenExt,
    {
        let presented = *is_presented.read();
        self.maybe_child(presented.then(|| crate::views::feedback::Sheet::new(content())))
    }

    fn help(self, text: impl Into<Cow<'static, str>>) -> Self {
        crate::tooltip::use_tooltip(text).attach(self)
    }

    fn focus_ring(self, corner_radius: f32, color: Rgba) -> Self
    where
        Self: DecorationExt,
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
        Self: BackgroundExt + DecorationExt + EffectExt,
    {
        self.painted(
            &Paints::current(ControlState::default(), system_effect),
            &StateValue::new(style),
        )
    }

    fn shadows(self, shadows: Vec<ShadowStyle>) -> Self
    where
        Self: DecorationExt,
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

    fn control_size(self, size: ControlSize) -> StyleScope {
        StyleScope::new(Theme::control_size(size), self)
    }

    fn frame(self) -> Frame {
        Frame::new(self)
    }
}

impl<T: IntoElement> StyleModifiers for T {}

pub trait EnvironmentModifiers: IntoElement + Sized {
    fn disabled(self, disabled: bool) -> impl IntoElement {
        DisabledScope::new(disabled, self)
    }

    fn environment<T: Clone + PartialEq + 'static>(self, value: T) -> impl IntoElement {
        EnvironmentScope::new(value, self)
    }

    fn focused<T: Clone + PartialEq + 'static>(
        self,
        state: FocusState<T>,
        value: T,
    ) -> impl IntoElement {
        Focused::new(state, value, self)
    }

    fn default_focus<T: Clone + PartialEq + 'static>(
        self,
        state: FocusState<T>,
        value: T,
    ) -> impl IntoElement {
        DefaultFocus::new(state, value, self)
    }
}

impl<T: IntoElement> EnvironmentModifiers for T {}
