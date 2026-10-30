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

#![allow(unused_imports)]

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

use activation::ActivationCallback;
use app::AppInstance;
use egl::{Graphics, GraphicsShared};
use input::seat::{RepeatState, cursor_shape_from_icon};
use surface::{SurfaceRole, SurfaceWindow, physical_point};
use timer::{PingWake, TimerQueue, Wake};
use toplevels::ToplevelEntry;
use window::WindowCommand;

pub(crate) use popup::PopupCommand;
use runtime::{Runtime, WindowContext, drain_user_events, insert_ping, make_sender};

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    ffi::c_void,
    os::fd::AsFd,
    pin::pin,
    rc::Rc,
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};

use accesskit::Role;
use freya_clipboard::copypasta::ClipboardProvider;
use freya_components::{cache::AssetCacher, integration::integration};
use freya_core::{
    accessibility::tree::{ACCESSIBILITY_ROOT_ID, AccessibilityTree},
    integration::*,
    prelude::{AccessibilityFocusStrategy, Color, CursorIcon, State},
};
use futures_util::task::ArcWake;
use ragnarok::{EventsExecutorRunner, EventsMeasurerRunner, NodesState};
use sctk::reexports::client::event_created_child;
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{
    Anchor as SctkAnchorPos, ConstraintAdjustment as SctkConstraint, Gravity as SctkGravity,
};
use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_session_lock, delegate_touch, delegate_xdg_popup, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            self, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction,
            generic::Generic,
            ping::{Ping, PingSource},
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, Dispatch, Proxy, QueueHandle, delegate_dispatch,
            globals::registry_queue_init,
            protocol::{
                wl_callback::WlCallback, wl_keyboard, wl_output, wl_pointer, wl_seat,
                wl_surface::WlSurface, wl_touch,
            },
        },
        protocols::ext::foreign_toplevel_list::v1::client::{
            ext_foreign_toplevel_handle_v1::{Event as FtlHandleEvent, ExtForeignToplevelHandleV1},
            ext_foreign_toplevel_list_v1::{Event as FtlListEvent, ExtForeignToplevelListV1},
        },
        protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::{
            Shape as CursorShape, WpCursorShapeDeviceV1,
        },
        protocols::xdg::activation::v1::client::{
            xdg_activation_token_v1::{self, XdgActivationTokenV1},
            xdg_activation_v1::XdgActivationV1,
        },
        protocols::xdg::shell::client::xdg_toplevel,
        protocols_wlr::{
            foreign_toplevel::v1::client::{
                zwlr_foreign_toplevel_handle_v1::{
                    Event as ZwlrForeignToplevelHandleV1Event,
                    State as ZwlrForeignToplevelHandleV1State, ZwlrForeignToplevelHandleV1,
                },
                zwlr_foreign_toplevel_manager_v1::{
                    Event as MgmtListEvent, ZwlrForeignToplevelManagerV1,
                },
            },
            layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Modifiers as SctkModifiers, RepeatInfo},
        pointer::{PointerEvent, PointerHandler, cursor_shape::CursorShapeManager},
        touch::TouchHandler,
    },
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockState, SessionLockSurface,
        SessionLockSurfaceConfigure,
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor as SctkAnchor, KeyboardInteractivity as SctkKeyboardInteractivity,
            Layer as SctkLayer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
            LayerSurfaceData,
        },
        xdg::{
            XdgPositioner, XdgShell, XdgSurface,
            popup::{Popup, PopupConfigure, PopupHandler},
            window::{DecorationMode, Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
};
use std::task::Waker;
use torin::prelude::{CursorPoint, Size2D};
use vow_appearance::{Resolved, load_appearance, load_dominant, resolve, watch::AppearanceWatch};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{Event as FractionalScaleEvent, WpFractionalScaleV1},
};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};
use wayland_protocols_plasma::appmenu::client::{
    org_kde_kwin_appmenu::OrgKdeKwinAppmenu, org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager,
};

pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
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
    #[error("Skia could not create its GL context")]
    Skia,
    #[error(transparent)]
    Wayland(#[from] sctk::reexports::client::ConnectError),
    #[error(transparent)]
    Registry(#[from] sctk::reexports::client::globals::GlobalError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
