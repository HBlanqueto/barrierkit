mod activation;
mod blur_region;
mod capture;
mod infinity;
mod layer;
mod lock;
mod menus;
mod screen_lock;
mod status_items;
mod theme;
mod toplevels;

pub use activation::Activation;
pub use blur_region::{BlurRegion, set_blur_region};
pub use capture::{CaptureWindow, WindowImage, capture_window};
pub use infinity::{
    InfinityWindowManagerV1, Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
pub use layer::{
    Anchors, CardFrames, InputRegion, KeyboardInteractivity, Layer, LayerGeometry, LayerSpec,
    PlateFrames, run_layer, set_card_frames, set_input_region, set_layer_geometry,
    set_plate_frames,
};
pub use lock::{SessionUnlock, run_lock};
pub use menus::GtkMenusAddress;
pub use screen_lock::{ScreenLock, use_screen_lock};
pub use status_items::{Pixmap, StatusHost, TrayItem, use_status_items};
pub use theme::{for_scheme, use_shell_theme};
pub use toplevels::{FocusBar, MinimizeTarget, ToplevelInfo, Toplevels, set_minimize_target};

use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use bk::extension::{Extension, ExtensionHooks, Runtime};
use freya_core::{
    integration::{AppComponent, Runner},
    prelude::State,
};
use infinity_protocol::client::session::infinity_session_v1::infinity_session_v1::InfinitySessionV1;
use sctk::reexports::{
    client::{
        QueueHandle,
        backend::ObjectId,
        globals::GlobalList,
        protocol::{wl_output::WlOutput, wl_seat::WlSeat},
    },
    protocols::ext::{
        foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        session_lock::v1::client::ext_session_lock_manager_v1::ExtSessionLockManagerV1,
    },
    protocols_wlr::{
        foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    },
};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1::ExtBackgroundEffectManagerV1,
    ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
};

use crate::{activation::ActivationCallback, lock::Lock};

// The orphan rule: a Dispatch impl for bk's Runtime needs a local type among its parameters.
pub(crate) struct Shell;

pub(crate) struct ShellState {
    pub(crate) app: AppComponent,
    pub(crate) layer_shell: Option<ZwlrLayerShellV1>,
    pub(crate) session_lock: Option<ExtSessionLockManagerV1>,
    pub(crate) active_lock: Option<Lock>,
    pub(crate) session_unlock: Option<SessionUnlock>,
    pub(crate) _foreign_list: Option<ExtForeignToplevelListV1>,
    pub(crate) _foreign_mgmt: Option<ZwlrForeignToplevelManagerV1>,
    pub(crate) toplevels: Toplevels,
    pub(crate) next_toplevel_id: u64,
    pub(crate) activation: Activation,
    pub(crate) activation_callbacks: HashMap<ObjectId, ActivationCallback>,
    pub(crate) screen_lock: State<ScreenLock>,
    pub(crate) _session: Option<InfinitySessionV1>,
    pub(crate) background_effect: Option<ExtBackgroundEffectManagerV1>,
    pub(crate) background_effects: HashMap<ObjectId, ExtBackgroundEffectSurfaceV1>,
    pub(crate) capture: Option<capture::Capture>,
}

impl ShellState {
    pub(crate) fn install(
        rt: &mut Runtime,
        app: AppComponent,
        globals: &GlobalList,
        qh: &QueueHandle<Runtime>,
    ) {
        let foreign_mgmt = globals
            .bind::<ZwlrForeignToplevelManagerV1, _, _>(qh, 3..=3, Shell)
            .ok();
        let foreign_list = if foreign_mgmt.is_some() {
            None
        } else {
            tracing::info!(
                "zwlr_foreign_toplevel_manager_v1 unavailable; using ext_foreign_toplevel_list_v1"
            );
            globals
                .bind::<ExtForeignToplevelListV1, _, _>(qh, 1..=1, Shell)
                .ok()
        };
        let infinity = globals
            .bind::<InfinityWindowManagerV1, _, _>(qh, 1..=8, Shell)
            .ok();
        if infinity.is_none() {
            tracing::info!("infinity_window_manager_v1 unavailable; continuing without it");
        }
        let toplevels = Toplevels {
            list: State::create_global(Vec::new()),
            focus_bar: State::create_global(FocusBar::default()),
            desktop: State::create_global(None),
            entries: Rc::new(RefCell::new(Vec::new())),
            seat: Rc::new(RefCell::new(rt.seat_handle.clone())),
            infinity,
        };
        let shell = Self {
            app,
            layer_shell: globals.bind(qh, 1..=4, Shell).ok(),
            session_lock: globals.bind(qh, 1..=1, Shell).ok(),
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
            screen_lock: State::create_global(ScreenLock::default()),
            _session: globals
                .bind::<InfinitySessionV1, _, _>(qh, 1..=1, Shell)
                .ok(),
            background_effect: globals
                .bind::<ExtBackgroundEffectManagerV1, _, _>(qh, 1..=1, Shell)
                .ok(),
            background_effects: HashMap::new(),
            capture: capture::Capture::bind(globals, qh),
        };

        rt.extension = Some(Extension {
            state: Box::new(shell),
            hooks: ExtensionHooks {
                provide_contexts: Self::provide_contexts,
                new_seat: Self::new_seat,
                new_output: Self::new_output,
                poll: Self::poll,
                kit_event: Self::kit_event,
            },
        });
    }

    fn provide_contexts(rt: &Runtime, runner: &mut Runner) {
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

    fn new_seat(rt: &mut Runtime, seat: &WlSeat) {
        *rt.extension::<ShellState>()
            .expect("new_seat called without ShellState")
            .toplevels
            .seat
            .borrow_mut() = Some(seat.clone());
    }

    fn new_output(rt: &mut Runtime, qh: &QueueHandle<Runtime>, output: &WlOutput) {
        lock::create_lock_surface(rt, qh, output);
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

    fn kit_event(rt: &mut Runtime, surface: &ObjectId, event: Box<dyn Any>) {
        let Err(event) = event
            .downcast::<CardFrames>()
            .map(|frames| layer::apply_card_frames(rt, surface, *frames))
        else {
            return;
        };
        let Err(event) = event
            .downcast::<PlateFrames>()
            .map(|frames| layer::apply_plate_frames(rt, surface, *frames))
        else {
            return;
        };
        let Err(event) = event
            .downcast::<BlurRegion>()
            .map(|region| blur_region::apply_blur_region(rt, surface, *region))
        else {
            return;
        };
        let Err(event) = event
            .downcast::<CaptureWindow>()
            .map(|request| capture::capture(rt, *request))
        else {
            return;
        };
        let Err(event) = event
            .downcast::<MinimizeTarget>()
            .map(|target| toplevels::apply_minimize_target(rt, surface, *target))
        else {
            return;
        };
        let Err(event) = event
            .downcast::<InputRegion>()
            .map(|region| layer::apply_input_region(rt, surface, *region))
        else {
            return;
        };
        if let Ok(geometry) = event.downcast::<LayerGeometry>() {
            layer::apply_layer_geometry(rt, surface, *geometry);
        }
    }
}
