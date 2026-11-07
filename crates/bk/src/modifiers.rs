use ctdesign::Rgba;
use freya_core::prelude::*;

use crate::menu::MenuItem;

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
        Self: ChildrenExt,
    {
        crate::interaction::ring(self, corner_radius, color)
    }
}

impl<T: EventHandlersExt> ViewModifiers for T {}
