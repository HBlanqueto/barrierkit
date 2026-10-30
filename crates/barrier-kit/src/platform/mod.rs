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

mod app;
mod app_menu_export;
mod egl;
mod infinity;
mod input;
mod popup;
mod surface;
mod timer;
mod window;

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

use app::AppInstance;
use egl::{Graphics, GraphicsShared};
use input::seat::{RepeatState, cursor_shape_from_icon};
use surface::{SurfaceRole, SurfaceWindow, physical_point};
use timer::{PingWake, TimerQueue, Wake};
use window::WindowCommand;

pub(crate) use popup::PopupCommand;

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

/// A layer of `zwlr_layer_shell_v1`, without exposing SCTK in the public API.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

/// Which edges a layer surface is anchored to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Anchors {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Anchors {
    pub const ALL: Self = Self {
        top: true,
        bottom: true,
        left: true,
        right: true,
    };
    pub const TOP: Self = Self {
        top: true,
        left: true,
        right: true,
        bottom: false,
    };
    pub const BOTTOM: Self = Self {
        bottom: true,
        left: true,
        right: true,
        top: false,
    };
}

/// How a layer surface wants the keyboard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    #[default]
    None,
    Exclusive,
    OnDemand,
}

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

/// One running toplevel, as the dock lists it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToplevelInfo {
    pub title: String,
    pub app_id: String,
    pub active: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub restorable: bool,
    /// The app's dbusmenu address, when it announced one.
    pub app_menu: Option<AppMenuAddress>,
    /// The GTK 3/4 D-Bus menu address, when announced via `gtk_shell1`.
    pub gtk_menus: Option<GtkMenusAddress>,
}

/// D-Bus address of a GTK global menu (ADR 0045).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GtkMenusAddress {
    pub unique_bus_name: String,
    pub application_id: String,
    pub menubar_path: String,
    pub app_menu_path: String,
    pub application_object_path: String,
    pub window_object_path: String,
}

impl GtkMenusAddress {
    /// Returns a `GtkMenusAddress` only when `unique_bus_name` is non-empty and
    /// at least one of `menubar_path` / `app_menu_path` is non-empty.
    pub(crate) fn new(
        unique_bus_name: String,
        application_id: String,
        menubar_path: String,
        app_menu_path: String,
        application_object_path: String,
        window_object_path: String,
    ) -> Option<Self> {
        if unique_bus_name.is_empty() || (menubar_path.is_empty() && app_menu_path.is_empty()) {
            return None;
        }
        Some(Self {
            unique_bus_name,
            application_id,
            menubar_path,
            app_menu_path,
            application_object_path,
            window_object_path,
        })
    }
}

/// A change that only touches one toplevel's metadata. Kept pure so the
/// interleaved-event ordering can be tested without Wayland.
#[derive(Clone, Debug, PartialEq, Eq)]
enum InfoEvent {
    Title(String),
    AppId(String),
    State {
        active: bool,
        maximized: bool,
        minimized: bool,
    },
    AppMenu(Option<AppMenuAddress>),
    GtkMenus(Option<GtkMenusAddress>),
    Closed,
}

fn apply_info_event(infos: &mut Vec<ToplevelInfo>, index: usize, event: InfoEvent) {
    let Some(info) = infos.get_mut(index) else {
        return;
    };
    match event {
        InfoEvent::Title(title) => info.title = title,
        InfoEvent::AppId(app_id) => info.app_id = app_id,
        InfoEvent::State {
            active,
            maximized,
            minimized,
        } => {
            info.active = active;
            info.maximized = maximized;
            info.minimized = minimized;
        }
        InfoEvent::AppMenu(app_menu) => info.app_menu = app_menu,
        InfoEvent::GtkMenus(gtk_menus) => info.gtk_menus = gtk_menus,
        InfoEvent::Closed => {
            infos.remove(index);
        }
    }
}

#[derive(Clone)]
struct ToplevelEntry {
    list: Option<ExtForeignToplevelHandleV1>,
    mgmt: Option<ZwlrForeignToplevelHandleV1>,
    restorable: bool,
    info: ToplevelInfo,
}

/// The running toplevels, provided as a root context. The dock reads
/// [`Toplevels::list`] and calls [`Toplevels::activate`] on a click.
type Activate = Rc<RefCell<Option<Rc<dyn Fn(usize)>>>>;
type Maximize = Rc<RefCell<Option<Rc<dyn Fn(usize, bool)>>>>;
type Arrange = Rc<RefCell<Option<Rc<dyn Fn(usize, InfinityLayout)>>>>;
type ArrangePair = Rc<RefCell<Option<Rc<dyn Fn(InfinityPairLayout)>>>>;
type ArrangeQuarters = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

#[derive(Clone)]
pub struct Toplevels {
    pub list: State<Vec<ToplevelInfo>>,
    activate: Activate,
    close: Activate,
    minimize: Activate,
    maximize: Maximize,
    unminimize: Activate,
    arrange: Arrange,
    arrange_pair: ArrangePair,
    arrange_quarters: ArrangeQuarters,
    restore: Activate,
    set_fullscreen: Maximize,
}

impl Toplevels {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn activate(&self, index: usize) {
        if let Some(f) = self.activate.borrow().as_ref() {
            f(index);
        }
    }

    pub fn close(&self, index: usize) {
        if let Some(f) = self.close.borrow().as_ref() {
            f(index);
        }
    }

    pub fn minimize(&self, index: usize) {
        if let Some(f) = self.minimize.borrow().as_ref() {
            f(index);
        }
    }

    pub fn unset_minimized(&self, index: usize) {
        if let Some(f) = self.unminimize.borrow().as_ref() {
            f(index);
        }
    }

    pub fn set_maximized(&self, index: usize, maximized: bool) {
        if let Some(f) = self.maximize.borrow().as_ref() {
            f(index, maximized);
        }
    }

    /// Minimizes every toplevel in the list.
    pub fn minimize_all(&self) {
        let len = self.list.read().len();
        for index in 0..len {
            self.minimize(index);
        }
    }

    /// Arranges the toplevel at `index` in the requested Infinity layout.
    pub fn arrange(&self, index: usize, layout: InfinityLayout) {
        if let Some(f) = self.arrange.borrow().as_ref() {
            f(index, layout);
        }
    }

    /// Tiles the active and previous toplevels as a pair.
    pub fn arrange_pair(&self, layout: InfinityPairLayout) {
        if let Some(f) = self.arrange_pair.borrow().as_ref() {
            f(layout);
        }
    }

    /// Tiles the four most recent toplevels in quarters.
    pub fn arrange_quarters(&self) {
        if let Some(f) = self.arrange_quarters.borrow().as_ref() {
            f();
        }
    }

    /// Restores the toplevel at `index` to its pre-arrange rectangle.
    pub fn restore(&self, index: usize) {
        if let Some(f) = self.restore.borrow().as_ref() {
            f(index);
        }
    }

    /// Sets or unsets fullscreen on the toplevel at `index`.
    pub fn set_fullscreen(&self, index: usize, fullscreen: bool) {
        if let Some(f) = self.set_fullscreen.borrow().as_ref() {
            f(index, fullscreen);
        }
    }
}

/// Unlocks the session from the lock app. Provided only to lock surfaces.
#[derive(Clone)]
pub struct SessionUnlock {
    unlock: Rc<dyn Fn()>,
}

impl SessionUnlock {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn unlock(&self) {
        (self.unlock)();
    }
}

type ActivationCallback = Box<dyn Fn(Option<String>)>;
type ActivationQueue = Rc<RefCell<Vec<ActivationCallback>>>;

