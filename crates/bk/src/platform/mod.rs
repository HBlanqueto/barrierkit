mod activation;
pub(crate) mod app;
mod atspi;
mod clipboard;
#[cfg(feature = "video")]
pub(crate) mod dmabuf;
#[cfg(test)]
mod doc_shots;
pub(crate) mod extension;
mod focus;
mod ime;
mod input;
mod main_menu_export;
mod popup;
mod primary;
pub(crate) mod runtime;
pub(crate) mod surface;
#[cfg(test)]
mod tests;
pub(crate) mod timer;
mod vulkan;
mod window;
mod window_style;

pub use input::inserts_text;
pub use main_menu_export::MainMenuAddress;
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timers, WakeHandle};

pub(crate) struct EditShortcut {
    pub(crate) field: freya_core::prelude::AccessibilityId,
    pub(crate) key: String,
}
pub use window::{
    DecorationResponse, FullscreenBar, PanelBehavior, WindowControls, WindowEdge, WindowOptions,
    WindowStatus, run_window,
};
pub(crate) use window::{open_utility_window, window_activated};
pub use window_style::Bar;

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
    #[error("the Vulkan device was lost")]
    DeviceLost,
    #[error("the Vulkan surface was lost")]
    SurfaceLost,
    #[error("a layer surface without a width or height must be anchored to both opposite edges")]
    LayerSize,
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
