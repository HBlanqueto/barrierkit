#[cfg(target_os = "linux")]
mod activation;
pub(crate) mod app;
#[cfg(all(feature = "a11y", target_os = "linux"))]
mod atspi;
pub(crate) mod backdrop;
#[cfg(target_os = "linux")]
mod chrome_surface;
#[cfg(target_os = "linux")]
mod clipboard;
#[cfg(target_os = "macos")]
#[path = "macos/clipboard.rs"]
mod clipboard;
#[cfg(target_os = "linux")]
mod dialog;
#[cfg(all(test, target_os = "linux"))]
mod doc_shots;
#[cfg(target_os = "linux")]
pub(crate) mod extension;
#[cfg(all(test, target_os = "linux"))]
mod fake_server;
#[cfg(target_os = "linux")]
mod focus;
mod glass;
#[cfg(target_os = "linux")]
mod ime;
mod input;
#[cfg(target_os = "macos")]
mod macos;
mod main_menu_export;
mod popup;
#[cfg(target_os = "linux")]
mod primary;
#[cfg(target_os = "linux")]
pub(crate) mod runtime;
#[cfg(all(test, target_os = "linux"))]
mod scene_tests;
#[cfg(target_os = "linux")]
pub(crate) mod surface;
#[cfg(all(test, target_os = "linux"))]
mod tests;
pub(crate) mod timer;
#[cfg(target_os = "linux")]
mod vulkan;
mod window;
mod window_style;

#[cfg(target_os = "macos")]
pub(crate) use ::calloop;
#[cfg(target_os = "linux")]
pub(crate) use sctk::reexports::calloop;

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
pub(crate) use window_style::Knob;
pub use window_style::{Bar, PopoverShape};

#[cfg(feature = "shell")]
pub(crate) use glass::packed;
pub(crate) use glass::{GlassOutbox, GlassPanels, GlassSupport, KnobOutbox, KnobSupport, Knobs};
#[cfg(test)]
pub(crate) use popup::PopupCommand;
pub(crate) use popup::anchor_rect;
#[cfg(target_os = "linux")]
pub use sctk::reexports::client::backend::ObjectId;

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

#[cfg(target_os = "macos")]
impl ObjectId {
    pub fn null() -> Self {
        Self(0)
    }

    fn from_raw(id: u64) -> Self {
        Self(id)
    }

    fn raw(&self) -> u64 {
        self.0
    }
}

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
    #[error("the application declares no window")]
    NoWindow,
    #[error("event loop failed: {0}")]
    EventLoop(&'static str),
    #[error("Skia could not create its Vulkan context")]
    Skia,
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Wayland(#[from] sctk::reexports::client::ConnectError),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Registry(#[from] sctk::reexports::client::globals::GlobalError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