/// Requests an `xdg-activation-v1` token for the next launched app.
///
/// The compositor only accepts a token created by the client that currently
/// has keyboard focus and that carries the last key press serial. The runner
/// owns both, so the app asks here and receives the token (or `None` when the
/// protocol is unavailable) in a callback.
#[derive(Clone)]
pub struct Activation {
    queue: ActivationQueue,
    ping: Ping,
}

impl Activation {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    /// Queues one token request. The callback runs on the Wayland loop with the
    /// token string, or with `None` if the compositor does not support it.
    pub fn request(&self, callback: impl Fn(Option<String>) + 'static) {
        self.queue.borrow_mut().push(Box::new(callback));
        self.ping.ping();
    }
}

pub(super) struct Runtime {
    registry: RegistryState,
    compositor: CompositorState,
    output: OutputState,
    layer_shell: Option<LayerShell>,
    xdg_shell: Option<XdgShell>,
    session_lock: Option<SessionLockState>,
    active_lock: Option<SessionLock>,
    surfaces: Vec<SurfaceWindow>,
    graphics: Rc<GraphicsShared>,
    seat: SeatState,
    pointer: Option<wl_pointer::WlPointer>,
    cursor_shape: Option<CursorShapeManager>,
    cursor_shape_device: Option<WpCursorShapeDeviceV1>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    touch: Option<wl_touch::WlTouch>,
    keyboard_focus: Option<WlSurface>,
    repeat: Option<RepeatState>,
    repeat_info: Option<RepeatInfo>,
    repeat_generation: u64,
    modifiers: SctkModifiers,
    loop_signal: Option<LoopSignal>,
    app: AppComponent,
    spec: SurfaceSpec,
    needs_exit: bool,
    request_redraw_all: bool,
    queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
    ping: Ping,
    ping_source: Option<PingSource>,
    wake: Wake,
    wake_source: Option<PingSource>,
    qh: QueueHandle<Runtime>,
    _foreign_list: Option<ExtForeignToplevelListV1>,
    _foreign_mgmt: Option<ZwlrForeignToplevelManagerV1>,
    _infinity_manager: Rc<RefCell<Option<InfinityWindowManagerV1>>>,
    appmenu_manager: Option<OrgKdeKwinAppmenuManager>,
    fractional_manager: Option<WpFractionalScaleManagerV1>,
    viewporter: Option<WpViewporter>,
    toplevel_entries: Rc<RefCell<Vec<ToplevelEntry>>>,
    toplevels: Toplevels,
    session_unlock: Option<SessionUnlock>,
    session_locked: bool,
    timers: TimerQueue,
    activation: Activation,
    activation_global: Option<XdgActivationV1>,
    activation_callbacks: Vec<Option<ActivationCallback>>,
    activation_seat: Option<wl_seat::WlSeat>,
    last_input_serial: Rc<Cell<u32>>,
    last_pointer_serial: Cell<u32>,
    touch_surfaces: std::collections::HashMap<i32, ObjectId>,
    popup_commands: Rc<RefCell<Vec<PopupCommand>>>,
    popup_next_id: Rc<Cell<u64>>,
    surface_count: Rc<Cell<usize>>,
    window_contexts: Vec<WindowContext>,
    decoration_manager: bool,
    decoration_warned: bool,
    appearance_watch: Option<AppearanceWatch>,
    resolved: State<Resolved>,
    /// Addresses delivered asynchronously by the D-Bus thread, waiting for the
    /// UI loop to call `org_kde_kwin_appmenu.set_address`.
    pending_menu_addresses: Arc<Mutex<Vec<(ObjectId, AppMenuAddress)>>>,
}

#[derive(Clone)]
struct WindowContext {
    controls: WindowControls,
    queue: Rc<RefCell<Vec<WindowCommand>>>,
}

fn make_sender(
    queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
    ping: Ping,
    owner: ObjectId,
) -> Rc<dyn Fn(UserEvent)> {
    Rc::new(move |event| {
        queue.borrow_mut().push((owner.clone(), event));
        ping.ping();
    })
}

fn anchors_to_sctk(a: Anchors) -> SctkAnchor {
    let mut out = SctkAnchor::empty();
    if a.top {
        out |= SctkAnchor::TOP;
    }
    if a.bottom {
        out |= SctkAnchor::BOTTOM;
    }
    if a.left {
        out |= SctkAnchor::LEFT;
    }
    if a.right {
        out |= SctkAnchor::RIGHT;
    }
    out
}

fn drain_user_events(
    events: &Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
) -> Vec<(ObjectId, UserEvent)> {
    std::mem::take(&mut *events.borrow_mut())
}

/// Runs a layer-shell surface with a Freya app until the compositor closes it.
pub fn run_layer(app: AppComponent, spec: SurfaceSpec) -> Result<(), RunError> {
    let (conn, event_queue, qh, mut shell) = Runtime::connect(app, spec)?;
    shell.qh = qh.clone();
    let layer_shell = shell.layer_shell.as_ref().ok_or_else(|| {
        tracing::info!(
            "zwlr_layer_shell_v1 no disponible; las superficies Layer no funcionan aquí"
        );
        RunError::MissingGlobal("zwlr_layer_shell_v1")
    })?;

    let (namespace, layer, anchors, exclusive_zone, keyboard, size) = match &shell.spec {
        SurfaceSpec::Layer {
            namespace,
            layer,
            anchors,
            exclusive_zone,
            keyboard,
            size,
        } => (
            namespace.clone(),
            *layer,
            *anchors,
            *exclusive_zone,
            *keyboard,
            *size,
        ),
        SurfaceSpec::Lock | SurfaceSpec::Window(_) => {
            return Err(RunError::MissingGlobal("layer surface"));
        }
    };

    let surface = shell.compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        match layer {
            Layer::Background => SctkLayer::Background,
            Layer::Bottom => SctkLayer::Bottom,
            Layer::Top => SctkLayer::Top,
            Layer::Overlay => SctkLayer::Overlay,
        },
        Some(&namespace),
        None,
    );
    layer_surface.set_anchor(anchors_to_sctk(anchors));
    layer_surface.set_exclusive_zone(exclusive_zone);
    layer_surface.set_keyboard_interactivity(match keyboard {
        KeyboardInteractivity::None => SctkKeyboardInteractivity::None,
        KeyboardInteractivity::Exclusive => SctkKeyboardInteractivity::Exclusive,
        KeyboardInteractivity::OnDemand => SctkKeyboardInteractivity::OnDemand,
    });
    if let Some((w, h)) = size {
        layer_surface.set_size(w, h);
    }
    surface.commit();

    let waker = futures_util::task::waker(Arc::new(PingWake(shell.ping.clone())));
    let sender = make_sender(shell.queue.clone(), shell.ping.clone(), surface.id());
    let popups = Popups {
        queue: shell.popup_commands.clone(),
        ping: shell.ping.clone(),
        next_id: shell.popup_next_id.clone(),
        last_serial: shell.last_input_serial.clone(),
        owner: surface.id(),
        current: None,
        surface_count: shell.surface_count.clone(),
        opens_left: Rc::new(Cell::new(false)),
    };
    let app_instance = AppInstance::new(
        shell.app.clone(),
        1.0,
        size.unwrap_or((1, 1)),
        sender,
        waker,
        Some(shell.toplevels.clone()),
        None,
        shell.activation.clone(),
        None,
        Timers {
            queue: shell.timers.clone(),
            ping: shell.ping.clone(),
            wake: shell.wake.clone(),
        },
        popups,
        None,
        shell.graphics.font_collection.clone(),
        shell.graphics.font_manager.clone(),
        None,
        None,
        shell.resolved,
    );
    let (fractional, viewport) = shell.create_fractional(&surface);
    shell.surfaces.push(SurfaceWindow {
        surface: surface.clone(),
        role: SurfaceRole::Layer(layer_surface),
        popup: None,
        shared: shell.graphics.clone(),
        graphics: None,
        app: app_instance,
        logical_size: size.unwrap_or((0, 0)),
        buffer_size: (0, 0),
        scale: 1.0,
        output: None,
        needs_redraw: true,
        frame_pending: false,
        draw_count: 0,
        configure_count: 0,
        pending_configure: None,
        appmenu: None,
        appmenu_address: None,
        menu_handle: None,
        fractional,
        viewport,
    });
    shell.surface_count.set(shell.surfaces.len());

    let mut event_loop: EventLoop<Runtime> =
        EventLoop::try_new().map_err(|_| RunError::Egl("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut shell)?;
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::Egl("inserting the Wayland source"))?;
    let signal = event_loop.get_signal();
    shell.loop_signal = Some(signal.clone());
    event_loop
        .run(None, &mut shell, |shell| {
            if shell.needs_exit
                && let Some(signal) = &shell.loop_signal
            {
                signal.stop();
            }
        })
        .map_err(|_| RunError::Egl("calloop event loop failed"))?;
    Ok(())
}

