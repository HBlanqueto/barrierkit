mod activation;
mod infinity;
mod layer;
mod lock;
mod menus;
mod toplevels;

pub use activation::Activation;
pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
pub use layer::{Anchors, KeyboardInteractivity, Layer, LayerSpec, run_layer};
pub use lock::{SessionUnlock, run_lock};
pub use menus::GtkMenusAddress;
pub use toplevels::{FocusBar, SpaceInfo, ToplevelInfo, Toplevels};

use freya_core::integration::AppComponent;

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

pub(crate) struct ShellState {
    pub(crate) app: AppComponent,
    pub(crate) layer_shell: Option<LayerShell>,
    pub(crate) session_lock: Option<SessionLockState>,
    pub(crate) active_lock: Option<SessionLock>,
    pub(crate) session_unlock: Option<SessionUnlock>,
    pub(crate) _foreign_list: Option<ExtForeignToplevelListV1>,
    pub(crate) _foreign_mgmt: Option<ZwlrForeignToplevelManagerV1>,
    pub(crate) toplevels: Toplevels,
    pub(crate) next_toplevel_id: u64,
    pub(crate) activation: Activation,
    pub(crate) activation_global: Option<XdgActivationV1>,
    pub(crate) activation_callbacks: Vec<Option<ActivationCallback>>,
}

impl ShellState {
    pub(crate) fn install(
        rt: &mut Runtime,
        app: AppComponent,
        globals: &GlobalList,
        qh: &QueueHandle<Runtime>,
    ) {
        let layer_shell = LayerShell::bind(globals, qh).ok();
        let session_lock = Some(SessionLockState::new(globals, qh));

        let foreign_mgmt = globals
            .bind::<ZwlrForeignToplevelManagerV1, _, _>(qh, 3..=3, ())
            .ok();
        let foreign_list = if foreign_mgmt.is_some() {
            None
        } else {
            tracing::info!(
                "zwlr_foreign_toplevel_manager_v1 no disponible; uso ext_foreign_toplevel_list_v1"
            );
            globals
                .bind::<ExtForeignToplevelListV1, _, _>(qh, 1..=1, ())
                .ok()
        };
        let infinity = globals
            .bind::<InfinityWindowManagerV1, _, _>(qh, 1..=5, ())
            .ok();
        if infinity.is_none() {
            tracing::info!("infinity_window_manager_v1 no disponible; sigo sin él");
        }
        let toplevels = Toplevels {
            list: freya_core::prelude::State::create_global(Vec::new()),
            space: freya_core::prelude::State::create_global(SpaceInfo::default()),
            entries: Rc::new(RefCell::new(Vec::new())),
            seat: Rc::new(RefCell::new(rt.seat_handle.clone())),
            infinity,
        };
        let activation_global = globals.bind::<XdgActivationV1, _, _>(qh, 1..=1, ()).ok();

        let activation = Activation {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: rt.ping.clone(),
        };

        let shell = Self {
            app,
            layer_shell,
            session_lock,
            active_lock: None,
            session_unlock: None,
            _foreign_list: foreign_list,
            _foreign_mgmt: foreign_mgmt,
            toplevels,
            next_toplevel_id: 0,
            activation,
            activation_global,
            activation_callbacks: Vec::new(),
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
        *rt.extension::<ShellState>()
            .expect("new_seat called without ShellState")
            .toplevels
            .seat
            .borrow_mut() = Some(seat.clone());
    }

    fn new_output(
        rt: &mut Runtime,
        qh: &QueueHandle<Runtime>,
        output: &sctk::reexports::client::protocol::wl_output::WlOutput,
    ) {
        rt.create_lock_surface(qh, output);
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
