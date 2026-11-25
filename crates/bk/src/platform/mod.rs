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

pub(crate) use app::KitSender;
#[cfg(all(feature = "shell", target_os = "linux"))]
pub(crate) use linux::extension;
#[cfg(target_os = "linux")]
pub(crate) use linux::{runtime, surface};
pub use sys::ObjectId;
pub(crate) use sys::{
    MODIFIER_GLYPHS, MOST_CHROMES, NATIVE_LIGHTS, OPTICAL_SIZING, SYSTEM_MOMENTUM, app_folders,
    appearance, clipboard, instance, keychain, notifications, panels, pasteboard,
    preferred_languages,
};

#[cfg(target_os = "macos")]
pub(crate) use ::calloop;
#[cfg(target_os = "linux")]
pub(crate) use sctk::reexports::calloop;

pub use input::{inserts_text, modifier_flags};
pub use main_menu_export::MainMenuAddress;
pub use popup::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
};
pub use timer::{Timer, Timers, WakeHandle};

pub(crate) enum Sent {
    User(freya_core::integration::UserEvent),
    Kit(Box<dyn std::any::Any>),
}

pub(crate) struct EditShortcut {
    pub(crate) field: freya_core::prelude::AccessibilityId,
    pub(crate) key: String,
}
pub use window::{
    DecorationResponse, PanelBehavior, WindowControls, WindowEdge, WindowOptions,
    WindowResizability, WindowStatus, WindowStyle, WindowToolbarFullScreenVisibility, run_window,
};
pub(crate) use window::{app_name, open_utility_window, window_activated};
pub(crate) use window_style::Knob;
pub use window_style::{Bar, PopoverShape};

pub(crate) use glass::{GlassOutbox, GlassPanels, GlassSupport, KnobOutbox, KnobSupport, Knobs};
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