/// Runs the session lock: one Freya surface per output.
pub fn run_lock(app: AppComponent) -> Result<(), RunError> {
    let (conn, event_queue, qh, mut shell) = Runtime::connect(app, SurfaceSpec::Lock)?;
    shell.qh = qh.clone();
    let session_lock_state = shell
        .session_lock
        .as_ref()
        .ok_or(RunError::MissingGlobal("ext_session_lock_manager_v1"))?;
    let lock = session_lock_state
        .lock(&qh)
        .map_err(|_| RunError::MissingGlobal("ext_session_lock_manager_v1 (lock refused)"))?;
    shell.active_lock = Some(lock);

    let mut event_loop: EventLoop<Runtime> =
        EventLoop::try_new().map_err(|_| RunError::Egl("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut shell)?;
    let flush_conn = conn.clone();
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::Egl("inserting the Wayland source"))?;
    let signal = event_loop.get_signal();
    shell.loop_signal = Some(signal.clone());
    if let Some(lock) = shell.active_lock.clone() {
        shell.session_unlock = Some(SessionUnlock {
            unlock: Rc::new(move || {
                lock.unlock();
                let _ = flush_conn.flush();
                signal.stop();
            }),
        });
    }
    event_loop
        .run(None, &mut shell, |shell| {
            if shell.needs_exit
                && let Some(signal) = &shell.loop_signal
            {
                signal.stop();
            }
        })
        .map_err(|_| RunError::Egl("calloop event loop failed"))?;
    Ok(())
}

fn insert_ping(event_loop: &mut EventLoop<Runtime>, shell: &mut Runtime) -> Result<(), RunError> {
    let source = shell
        .ping_source
        .take()
        .ok_or(RunError::Egl("the Freya ping source is already installed"))?;
    let handle = event_loop.handle();
    event_loop
        .handle()
        .insert_source(source, move |_, _, shell: &mut Runtime| {
            for (surface_id, event) in drain_user_events(&shell.queue) {
                match event {
                    UserEvent::RequestRedraw => {
                        shell.request_redraw_all = true;
                    }
                    UserEvent::SetCursorIcon(icon) => {
                        if let Some(device) = &shell.cursor_shape_device {
                            device.set_shape(
                                shell.last_pointer_serial.get(),
                                cursor_shape_from_icon(icon),
                            );
                        }
                    }
                    UserEvent::FocusAccessibilityNode(strategy) => {
                        if let Some(index) = shell
                            .surfaces
                            .iter()
                            .position(|s| s.surface.id() == surface_id)
                        {
                            shell.surfaces[index].app.requested_focus = Some(strategy);
                            shell.surfaces[index].needs_redraw = true;
                        }
                    }
                    UserEvent::Erased(_) => {}
                }
            }
            shell.poll_all();
            shell.process_popup_commands();
            shell.install_timers(&handle);
            shell.install_repeat(&handle);
            shell.redraw_all();
        })
        .map_err(|_| RunError::Egl("inserting the Freya ping source"))?;

    let wake_source = shell
        .wake_source
        .take()
        .ok_or(RunError::Egl("the wake ping source is already installed"))?;
    let callbacks = shell.wake.callbacks.clone();
    event_loop
        .handle()
        .insert_source(wake_source, move |_, _, shell: &mut Runtime| {
            let callbacks: Vec<Rc<dyn Fn()>> = callbacks.borrow().clone();
            for callback in callbacks {
                callback();
            }
            shell.process_pending_menu_addresses();
            shell.process_menu_actions();
            shell.poll_all();
            shell.redraw_all();
        })
        .map_err(|_| RunError::Egl("inserting the wake ping source"))?;

    // Watch appearance.domain and wallpaper.state; when they change, resolve
    // the theme again and update the process-wide reactive state so every
    // surface that reads theme::resolved() re-executes.
    let watch = AppearanceWatch::new().map_err(|error| {
        RunError::Io(std::io::Error::other(format!(
            "creating the appearance watcher: {error}"
        )))
    })?;
    let watch_fd = watch.as_fd().try_clone_to_owned().map_err(RunError::Io)?;
    shell.appearance_watch = Some(watch);
    event_loop
        .handle()
        .insert_source(
            Generic::new(watch_fd, Interest::READ, Mode::Level),
            |_, _, shell: &mut Runtime| {
                if let Some(watch) = shell.appearance_watch.as_ref()
                    && let Ok(events) = watch.drain()
                    && !events.is_empty()
                {
                    let resolved = resolve(&load_appearance(), load_dominant());
                    if *shell.resolved.peek() != resolved {
                        shell.resolved.set(resolved);
                        shell.request_redraw_all = true;
                        shell.ping.ping();
                    }
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|_| RunError::Egl("inserting the appearance watch source"))?;
    Ok(())
}

impl Runtime {
    fn connect(
        app: AppComponent,
        spec: SurfaceSpec,
    ) -> Result<
        (
            Connection,
            sctk::reexports::client::EventQueue<Runtime>,
            QueueHandle<Runtime>,
            Runtime,
        ),
        RunError,
    > {
        let conn = Connection::connect_to_env().map_err(|_| RunError::NoDisplay)?;
        let (globals, event_queue) = registry_queue_init::<Runtime>(&conn)?;
        let qh = event_queue.handle();
        let queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>> = Rc::new(RefCell::new(Vec::new()));
        let (ping, ping_source) =
            calloop::ping::make_ping().map_err(|_| RunError::Egl("calloop ping failed"))?;
        let (wake_ping, wake_source) =
            calloop::ping::make_ping().map_err(|_| RunError::Egl("calloop wake ping failed"))?;
        let resolved = State::create_global(resolve(&load_appearance(), load_dominant()));
        let toplevel_entries: Rc<RefCell<Vec<ToplevelEntry>>> = Rc::new(RefCell::new(Vec::new()));
        let close_entries = toplevel_entries.clone();
        let minimize_entries = toplevel_entries.clone();
        let maximize_entries = toplevel_entries.clone();
        let unminimize_entries = toplevel_entries.clone();
        let activate_entries = toplevel_entries.clone();
        let arrange_entries = toplevel_entries.clone();
        let restore_entries = toplevel_entries.clone();
        let fullscreen_entries = toplevel_entries.clone();
        let infinity_manager = Rc::new(RefCell::new(None::<InfinityWindowManagerV1>));
        let infinity_manager_arrange = infinity_manager.clone();
        let infinity_manager_pair = infinity_manager.clone();
        let infinity_manager_quarters = infinity_manager.clone();
        let infinity_manager_restore = infinity_manager.clone();
        let infinity_manager_fullscreen = infinity_manager.clone();
        let toplevels = Toplevels {
            list: State::create_global(Vec::new()),
            activate: Rc::new(RefCell::new(None)),
            close: Rc::new(RefCell::new(Some(Rc::new(move |index| {
                if let Some(entry) = close_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    mgmt.close();
                }
            })))),
            minimize: Rc::new(RefCell::new(Some(Rc::new(move |index| {
                if let Some(entry) = minimize_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    mgmt.set_minimized();
                }
            })))),
            maximize: Rc::new(RefCell::new(Some(Rc::new(move |index, maximized| {
                if let Some(entry) = maximize_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    if maximized {
                        mgmt.set_maximized();
                    } else {
                        mgmt.unset_maximized();
                    }
                }
            })))),
            unminimize: Rc::new(RefCell::new(Some(Rc::new(move |index| {
                if let Some(entry) = unminimize_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    mgmt.unset_minimized();
                }
            })))),
            arrange: Rc::new(RefCell::new(Some(Rc::new(move |index, layout| {
                if let Some(entry) = arrange_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                    && let Some(manager) = infinity_manager_arrange.borrow().as_ref()
                {
                    manager.arrange(mgmt, layout);
                }
            })))),
            arrange_pair: Rc::new(RefCell::new(Some(Rc::new(move |layout| {
                if let Some(manager) = infinity_manager_pair.borrow().as_ref() {
                    manager.arrange_pair(layout);
                }
            })))),
            arrange_quarters: Rc::new(RefCell::new(Some(Rc::new(move || {
                if let Some(manager) = infinity_manager_quarters.borrow().as_ref() {
                    manager.arrange_quarters();
                }
            })))),
            restore: Rc::new(RefCell::new(Some(Rc::new(move |index| {
                if let Some(entry) = restore_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                    && let Some(manager) = infinity_manager_restore.borrow().as_ref()
                {
                    manager.restore(mgmt);
                }
            })))),
            set_fullscreen: Rc::new(RefCell::new(Some(Rc::new(move |index, fullscreen| {
                if let Some(entry) = fullscreen_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                    && let Some(manager) = infinity_manager_fullscreen.borrow().as_ref()
                {
                    manager.set_fullscreen(mgmt, fullscreen as u32);
                }
            })))),
        };
        let foreign_list = globals
            .bind::<ExtForeignToplevelListV1, _, _>(&qh, 1..=1, ())
            .ok();
        if foreign_list.is_none() {
            tracing::info!("ext_foreign_toplevel_list_v1 no disponible; sigo sin él");
        }
        let foreign_mgmt = globals
            .bind::<ZwlrForeignToplevelManagerV1, _, _>(&qh, 3..=3, ())
            .ok();
        if foreign_mgmt.is_none() {
            tracing::info!("zwlr_foreign_toplevel_manager_v1 no disponible; sigo sin él");
        }
        let infinity_manager_bound = globals
            .bind::<InfinityWindowManagerV1, _, _>(&qh, 1..=3, ())
            .ok();
        if infinity_manager_bound.is_none() {
            tracing::info!("infinity_window_manager_v1 no disponible; sigo sin él");
        }
        *infinity_manager.borrow_mut() = infinity_manager_bound;
        let activation_global = globals.bind::<XdgActivationV1, _, _>(&qh, 1..=1, ()).ok();
        let appmenu_manager = globals
            .bind::<OrgKdeKwinAppmenuManager, _, _>(&qh, 1..=1, ())
            .ok();
        if appmenu_manager.is_none() {
            tracing::info!("org_kde_kwin_appmenu_manager no disponible; sigo sin él");
        }
        let fractional_manager = globals
            .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        if fractional_manager.is_none() {
            tracing::info!(
                "wp_fractional_scale_manager_v1 no disponible; sigo con escalas enteras"
            );
        }
        let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
        if viewporter.is_none() {
            tracing::info!("wp_viewporter no disponible; sigo con escalas enteras");
        }
        // `SeatState::new` binds the seats that already exist without calling
        // `SeatHandler::new_seat`, so remember one here for activation and
        // popup grabs. Hotplugged seats still arrive through `new_seat`.
        let seat = SeatState::new(&globals, &qh);
        let activation_seat = seat.seats().next();
        // `SeatState::new` binds existing seats without calling `new_seat`, so
        // install the activation command for the seat already present.
        if let Some(seat) = activation_seat.clone() {
            *toplevels.activate.borrow_mut() = Some(Rc::new(move |index| {
                if let Some(entry) = activate_entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    mgmt.activate(&seat);
                }
            }));
        }
        let shell = Runtime {
            registry: RegistryState::new(&globals),
            compositor: CompositorState::bind(&globals, &qh)
                .map_err(|_| RunError::MissingGlobal("wl_compositor"))?,
            output: OutputState::new(&globals, &qh),
            layer_shell: LayerShell::bind(&globals, &qh).ok(),
            xdg_shell: XdgShell::bind(&globals, &qh).ok(),
            session_lock: Some(SessionLockState::new(&globals, &qh)),
            active_lock: None,
            surfaces: Vec::new(),
            graphics: {
                let display_ptr = conn.display().id().as_ptr().cast::<c_void>();
                Rc::new(GraphicsShared::new(display_ptr)?)
            },
            seat,
            pointer: None,
            cursor_shape: {
                let manager = CursorShapeManager::bind(&globals, &qh).ok();
                if manager.is_none() {
                    tracing::info!(
                        "wp_cursor_shape_manager_v1 no disponible; uso cursor por defecto"
                    );
                }
                manager
            },
            cursor_shape_device: None,
            keyboard: None,
            touch: None,
            keyboard_focus: None,
            repeat: None,
            repeat_info: None,
            repeat_generation: 0,
            modifiers: SctkModifiers::default(),
            loop_signal: None,
            app,
            spec,
            needs_exit: false,
            request_redraw_all: false,
            queue,
            ping: ping.clone(),
            ping_source: Some(ping_source),
            wake: Wake {
                ping: wake_ping,
                callbacks: Rc::new(RefCell::new(Vec::new())),
            },
            wake_source: Some(wake_source),
            qh: qh.clone(),
            _foreign_list: foreign_list,
            _foreign_mgmt: foreign_mgmt,
            _infinity_manager: infinity_manager,
            appmenu_manager,
            fractional_manager,
            viewporter,
            toplevel_entries,
            toplevels,
            session_unlock: None,
            session_locked: false,
            timers: Rc::new(RefCell::new(Vec::new())),
            activation: Activation {
                queue: Rc::new(RefCell::new(Vec::new())),
                ping: ping.clone(),
            },
            activation_global,
            activation_callbacks: Vec::new(),
            activation_seat,
            last_input_serial: Rc::new(Cell::new(0)),
            last_pointer_serial: Cell::new(0),
            touch_surfaces: std::collections::HashMap::new(),
            popup_commands: Rc::new(RefCell::new(Vec::new())),
            popup_next_id: Rc::new(Cell::new(1)),
            surface_count: Rc::new(Cell::new(0)),
            window_contexts: Vec::new(),
            decoration_manager: globals.contents().with_list(|list| {
                list.iter()
                    .any(|global| global.interface == "zxdg_decoration_manager_v1")
            }),
            decoration_warned: false,
            appearance_watch: None,
            resolved,
            pending_menu_addresses: Arc::new(Mutex::new(Vec::new())),
        };
        Ok((conn, event_queue, qh, shell))
    }

    pub(super) fn surface_for(&self, surface: &WlSurface) -> Option<usize> {
        self.surfaces.iter().position(|s| s.surface == *surface)
    }

    pub(super) fn surface_index_by_id(&self, surface_id: &ObjectId) -> Option<usize> {
        self.surfaces
            .iter()
            .position(|s| s.surface.id() == *surface_id)
    }
}

