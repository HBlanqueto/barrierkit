mod activation;
mod app;
#[cfg(feature = "a11y")]
pub(super) mod atspi;
mod chrome_surface;
pub(crate) mod clipboard;
mod dialog;
mod effects;
pub(crate) mod extension;
mod focus;
mod glass;
mod ime;
pub(crate) mod input;
pub(crate) mod instance;
pub(crate) mod keychain;
mod main_menu_export;
pub(crate) mod notifications;
pub(crate) mod panels;
pub(crate) mod pasteboard;
mod popup;
mod portal;
mod primary;
pub(crate) mod runtime;
mod seat;
pub(crate) mod surface;
mod system;
mod timer;
mod vulkan;
mod window;
mod window_style;

use std::path::PathBuf;

pub(crate) use ctappearance::language::locale_languages as preferred_languages;
use freya_core::prelude::Modifiers;
pub use sctk::reexports::client::backend::ObjectId;

pub(crate) fn blocking_session_bus() -> zbus::Result<zbus::blocking::Connection> {
    async_io::block_on(crate::session_bus::connection()).map(Into::into)
}

pub(crate) const SYSTEM_MOMENTUM: bool = false;
pub(crate) const NATIVE_LIGHTS: bool = false;
pub(crate) const OPTICAL_SIZING: bool = false;
// Fontconfig resolves the generic names to the faces the desktop sets as default.
pub(crate) const SYSTEM_FONTS: (&str, &str) = ("sans-serif", "monospace");
pub(crate) const MODIFIER_GLYPHS: [(Modifiers, &str); 4] = [
    (Modifiers::CONTROL, "⌃"),
    (Modifiers::ALT, "⌥"),
    (Modifiers::SHIFT, "⇧"),
    (Modifiers::META, "❖"),
];
// Malaysia serves 4 requesting surfaces per toplevel and the sidebar's is the window's own (docs/barrierkit/15-backdrops-and-media.md#glass-on-any-view).
pub(crate) const MOST_CHROMES: usize = 3;

pub(crate) fn app_folders(app_id: &str) -> (PathBuf, PathBuf) {
    let folder = |home: Option<PathBuf>, fallback: &str| {
        home.unwrap_or_else(|| PathBuf::from(fallback)).join(app_id)
    };
    (
        folder(ctappearance::xdg::config_home(), ".config"),
        folder(ctappearance::xdg::data_home(), ".local/share"),
    )
}

#[cfg(feature = "a11y")]
pub(super) use atspi::Bridge;
pub(crate) use system::appearance;
pub(super) use window::run_window;
