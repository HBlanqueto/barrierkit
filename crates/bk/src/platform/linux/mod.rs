mod activation;
mod app;
#[cfg(feature = "a11y")]
pub(super) mod atspi;
mod chrome_surface;
pub(crate) mod clipboard;
mod dialog;
#[cfg(test)]
mod doc_shots;
pub(crate) mod extension;
#[cfg(test)]
pub(super) mod fake_server;
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
#[cfg(test)]
mod scene_tests;
mod seat;
pub(crate) mod surface;
mod system;
#[cfg(test)]
mod tests;
mod timer;
mod vulkan;
mod window;
mod window_style;

pub use sctk::reexports::client::backend::ObjectId;

pub(crate) const SYSTEM_MOMENTUM: bool = false;
// Malaysia serves 4 requesting surfaces per toplevel and the sidebar's is the window's own (ADR 0113).
pub(crate) const MOST_CHROMES: usize = 3;

#[cfg(feature = "a11y")]
pub(super) use atspi::Bridge;
pub(super) use window::run_window;