impl Runtime {
    fn create_lock_surface(
        &mut self,
        qh: &QueueHandle<Self>,
        output: &wl_output::WlOutput,
    ) -> Option<usize> {
        if self.surfaces.iter().any(|s| s.output == Some(output.id())) {
            return None;
        }
        let lock = self.active_lock.as_ref()?;
        let surface = self.compositor.create_surface(qh);
        let lock_surface = lock.create_lock_surface(surface.clone(), output, qh);
        let waker = futures_util::task::waker(Arc::new(PingWake(self.ping.clone())));
        let sender = make_sender(self.queue.clone(), self.ping.clone(), surface.id());
        let popups = Popups {
            queue: self.popup_commands.clone(),
            ping: self.ping.clone(),
            next_id: self.popup_next_id.clone(),
            last_serial: self.last_input_serial.clone(),
            owner: surface.id(),
            current: None,
            surface_count: self.surface_count.clone(),
            opens_left: Rc::new(Cell::new(false)),
        };
        let app = AppInstance::new(
            self.app.clone(),
            1.0,
            (1, 1),
            sender,
            waker,
            Some(self.toplevels.clone()),
            self.session_unlock.clone(),
            self.activation.clone(),
            None,
            Timers {
                queue: self.timers.clone(),
                ping: self.ping.clone(),
                wake: self.wake.clone(),
            },
            popups,
            None,
            self.graphics.font_collection.clone(),
            self.graphics.font_manager.clone(),
            None,
            None,
            self.resolved,
        );
        let (fractional, viewport) = self.create_fractional(&surface);
        self.surfaces.push(SurfaceWindow {
            surface,
            role: SurfaceRole::Lock(lock_surface),
            popup: None,
            shared: self.graphics.clone(),
            graphics: None,
            app,
            logical_size: (0, 0),
            buffer_size: (0, 0),
            scale: 1.0,
            output: Some(output.id()),
            needs_redraw: true,
            frame_pending: false,
            draw_count: 0,
            configure_count: 0,
            pending_configure: None,
            appmenu: None,
            appmenu_address: None,
            menu_handle: None,
            fractional,
            viewport,
        });
        self.surface_count.set(self.surfaces.len());
        Some(self.surfaces.len() - 1)
    }

