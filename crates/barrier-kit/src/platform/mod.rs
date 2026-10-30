//! The Wayland runner for Freya's core (ADR 0034).
//!
//! `freya-winit` cannot be used: it drags in winit, GTK, the tray stack and
//! glutin, and winit has no layer-shell or session-lock support. This module
//! owns the other half: it creates the surface (layer-shell or
//! ext-session-lock), an EGL context on the Wayland display, a Skia GL
//! context, and feeds input into a Freya [`Runner`]. Each surface gets its own
//! runner, tree, font collection and Skia context.
//!
//! The event loop is calloop (the same one the shell already uses); Freya's
//! futures are polled from its ping source, never from a runtime thread.

pub(crate) mod app;
pub(crate) mod app_menu_export;
mod egl;
pub(crate) mod extension;
pub(crate) mod input;
pub(crate) mod popup;
pub(crate) mod runtime;
pub(crate) mod surface;
#[cfg(test)]
mod tests;
pub(crate) mod timer;
pub(crate) mod window;

pub use app_menu_export::AppMenuAddress;
pub use input::inserts_text;
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timers, WakeHandle};
pub use window::{
    DecorationResponse, Decorations, MenuActionHandler, WindowControls, WindowEdge, WindowOptions,
    WindowStatus, run_window,
};

#[cfg(test)]
pub(crate) use popup::PopupCommand;
pub use sctk::reexports::client::backend::ObjectId;

/// A failure while setting up or running the surface.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("WAYLAND_DISPLAY is not set to a reachable socket")]
    NoDisplay,
    #[error("the compositor does not advertise {0}")]
    MissingGlobal(&'static str),
    #[error("EGL initialisation failed: {0}")]
    Egl(&'static str),
    #[error("event loop failed: {0}")]
    EventLoop(&'static str),
    #[error("Skia could not create its GL context")]
    Skia,
    #[error(transparent)]
    Wayland(#[from] sctk::reexports::client::ConnectError),
    #[error(transparent)]
    Registry(#[from] sctk::reexports::client::globals::GlobalError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
