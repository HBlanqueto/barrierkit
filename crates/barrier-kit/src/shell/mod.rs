pub mod activation;
pub mod infinity;
pub mod layer;
pub mod lock;
pub mod menus;
pub mod toplevels;

pub use activation::Activation;
pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
pub use layer::{Anchors, KeyboardInteractivity, Layer, run_layer};
pub use lock::{SessionUnlock, run_lock};
pub use menus::GtkMenusAddress;
pub use toplevels::{ToplevelInfo, Toplevels};

use crate::platform::WindowOptions;

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

use sctk::{
    reexports::{
        client::{QueueHandle, globals::GlobalList},
        protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1,
        protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
    },
    session_lock::{SessionLock, SessionLockState},
    shell::wlr_layer::LayerShell,
};
use std::{cell::RefCell, rc::Rc};

use crate::platform::{
    extension::{Extension, ExtensionHooks},
    runtime::Runtime,
};
use activation::ActivationCallback;
use toplevels::ToplevelEntry;

/// All state owned by the shell (dock, menubar, lock, launcher). It lives
/// inside `Runtime` as the single installed [`Extension`].
pub(crate) struct ShellState {
    pub(crate) layer_shell: Option<LayerShell>,
    pub(crate) session_lock: Option<SessionLockState>,
    pub(crate) active_lock: Option<SessionLock>,
    pub(crate) session_unlock: Option<SessionUnlock>,
    pub(crate) session_locked: bool,
    pub(crate) _foreign_list: Option<ExtForeignToplevelListV1>,
    pub(crate) foreign_mgmt: Option<ZwlrForeignToplevelManagerV1>,
    pub(crate) _infinity_manager: Rc<RefCell<Option<InfinityWindowManagerV1>>>,
    pub(crate) toplevel_entries: Rc<RefCell<Vec<ToplevelEntry>>>,
    pub(crate) toplevels: Toplevels,
    pub(crate) activation: Activation,
    pub(crate) activation_global: Option<XdgActivationV1>,
    pub(crate) activation_callbacks: Vec<Option<ActivationCallback>>,
    pub(crate) lock_mode: bool,
}

impl ShellState {
    /// Binds the shell globals, builds `Toplevels` and `Activation`, and
    /// installs the extension into `rt`.
    pub(crate) fn install(
        rt: &mut Runtime,
        globals: &GlobalList,
        qh: &QueueHandle<Runtime>,
        lock_mode: bool,
    ) {
        let layer_shell = LayerShell::bind(globals, qh).ok();
        let session_lock = Some(SessionLockState::new(globals, qh));

        let toplevel_entries: Rc<RefCell<Vec<ToplevelEntry>>> = Rc::new(RefCell::new(Vec::new()));
        let close_entries = toplevel_entries.clone();
        let minimize_entries = toplevel_entries.clone();
        let maximize_entries = toplevel_entries.clone();
        let unminimize_entries = toplevel_entries.clone();
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
            list: freya_core::prelude::State::create_global(Vec::new()),
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
            .bind::<ExtForeignToplevelListV1, _, _>(qh, 1..=1, ())
            .ok();
        if foreign_list.is_none() {
            tracing::info!("ext_foreign_toplevel_list_v1 no disponible; sigo sin él");
        }
        let foreign_mgmt = globals
            .bind::<ZwlrForeignToplevelManagerV1, _, _>(qh, 3..=3, ())
            .ok();
        if foreign_mgmt.is_none() {
            tracing::info!("zwlr_foreign_toplevel_manager_v1 no disponible; sigo sin él");
        }
        let infinity_manager_bound = globals
            .bind::<InfinityWindowManagerV1, _, _>(qh, 1..=3, ())
            .ok();
        if infinity_manager_bound.is_none() {
            tracing::info!("infinity_window_manager_v1 no disponible; sigo sin él");
        }
        *infinity_manager.borrow_mut() = infinity_manager_bound;
        let activation_global = globals.bind::<XdgActivationV1, _, _>(qh, 1..=1, ()).ok();

        if let Some(seat) = rt.seat_handle.clone() {
            let entries = toplevel_entries.clone();
            *toplevels.activate.borrow_mut() = Some(Rc::new(move |index| {
                if let Some(entry) = entries.borrow().get(index)
                    && let Some(mgmt) = &entry.mgmt
                {
                    mgmt.activate(&seat);
                }
            }));
        }

        let activation = Activation {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: rt.ping.clone(),
        };

        let shell = Self {
            layer_shell,
            session_lock,
            active_lock: None,
            session_unlock: None,
            session_locked: false,
            _foreign_list: foreign_list,
            foreign_mgmt,
            _infinity_manager: infinity_manager,
            toplevel_entries,
            toplevels,
            activation,
            activation_global,
            activation_callbacks: Vec::new(),
            lock_mode,
        };

        rt.install_extension(Extension {
            state: Box::new(shell),
            hooks: ExtensionHooks {
                provide_contexts: Self::provide_contexts,
                new_seat: Self::new_seat,
                new_output: Self::new_output,
                poll: Self::poll,
            },
        });
    }

    fn provide_contexts(rt: &Runtime, runner: &mut freya_core::integration::Runner) {
        let shell = rt
            .extension::<ShellState>()
            .expect("provide_contexts called without ShellState");
        let toplevels = shell.toplevels.clone();
        runner.provide_root_context(move || toplevels);
        if let Some(unlock) = shell.session_unlock.clone() {
            runner.provide_root_context(move || unlock);
        }
        let activation = shell.activation.clone();
        runner.provide_root_context(move || activation);
    }

    fn new_seat(rt: &mut Runtime, seat: &sctk::reexports::client::protocol::wl_seat::WlSeat) {
        let seat = seat.clone();
        let (entries, activate) = {
            let shell = rt
                .extension::<ShellState>()
                .expect("new_seat called without ShellState");
            (
                shell.toplevel_entries.clone(),
                shell.toplevels.activate.clone(),
            )
        };
        *activate.borrow_mut() = Some(Rc::new(move |index| {
            if let Some(entry) = entries.borrow().get(index)
                && let Some(mgmt) = &entry.mgmt
            {
                mgmt.activate(&seat);
            }
        }));
    }

    fn new_output(
        rt: &mut Runtime,
        qh: &QueueHandle<Runtime>,
        output: &sctk::reexports::client::protocol::wl_output::WlOutput,
    ) {
        let (session_locked, lock_mode) = {
            let shell = rt
                .extension::<ShellState>()
                .expect("new_output called without ShellState");
            (shell.session_locked, shell.lock_mode)
        };
        if session_locked && lock_mode {
            rt.create_lock_surface(qh, output);
        }
    }

    fn poll(rt: &mut Runtime) {
        let seat = rt.seat_handle.clone();
        let window = rt.surfaces.first().map(|s| s.surface.clone());
        let serial = rt.last_input_serial.get();
        let qh = rt.qh.clone();
        if let Some(shell) = rt.extension_mut::<ShellState>() {
            shell.poll_activation(seat.as_ref(), window.as_ref(), serial, &qh);
        }
    }
}

#[cfg(test)]
mod tests;