    pub(super) fn poll_all(&mut self) {
        for window in &mut self.surfaces {
            let dirty = window.app.poll();
            if dirty {
                window.app.update();
                window.needs_redraw = true;
            }
            if self.request_redraw_all {
                window.needs_redraw = true;
            }
        }
        self.request_redraw_all = false;
        self.poll_activation();
        self.process_window_commands();
    }

    fn poll_activation(&mut self) {
        let requests = std::mem::take(&mut *self.activation.queue.borrow_mut());
        for callback in requests {
            let (Some(global), Some(seat), Some(window)) = (
                self.activation_global.as_ref(),
                self.activation_seat.as_ref(),
                self.surfaces.first(),
            ) else {
                callback(None);
                continue;
            };
            let serial = self.last_input_serial.get();
            if serial == 0 {
                callback(None);
                continue;
            }
            let id = self.activation_callbacks.len();
            self.activation_callbacks.push(Some(callback));
            let token = global.get_activation_token(&self.qh, id);
            token.set_serial(serial, seat);
            token.set_surface(&window.surface);
            token.commit();
        }
    }

    fn sync_toplevels(&mut self) {
        let infos: Vec<ToplevelInfo> = self
            .toplevel_entries
            .borrow()
            .iter()
            .map(|entry| {
                let mut info = entry.info.clone();
                info.restorable = entry.restorable;
                info
            })
            .collect();
        let mut list = self.toplevels.list;
        if *list.peek() != infos {
            list.set(infos);
            self.request_redraw_all = true;
            self.ping.ping();
        }
    }

    pub(super) fn redraw_all(&mut self) {
        let qh = self.qh.clone();
        for window in &mut self.surfaces {
            if window.needs_redraw {
                // With the compositor now scheduling a repaint for any surface
                // commit that carries damage or asks for a frame callback, we
                // can go back to drawing on the frame callback instead of
                // committing immediately on every user event.
                window.request_redraw(&qh);
            }
        }
    }
}

impl ProvidesRegistryState for Runtime {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    fn runtime_add_global(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: u32,
        _: &str,
        _: u32,
    ) {
    }

    fn runtime_remove_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str) {}
}

impl LayerShellHandler for Runtime {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.needs_exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        layer_surface: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let Some(index) = self.surfaces.iter().position(
            |s| matches!(&s.role, SurfaceRole::Layer(existing) if existing == layer_surface),
        ) else {
            return;
        };
        let scale = self.surfaces[index].scale;
        self.surfaces[index].configure(configure.new_size, scale);
        self.surfaces[index].request_redraw(qh);
    }
}

impl SessionLockHandler for Runtime {
    fn locked(&mut self, _: &Connection, qh: &QueueHandle<Self>, session_lock: SessionLock) {
        self.active_lock = Some(session_lock);
        self.session_locked = true;
        let outputs: Vec<wl_output::WlOutput> = self.output.outputs().collect();
        for output in outputs {
            self.create_lock_surface(qh, &output);
        }
    }

    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        self.session_locked = false;
        eprintln!("malaysia-shell lock: the compositor refused the session lock");
        std::process::exit(1);
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _serial: u32,
    ) {
        let wl_surface = surface.wl_surface().clone();
        let Some(index) = self.surface_for(&wl_surface) else {
            return;
        };
        let scale = self.surfaces[index].scale;
        self.surfaces[index].configure(configure.new_size, scale);
        self.surfaces[index].request_redraw(qh);
    }
}

impl OutputHandler for Runtime {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(&mut self, _: &Connection, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        if self.session_locked
            && matches!(self.spec, SurfaceSpec::Lock)
            && !self.surfaces.iter().any(|s| s.output == Some(output.id()))
        {
            self.create_lock_surface(qh, &output);
        }
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl Dispatch<ExtForeignToplevelListV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        _: &ExtForeignToplevelListV1,
        event: <ExtForeignToplevelListV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let FtlListEvent::Toplevel { toplevel } = event
            && state._foreign_mgmt.is_none()
        {
            state.toplevel_entries.borrow_mut().push(ToplevelEntry {
                list: Some(toplevel),
                mgmt: None,
                restorable: false,
                info: ToplevelInfo::default(),
            });
            state.sync_toplevels();
        }
    }

