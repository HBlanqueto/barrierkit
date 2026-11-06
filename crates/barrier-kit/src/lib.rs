mod fonts;
mod logging;

pub use fonts::register_font;
pub use logging::init_tracing;

pub mod i18n;

pub mod platform;

#[cfg(feature = "shell")]
pub mod shell;

pub mod password;

pub mod theme;

pub mod titlebar;

pub mod chrome;

pub mod traffic;

pub mod animation;

#[cfg(feature = "video")]
pub mod video;

pub mod interaction;

pub mod controls;

pub mod choices;

pub mod progress;

pub mod media;

mod scroll;

pub mod level;

pub mod compact;

pub mod adornments;

pub mod charts;

pub mod popup_button;
pub mod rich_text;

pub mod buttons;

pub mod combo_box;

pub mod text_editor;

pub mod popover;

pub mod drag;
pub mod image_well;
pub mod indicator;

pub mod token_field;

pub mod path_control;

pub mod date;

pub mod date_picker;

pub mod portal;

pub mod color_picker;

pub mod context_menu;

pub mod tooltip;

pub mod backdrop;

mod glass;

mod controlled;

pub mod adaptive;

pub mod style;

pub mod app;

pub mod views;

pub mod patterns;

pub mod symbol;

pub mod menu;

pub mod app_menu;

pub mod sidebar;

use std::rc::Rc;

use freya_core::element::ElementExt;
use freya_core::prelude::{AccessibilityExt, Rect};
use freya_core::tree::DiffModifies;

pub trait ProbeExt: Sized {
    fn probe(self, id: &'static str) -> Self;
}

impl ProbeExt for Rect {
    fn probe(self, id: &'static str) -> Self {
        self.a11y_builder(move |node| node.set_author_id(id))
    }
}

pub(crate) fn style_diff<T: PartialEq + 'static>(
    element: &T,
    other: &Rc<dyn ElementExt>,
) -> DiffModifies {
    match (other.as_ref() as &dyn std::any::Any).downcast_ref::<T>() {
        Some(other) if other == element => DiffModifies::empty(),
        Some(_) => DiffModifies::STYLE,
        None => DiffModifies::all(),
    }
}
