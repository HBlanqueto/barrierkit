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

mod activation;
mod app;
mod app_menu_export;
mod egl;
mod foreign_menus;
mod infinity;
mod input;
mod layer;
mod lock;
mod popup;
mod runtime;
mod surface;
#[cfg(test)]
mod tests;
mod timer;
mod toplevels;
mod window;

pub use activation::Activation;
pub use app_menu_export::AppMenuAddress;
pub use foreign_menus::GtkMenusAddress;
pub use input::inserts_text;
pub use layer::{Anchors, KeyboardInteractivity, Layer, run_layer};
pub use lock::{SessionUnlock, run_lock};
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timers, WakeHandle};
pub use toplevels::{ToplevelInfo, Toplevels};
pub use window::{
    DecorationResponse, Decorations, MenuActionHandler, WindowControls, WindowEdge, WindowOptions,
    WindowStatus, run_window,
};

pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
#[cfg(test)]
pub(crate) use popup::PopupCommand;
pub use sctk::reexports::client::backend::ObjectId;

/// What kind of surface the runner owns.
#[derive(Clone, Debug)]
pub enum SurfaceSpec {
    /// A `zwlr_layer_surface_v1` on the given layer.
    Layer {
        namespace: String,
        layer: Layer,
        anchors: Anchors,
        exclusive_zone: i32,
        keyboard: KeyboardInteractivity,
        /// Logical size, or `None` to let the compositor decide.
        size: Option<(u32, u32)>,
    },
    /// One `ext-session-lock-v1` surface per output.
    Lock,
    /// A normal xdg_toplevel owned by a BarrierKit app.
    Window(WindowOptions),
}

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