    event_created_child!(Runtime, ExtForeignToplevelListV1, [
        sctk::reexports::protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::EVT_TOPLEVEL_OPCODE => (ExtForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ExtForeignToplevelHandleV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        handle: &ExtForeignToplevelHandleV1,
        event: <ExtForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if state._foreign_mgmt.is_some() {
            return;
        }
        if matches!(event, FtlHandleEvent::Closed) {
            state.toplevel_entries.borrow_mut().retain(|entry| {
                entry
                    .list
                    .as_ref()
                    .is_none_or(|list| list.id() != handle.id())
            });
            state.sync_toplevels();
            return;
        }
        let mut entries = state.toplevel_entries.borrow_mut();
        let index = entries
            .iter()
            .position(|e| e.list.as_ref().is_some_and(|list| list.id() == handle.id()));
        if let Some(index) = index {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            let info_event = match event {
                FtlHandleEvent::Title { title } => Some(InfoEvent::Title(title)),
                FtlHandleEvent::AppId { app_id } => Some(InfoEvent::AppId(app_id)),
                _ => None,
            };
            if let Some(info_event) = info_event {
                apply_info_event(&mut infos, index, info_event);
                entries[index].info = infos[index].clone();
            }
        }
        drop(entries);
        state.sync_toplevels();
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: <ZwlrForeignToplevelManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let MgmtListEvent::Toplevel { toplevel } = event {
            state.toplevel_entries.borrow_mut().push(ToplevelEntry {
                list: None,
                mgmt: Some(toplevel),
                restorable: false,
                info: ToplevelInfo::default(),
            });
            state.sync_toplevels();
        }
    }

    event_created_child!(Runtime, ZwlrForeignToplevelManagerV1, [
        sctk::reexports::protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: <ZwlrForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let mut entries = state.toplevel_entries.borrow_mut();
        let index = entries
            .iter()
            .position(|e| e.mgmt.as_ref().is_some_and(|mgmt| mgmt.id() == handle.id()));
        let Some(index) = index else {
            return;
        };
        if matches!(event, ZwlrForeignToplevelHandleV1Event::Closed) {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            apply_info_event(&mut infos, index, InfoEvent::Closed);
            entries.remove(index);
            drop(entries);
            state.sync_toplevels();
            return;
        }
        let info_event = match event {
            ZwlrForeignToplevelHandleV1Event::Title { title } => Some(InfoEvent::Title(title)),
            ZwlrForeignToplevelHandleV1Event::AppId { app_id } => Some(InfoEvent::AppId(app_id)),
            ZwlrForeignToplevelHandleV1Event::State { state } => Some(InfoEvent::State {
                active: state.contains(&(ZwlrForeignToplevelHandleV1State::Activated as u8)),
                maximized: state.contains(&(ZwlrForeignToplevelHandleV1State::Maximized as u8)),
                minimized: state.contains(&(ZwlrForeignToplevelHandleV1State::Minimized as u8)),
            }),
            _ => None,
        };
        if let Some(info_event) = info_event {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            apply_info_event(&mut infos, index, info_event);
            entries[index].info = infos[index].clone();
        }
        drop(entries);
        state.sync_toplevels();
    }
}

impl Dispatch<XdgActivationV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &XdgActivationV1,
        _: <XdgActivationV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<XdgActivationTokenV1, usize> for Runtime {
    fn event(
        state: &mut Self,
        _: &XdgActivationTokenV1,
        event: xdg_activation_token_v1::Event,
        id: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_activation_token_v1::Event::Done { token } = event
            && let Some(callback) = state
                .activation_callbacks
                .get_mut(*id)
                .and_then(Option::take)
        {
            callback(Some(token));
        }
    }
}

impl Dispatch<InfinityWindowManagerV1, ()> for Runtime {
    fn event(
        state: &mut Self,
        _: &InfinityWindowManagerV1,
        event: <InfinityWindowManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use infinity::protocol::infinity_window_manager_v1::Event;
        match event {
            Event::Restorable { handle, restorable } => {
                let restorable = restorable != 0;
                for entry in state.toplevel_entries.borrow_mut().iter_mut() {
                    if entry
                        .mgmt
                        .as_ref()
                        .is_some_and(|mgmt| mgmt.id() == handle.id())
                    {
                        entry.restorable = restorable;
                        break;
                    }
                }
            }
            Event::AppMenu {
                toplevel,
                service,
                path,
            } => {
                let address = if service.is_empty() || path.is_empty() {
                    None
                } else {
                    Some(AppMenuAddress { service, path })
                };
                update_toplevel_info(
                    &mut state.toplevel_entries.borrow_mut(),
                    &toplevel,
                    InfoEvent::AppMenu(address),
                );
            }
            Event::GtkMenus {
                toplevel,
                unique_bus_name,
                application_id,
                menubar_path,
                app_menu_path,
                application_object_path,
                window_object_path,
            } => {
                let address = GtkMenusAddress::new(
                    unique_bus_name,
                    application_id,
                    menubar_path,
                    app_menu_path,
                    application_object_path,
                    window_object_path,
                );
                update_toplevel_info(
                    &mut state.toplevel_entries.borrow_mut(),
                    &toplevel,
                    InfoEvent::GtkMenus(address),
                );
            }
            // shortcut: future Infinity events are ignored; add handlers when
            // the compositor starts sending them.
            _ => {}
        }
        state.sync_toplevels();
    }
}

fn update_toplevel_info(
    entries: &mut [ToplevelEntry],
    toplevel: &ZwlrForeignToplevelHandleV1,
    event: InfoEvent,
) {
    if let Some(index) = entries.iter().position(|entry| {
        entry
            .mgmt
            .as_ref()
            .is_some_and(|mgmt| mgmt.id() == toplevel.id())
    }) {
        let mut infos: Vec<ToplevelInfo> = entries.iter().map(|entry| entry.info.clone()).collect();
        apply_info_event(&mut infos, index, event);
        entries[index].info = infos[index].clone();
    }
}

delegate_compositor!(Runtime);
delegate_output!(Runtime);
delegate_registry!(Runtime);
delegate_xdg_shell!(Runtime);
delegate_xdg_window!(Runtime);
delegate_session_lock!(Runtime);
delegate_dispatch!(Runtime: [zwlr_layer_shell_v1::ZwlrLayerShellV1: sctk::globals::GlobalData] => LayerShell);
delegate_dispatch!(Runtime: [zwlr_layer_surface_v1::ZwlrLayerSurfaceV1: LayerSurfaceData] => LayerShell);

#[cfg(test)]
mod tests {
    use super::*;
    use input::seat::{
        RepeatState, cursor_shape_from_icon, dispatch_touch_cancels, find_index_by_id,
        released_repeated_key, should_repeat_key, stop_repeat,
    };
    use sctk::seat::keyboard::Keysym;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use surface::{
        buffer_size, destroy_fractional_objects, fractional_scale_enabled, physical_point,
        scale_from_wire,
    };
    use timer::{Timers, Wake, WakeQueue};
    use window::clamp_toplevel_size_dimension;

    #[test]
    fn wakeup_runs_its_callback_on_the_loop_thread() {
        let (ping, source) = calloop::ping::make_ping().unwrap();
        let callbacks: WakeQueue = Rc::new(RefCell::new(Vec::new()));
        let mut event_loop = EventLoop::<()>::try_new().unwrap();
        let queue = callbacks.clone();
        event_loop
            .handle()
            .insert_source(source, move |_, _, ()| {
                for callback in queue.borrow().clone() {
                    callback();
                }
            })
            .unwrap();
        let timers = Timers {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: ping.clone(),
            wake: Wake { ping, callbacks },
        };
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let handle = timers.wakeup(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        let thread_handle = handle.clone();
        let writer = std::thread::spawn(move || thread_handle.wake());
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while seen.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
            event_loop
                .dispatch(Duration::from_millis(10), &mut ())
                .unwrap();
        }
        writer.join().unwrap();
        assert_eq!(seen.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn toplevel_size_dimension_clamps_to_unbounded() {
        assert_eq!(clamp_toplevel_size_dimension(u32::MAX), 0);
        assert_eq!(clamp_toplevel_size_dimension(i32::MAX as u32 + 1), 0);
        assert_eq!(
            clamp_toplevel_size_dimension(i32::MAX as u32),
            i32::MAX as u32
        );
        assert_eq!(clamp_toplevel_size_dimension(760), 760);
        assert_eq!(clamp_toplevel_size_dimension(0), 0);
    }

    #[test]
    fn scale_from_wire_converts_fractional_scale_values() {
        assert!((scale_from_wire(120) - 1.0).abs() < f64::EPSILON);
        assert!((scale_from_wire(150) - 1.25).abs() < f64::EPSILON);
        assert!((scale_from_wire(180) - 1.5).abs() < f64::EPSILON);
        assert!((scale_from_wire(240) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn buffer_size_rounds_logical_size_at_fractional_scale() {
        assert_eq!(buffer_size((801, 601), 1.25), (1001, 751));
    }

    #[test]
    fn fractional_scale_mode_needs_both_globals() {
        assert!(fractional_scale_enabled(true, true, 1.25));
        assert!(!fractional_scale_enabled(true, false, 1.25));
        assert!(!fractional_scale_enabled(false, true, 1.25));
        assert!(!fractional_scale_enabled(false, false, 1.25));
    }

    #[test]
    fn pointer_and_touch_coordinates_become_physical() {
        assert_eq!(
            physical_point((10.0, 20.0), 1.5),
            CursorPoint::from((15.0, 30.0))
        );
        assert_eq!(
            physical_point((3.0, 4.0), 2.0),
            CursorPoint::from((6.0, 8.0))
        );
        assert_eq!(
            physical_point((10.0, 20.0), 1.25),
            CursorPoint::from((12.5, 25.0))
        );
    }

    #[test]
    fn fractional_scale_hit_test_respects_one_pixel_border() {
        let scale = 1.25;
        let border_physical = 1.0 * scale;
        let inside = physical_point((0.5, 0.5), scale);
        let outside = physical_point((1.5, 1.5), scale);
        assert!(inside.x < border_physical);
        assert!(inside.y < border_physical);
        assert!(outside.x >= border_physical);
        assert!(outside.y >= border_physical);
    }

    #[test]
    fn losing_focus_cancels_repeat_and_invalidates_old_timers() {
        let event = KeyEvent {
            time: 0,
            raw_code: 0,
            keysym: Keysym::new(0),
            utf8: None,
        };
        let mut repeat = Some(RepeatState {
            event,
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: false,
            pending: true,
            generation: 7,
        });
        let mut generation = 7;
        stop_repeat(&mut repeat, &mut generation);
        assert!(repeat.is_none());
        assert_eq!(generation, 8);
    }

    #[test]
    fn modifier_keys_do_not_start_repeat() {
        let shift = KeyEvent {
            time: 0,
            raw_code: 50,
            keysym: Keysym::Shift_L,
            utf8: None,
        };
        assert!(!should_repeat_key(&shift));
    }

    #[test]
    fn regular_keys_start_repeat() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        assert!(should_repeat_key(&a));
    }

    #[test]
    fn releasing_repeated_key_stops_repeat() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let repeat = Some(RepeatState {
            event: a.clone(),
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 1,
        });
        assert!(released_repeated_key(&repeat, &a));
    }

    #[test]
    fn releasing_a_different_key_keeps_repeat_running() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let b = KeyEvent {
            time: 0,
            raw_code: 56,
            keysym: Keysym::b,
            utf8: Some("b".into()),
        };
        let repeat = Some(RepeatState {
            event: b,
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 1,
        });
        assert!(!released_repeated_key(&repeat, &a));
    }

    #[test]
    fn pressing_b_then_releasing_a_keeps_b_repeating() {
        let a = KeyEvent {
            time: 0,
            raw_code: 38,
            keysym: Keysym::a,
            utf8: Some("a".into()),
        };
        let b = KeyEvent {
            time: 0,
            raw_code: 56,
            keysym: Keysym::b,
            utf8: Some("b".into()),
        };

        // Press A: it starts repeating.
        let mut repeat: Option<RepeatState> = None;
        if should_repeat_key(&a) {
            repeat = Some(RepeatState {
                event: a.clone(),
                delay: Duration::from_millis(200),
                rate: Duration::from_millis(40),
                first: true,
                pending: false,
                generation: 1,
            });
        }

        // Press B while A is held: B becomes the repeated key.
        if should_repeat_key(&b) {
            repeat = Some(RepeatState {
                event: b.clone(),
                delay: Duration::from_millis(200),
                rate: Duration::from_millis(40),
                first: true,
                pending: false,
                generation: 2,
            });
        }

        // Release A: repeat must stay alive because B is the repeated key.
        assert!(!released_repeated_key(&repeat, &a));
        assert!(
            repeat
                .as_ref()
                .is_some_and(|state| state.event.keysym == b.keysym)
        );
    }

    #[test]
    fn interleaved_info_events_update_their_own_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(&mut infos, 0, InfoEvent::Title("Editor".into()));
        apply_info_event(&mut infos, 1, InfoEvent::Title("Navegador".into()));
        apply_info_event(&mut infos, 0, InfoEvent::AppId("editor".into()));
        apply_info_event(
            &mut infos,
            1,
            InfoEvent::State {
                active: true,
                maximized: true,
                minimized: false,
            },
        );
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::State {
                active: false,
                maximized: false,
                minimized: true,
            },
        );

        assert_eq!(infos[0].title, "Editor");
        assert_eq!(infos[0].app_id, "editor");
        assert!(!infos[0].active);
        assert!(!infos[0].maximized);
        assert!(infos[0].minimized);
        assert_eq!(infos[1].title, "Navegador");
        assert!(infos[1].active);
        assert!(infos[1].maximized);
        assert!(!infos[1].minimized);

        apply_info_event(&mut infos, 0, InfoEvent::Closed);
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].title, "Navegador");
    }

    #[test]
    fn app_menu_event_updates_only_its_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::AppMenu(Some(AppMenuAddress {
                service: ":1.2".to_string(),
                path: "/MenuBar/1".to_string(),
            })),
        );
        assert_eq!(
            infos[0].app_menu.as_ref().map(|a| a.service.as_str()),
            Some(":1.2")
        );
        assert!(infos[1].app_menu.is_none());

