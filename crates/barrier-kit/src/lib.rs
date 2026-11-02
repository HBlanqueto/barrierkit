//! BarrierKit: UI toolkit for the shell and the system's own apps.
//! Provides windows, titlebars, traffic lights, widgets and text.

mod fonts;

pub mod platform;

#[cfg(feature = "shell")]
pub mod shell;

pub mod password;

pub mod theme;

pub mod titlebar;

pub mod traffic;

pub mod animation;

pub mod interaction;

pub mod controls;

pub mod choices;

pub mod progress;

pub mod level;

pub mod compact;

pub mod adornments;

pub mod symbol;

pub mod menu;

pub mod app_menu;

pub mod sidebar;

use freya_core::prelude::{AccessibilityExt, Rect};

pub trait ProbeExt: Sized {
    fn probe(self, id: &'static str) -> Self;
}

impl ProbeExt for Rect {
    fn probe(self, id: &'static str) -> Self {
        self.a11y_builder(move |node| node.set_author_id(id))
    }
}
