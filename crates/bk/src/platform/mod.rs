mod activation;
pub(crate) mod app;
#[cfg(feature = "a11y")]
mod atspi;
mod clipboard;
mod dialog;
#[cfg(feature = "video")]
pub(crate) mod dmabuf;
#[cfg(test)]
mod doc_shots;
pub(crate) mod extension;
mod focus;
mod glass;
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

pub use input::{inserts_text, modifier_flags};
pub use main_menu_export::MainMenuAddress;
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timers, WakeHandle};

pub(crate) enum Sent {
    User(freya_core::integration::UserEvent),
    Kit(Box<dyn std::any::Any>),
}

#[derive(Clone)]
pub(crate) struct KitSender(pub(crate) std::rc::Rc<dyn Fn(Sent)>);

impl KitSender {
    // Without the kit's runner (freya-testing) these events have nowhere to go.
    pub(crate) fn get() -> Self {
        freya_core::prelude::try_consume_root_context()
            .unwrap_or_else(|| Self(std::rc::Rc::new(|_| {})))
    }

    pub(crate) fn send(&self, event: impl std::any::Any) {
        (self.0)(Sent::Kit(Box::new(event)));
    }
}

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

pub(crate) use glass::{GlassPanels, GlassSupport, set_glass_panels};
#[cfg(test)]
pub(crate) use popup::PopupCommand;
pub(crate) use popup::anchor_rect;
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