        apply_info_event(&mut infos, 0, InfoEvent::AppMenu(None));
        assert!(infos[0].app_menu.is_none());
    }

    #[test]
    fn gtk_menus_event_updates_only_its_toplevel() {
        let mut infos = vec![ToplevelInfo::default(), ToplevelInfo::default()];
        apply_info_event(
            &mut infos,
            0,
            InfoEvent::GtkMenus(GtkMenusAddress::new(
                ":1.3".to_string(),
                "org.gtk.Demo".to_string(),
                "/menubar".to_string(),
                "/appmenu".to_string(),
                "/application".to_string(),
                "/window".to_string(),
            )),
        );
        assert_eq!(
            infos[0]
                .gtk_menus
                .as_ref()
                .map(|a| a.unique_bus_name.as_str()),
            Some(":1.3")
        );
        assert!(infos[1].gtk_menus.is_none());

        apply_info_event(&mut infos, 0, InfoEvent::GtkMenus(None));
        assert!(infos[0].gtk_menus.is_none());
    }

    #[test]
    fn gtk_menus_address_needs_a_bus_name() {
        assert!(
            GtkMenusAddress::new(
                "".to_string(),
                "org.gtk.Demo".to_string(),
                "/menubar".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .is_none()
        );
    }

    #[test]
    fn gtk_menus_address_accepts_app_menu_only() {
        let address = GtkMenusAddress::new(
            ":1.2".to_string(),
            "org.gtk.Demo".to_string(),
            "".to_string(),
            "/appmenu".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(address.is_some());
        assert_eq!(address.unwrap().app_menu_path, "/appmenu");
    }

    #[test]
    fn gtk_menus_address_accepts_menubar_only() {
        let address = GtkMenusAddress::new(
            ":1.2".to_string(),
            "org.gtk.Demo".to_string(),
            "/menubar".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        assert!(address.is_some());
        assert_eq!(address.unwrap().menubar_path, "/menubar");
    }

    #[test]
    fn gtk_menus_address_rejects_both_paths_empty() {
        assert!(
            GtkMenusAddress::new(
                ":1.2".to_string(),
                "org.gtk.Demo".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .is_none()
        );
    }

    #[test]
    fn sender_tags_user_events_with_surface_id() {
        let (ping, _) = calloop::ping::make_ping().unwrap();
        let queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>> = Rc::new(RefCell::new(Vec::new()));
        let owner = ObjectId::null();
        let sender = make_sender(queue.clone(), ping, owner.clone());
        sender(UserEvent::RequestRedraw);
        let events = drain_user_events(&queue);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, owner);
    }

    #[test]
    fn cursor_icon_maps_to_wayland_shape() {
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Default),
            CursorShape::Default
        );
        assert_eq!(cursor_shape_from_icon(CursorIcon::Text), CursorShape::Text);
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Pointer),
            CursorShape::Pointer
        );
        assert_eq!(cursor_shape_from_icon(CursorIcon::Grab), CursorShape::Grab);
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::Grabbing),
            CursorShape::Grabbing
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NotAllowed),
            CursorShape::NotAllowed
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NResize),
            CursorShape::NResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::EResize),
            CursorShape::EResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::EwResize),
            CursorShape::EwResize
        );
        assert_eq!(
            cursor_shape_from_icon(CursorIcon::NsResize),
            CursorShape::NsResize
        );
    }

    #[test]
    fn touch_cancel_dispatches_cancelled_for_every_active_contact() {
        let mut touch_surfaces: std::collections::HashMap<i32, ObjectId> =
            std::collections::HashMap::new();
        let surface_a = ObjectId::null();
        let surface_b = ObjectId::null();
        // Simulate two fingers down on surface A and one on surface B.
        touch_surfaces.insert(0, surface_a.clone());
        touch_surfaces.insert(1, surface_a.clone());
        touch_surfaces.insert(2, surface_b.clone());

        let mut cancelled = Vec::new();
        dispatch_touch_cancels(&mut touch_surfaces, |id, surface_id| {
            cancelled.push((id, surface_id));
        });

        assert!(
            touch_surfaces.is_empty(),
            "touch_surfaces must be empty after cancel"
        );
        assert_eq!(cancelled.len(), 3);
        assert!(cancelled.contains(&(0, surface_a.clone())));
        assert!(cancelled.contains(&(1, surface_a.clone())));
        assert!(cancelled.contains(&(2, surface_b.clone())));
    }

    #[test]
    fn touch_routing_uses_the_stored_surface_id() {
        // We cannot build real WlSurface objects in a unit test, so we model
        // the surfaces as (label, id) tuples. Surface B was created first,
        // surface A second; a touch started on A must route to A, not to B.
        let surfaces = vec![('B', 1usize), ('A', 2usize)];
        let touch_id = 7;
        let stored_surface_id = 2usize;

        // Simulate down storing A's id for this touch.
        let mut touch_surfaces: std::collections::HashMap<i32, usize> =
            std::collections::HashMap::new();
        touch_surfaces.insert(touch_id, stored_surface_id);

        // Simulate motion / up: the stored id is looked up in the surface list.
        let routed_id = touch_surfaces.get(&touch_id).copied();
        let index = find_index_by_id(&surfaces, routed_id.unwrap(), |item| item.1);

        assert_eq!(routed_id, Some(stored_surface_id));
        assert_eq!(
            index,
            Some(1),
            "motion/up for the same touch id must reach surface A"
        );
    }

    #[test]
    fn destroy_fractional_objects_is_safe_when_missing() {
        let mut fractional: Option<WpFractionalScaleV1> = None;
        let mut viewport: Option<WpViewport> = None;
        destroy_fractional_objects(&mut fractional, &mut viewport);
        assert!(fractional.is_none());
        assert!(viewport.is_none());
    }

    // Manual test for destroy order:
    // Run with WAYLAND_DEBUG=1 and close a window. Verify that
    // wp_fractional_scale_v1@...destroy and wp_viewport@...destroy lines appear
    // before the wl_surface@...destroy line for the same surface.

    // Manual test for the configure overwrite behaviour:
    // Run with WAYLAND_DEBUG=1 while resizing. Two configures arriving in the
    // same batch (same size, different state) must result in the second state
    // being applied; the code now stores only the latest configure in
    // `pending_configure: Option<_>`.
}

