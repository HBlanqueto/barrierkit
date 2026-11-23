pub(crate) mod app;
pub(crate) mod backdrop;
mod glass;
mod input;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as sys;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as sys;
mod main_menu_export;
mod popup;
pub(crate) mod timer;
mod window;
mod window_style;

#[cfg(all(feature = "shell", target_os = "linux"))]
pub(crate) use linux::extension;
#[cfg(target_os = "linux")]
pub(crate) use linux::{runtime, surface};
pub use sys::ObjectId;
pub(crate) use sys::{
    MOST_CHROMES, SYSTEM_MOMENTUM, clipboard, instance, keychain, notifications, panels, pasteboard,
};

#[cfg(target_os = "macos")]
pub(crate) use ::calloop;
#[cfg(target_os = "linux")]
pub(crate) use sctk::reexports::calloop;

// CoreText sets each face's optical size and `trak` tracking from the point size; Linux keeps the face as it is.
pub(crate) const OPTICAL_SIZING: bool = cfg!(target_os = "macos");

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
    WindowStatus, WindowStyle, run_window,
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
