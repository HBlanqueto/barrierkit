//! Wayland runner that drives Freya's core.

mod activation;
pub(crate) mod app;
mod app_menu_export;
mod atspi;
mod clipboard;
#[cfg(feature = "video")]
pub(crate) mod dmabuf;
#[cfg(test)]
mod doc_shots;
pub(crate) mod extension;
mod focus;
mod input;
mod popup;
pub(crate) mod runtime;
pub(crate) mod surface;
#[cfg(test)]
mod tests;
pub(crate) mod timer;
mod vulkan;
mod window;
mod window_style;

pub use app_menu_export::AppMenuAddress;
pub use input::inserts_text;
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timers, WakeHandle};
pub(crate) use window::window_activated;
pub use window::{
    DecorationResponse, FullscreenBar, WindowControls, WindowEdge, WindowOptions, WindowStatus,
    run_window,
};

#[cfg(test)]
pub(crate) use popup::PopupCommand;
pub use sctk::reexports::client::backend::ObjectId;

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("WAYLAND_DISPLAY is not set to a reachable socket")]
    NoDisplay,
    #[error("the compositor does not advertise {0}")]
    MissingGlobal(&'static str),
    #[error("Vulkan failed: {0}")]
    Vulkan(&'static str),
    #[error("event loop failed: {0}")]
    EventLoop(&'static str),
    #[error("Skia could not create its Vulkan context")]
    Skia,
    #[error(transparent)]
    Wayland(#[from] sctk::reexports::client::ConnectError),
    #[error(transparent)]
    Registry(#[from] sctk::reexports::client::globals::GlobalError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