#[cfg(test)]
mod drag_band_tests {
    use super::*;
    use crate::ProbeExt;
    use crate::sidebar::{Sidebar, SidebarItem, SidebarLayout};
    use freya_core::integration::AppComponent;
    use freya_core::prelude::{App, IntoElement, rect};
    use freya_testing::TestingRunner;
    use torin::prelude::{Size, Size2D};
    use vow_appearance::{AppearanceConfig, SidebarStyle, resolve};
    use window::WindowCommand;

    struct DragCase;

    impl App for DragCase {
        fn render(&self) -> impl IntoElement {
            SidebarLayout::new(
                Sidebar::new()
                    .traffic_lights(true)
                    .child(SidebarItem::new("Uno").probe("island-first-item")),
                rect().width(Size::fill()).height(Size::fill()),
            )
        }
    }

    #[test]
    fn a_mouse_down_on_the_island_top_band_moves_the_window() {
        let queue: Rc<RefCell<Vec<WindowCommand>>> = Rc::new(RefCell::new(Vec::new()));
        let controls_queue = queue.clone();
        let resolved = resolve(
            &AppearanceConfig {
                sidebar_style: SidebarStyle::Island,
                ..Default::default()
            },
            None,
        );
        let app = AppComponent::new(DragCase);
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(400.0, 400.0),
            move |runner| {
                runner.provide_root_context(move || {
                    let (ping, _source) = calloop::ping::make_ping().unwrap();
                    WindowControls {
                        queue: controls_queue.clone(),
                        ping,
                        pointer_serial: Rc::new(Cell::new(0)),
                        status: State::create(WindowStatus {
                            activated: true,
                            maximized: false,
                            decoration: DecorationResponse::Client,
                            fixed: false,
                        }),
                        title: "Ventana".to_string(),
                        fixed: false,
                        surface_id: ObjectId::null(),
                        menu_handle: None,
                    }
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        test.press_cursor((100.0, 4.0));
        assert!(
            queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the island top band must request a window move"
        );

        queue.borrow_mut().clear();
        test.press_cursor((30.0, 30.0));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on a traffic light must not move the window"
        );

        queue.borrow_mut().clear();
        let (x, y) = test
            .find(|node, el| {
                (el.accessibility().builder.author_id() == Some("island-first-item")).then(|| {
                    let area = node.layout().area;
                    (
                        area.origin.x + area.size.width / 2.0,
                        area.origin.y + area.size.height / 2.0,
                    )
                })
            })
            .expect("the first island row exists");
        test.press_cursor((f64::from(x), f64::from(y)));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the first island row must not move the window"
        );
    }
}
