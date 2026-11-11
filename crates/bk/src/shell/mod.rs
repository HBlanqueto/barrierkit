mod activation;
mod blur_region;
mod infinity;
mod layer;
mod lock;
mod menus;
mod screen_lock;
mod status_items;
mod toplevels;

pub use activation::Activation;
pub use blur_region::{BlurRegion, set_blur_region};
pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
pub use layer::{
    Anchors, CardFrames, InputRegion, KeyboardInteractivity, Layer, LayerGeometry, LayerSpec,
    run_layer, set_card_frames, set_input_region, set_layer_geometry,
};
pub use lock::{SessionUnlock, run_lock};
pub use menus::GtkMenusAddress;
pub use screen_lock::{ScreenLock, use_screen_lock};
pub use status_items::{Pixmap, StatusHost, TrayItem, use_status_items};
pub use toplevels::{FocusBar, ToplevelInfo, Toplevels};

use freya_core::integration::AppComponent;

use infinity_protocol::client::session::infinity_session_v1::infinity_session_v1::InfinitySessionV1;
use sctk::{
    reexports::{
        client::{QueueHandle, backend::ObjectId, globals::GlobalList},
        protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
    },
    session_lock::{SessionLock, SessionLockState},
    shell::wlr_layer::LayerShell,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1::ExtBackgroundEffectManagerV1,
    ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
};

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
    pub(crate) activation_callbacks: HashMap<ObjectId, ActivationCallback>,
    pub(crate) screen_lock: freya_core::prelude::State<ScreenLock>,
    pub(crate) _session: Option<InfinitySessionV1>,
    pub(crate) background_effect: Option<ExtBackgroundEffectManagerV1>,
    pub(crate) background_effects: HashMap<ObjectId, ExtBackgroundEffectSurfaceV1>,
}

impl ShellState {
    pub(crate) fn install(
        rt: &mut Runtime,
        app: AppComponent,
        globals: &GlobalList,
        qh: &QueueHandle<Runtime>,
    ) {
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
            .bind::<InfinityWindowManagerV1, _, _>(qh, 1..=8, ())
            .ok();
        if infinity.is_none() {
            tracing::info!("infinity_window_manager_v1 no disponible; sigo sin él");
        }
        let toplevels = Toplevels {
            list: freya_core::prelude::State::create_global(Vec::new()),
            focus_bar: freya_core::prelude::State::create_global(FocusBar::default()),
            desktop: freya_core::prelude::State::create_global(None),
            entries: Rc::new(RefCell::new(Vec::new())),
            seat: Rc::new(RefCell::new(rt.seat_handle.clone())),
            infinity,
        };
        let shell = Self {
            app,
            layer_shell: LayerShell::bind(globals, qh).ok(),
            session_lock: Some(SessionLockState::new(globals, qh)),
            active_lock: None,
            session_unlock: None,
            _foreign_list: foreign_list,
            _foreign_mgmt: foreign_mgmt,
            toplevels,
            next_toplevel_id: 0,
            activation: Activation {
                queue: Rc::new(RefCell::new(Vec::new())),
                ping: rt.ping.clone(),
            },
            activation_callbacks: HashMap::new(),
            screen_lock: freya_core::prelude::State::create_global(ScreenLock::default()),
            _session: globals.bind::<InfinitySessionV1, _, _>(qh, 1..=1, ()).ok(),
            background_effect: globals
                .bind::<ExtBackgroundEffectManagerV1, _, _>(qh, 1..=1, ())
                .ok(),
            background_effects: HashMap::new(),
        };

        rt.extension = Some(Extension {
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
        let screen_lock = shell.screen_lock;
        runner.provide_root_context(move || screen_lock);
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
        let global = rt.activation.clone();
        let window = rt
            .last_input_surface
            .clone()
            .filter(|surface| rt.surface_for(surface).is_some())
            .or_else(|| rt.surfaces.first().map(|s| s.surface.clone()));
        let serial = rt.last_input_serial.get();
        let qh = rt.qh.clone();
        if let Some(shell) = rt.extension_mut::<ShellState>() {
            shell.poll_activation(global.as_ref(), seat.as_ref(), window.as_ref(), serial, &qh);
        }
    }
}

#[cfg(test)]
mod tests;
