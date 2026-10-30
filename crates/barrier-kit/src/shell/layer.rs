use freya_core::integration::AppComponent;
use sctk::{
    reexports::{
        calloop::EventLoop,
        calloop_wayland_source::WaylandSource,
        client::{Connection, Proxy, QueueHandle, delegate_dispatch},
        protocols::xdg::shell::client::xdg_popup::XdgPopup,
        protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    },
    shell::wlr_layer::{
        Anchor as SctkAnchor, KeyboardInteractivity as SctkKeyboardInteractivity,
        Layer as SctkLayer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        LayerSurfaceData,
    },
};
use std::{cell::Cell, rc::Rc, sync::Arc};

use crate::platform::{
    RunError,
    app::AppInstance,
    popup::Popups,
    runtime::{Connected, Runtime, insert_ping, make_sender},
    surface::{ExtensionRole, SurfaceRole, SurfaceWindow},
    timer::{PingWake, Timers},
};
use crate::shell::{ShellState, SurfaceSpec};

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

pub(super) fn anchors_to_sctk(a: Anchors) -> SctkAnchor {
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

/// Runs a layer-shell surface with a Freya app until the compositor closes it.
pub fn run_layer(app: AppComponent, spec: SurfaceSpec) -> Result<(), RunError> {
    let (namespace, layer, anchors, exclusive_zone, keyboard, size) = match &spec {
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

    let Connected {
        conn,
        event_queue,
        runtime: mut shell,
        globals,
    } = Runtime::connect()?;
    let qh = shell.qh.clone();
    ShellState::install(&mut shell, app, &globals, &qh, false);
    let surface = shell.compositor.create_surface(&qh);
    let layer_surface = {
        let layer_shell = shell
            .extension::<ShellState>()
            .expect("ShellState not installed after install")
            .layer_shell
            .as_ref()
            .ok_or_else(|| {
                tracing::info!(
                    "zwlr_layer_shell_v1 no disponible; las superficies Layer no funcionan aquí"
                );
                RunError::MissingGlobal("zwlr_layer_shell_v1")
            })?;
        layer_shell.create_layer_surface(
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
        )
    };
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
    let app = shell
        .extension::<ShellState>()
        .expect("ShellState not installed after install")
        .app
        .clone();
    let app_instance = AppInstance::new(
        app,
        1.0,
        size.unwrap_or((1, 1)),
        sender,
        waker,
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
        |r| {
            if let Some(hooks) = shell.extension_hooks() {
                (hooks.provide_contexts)(&shell, r);
            }
        },
    );
    let (fractional, viewport) = shell.create_fractional(&surface);
    shell.surfaces.push(SurfaceWindow {
        surface: surface.clone(),
        role: SurfaceRole::Extension(Box::new(layer_surface)),
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

    let mut event_loop: EventLoop<Runtime> = EventLoop::try_new()
        .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut shell)?;
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::EventLoop("inserting the Wayland source"))?;
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
        .map_err(|_| RunError::EventLoop("calloop event loop failed"))?;
    Ok(())
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
        let Some(index) = self.surfaces.iter().position(|s| {
            s.role
                .as_extension_role::<LayerSurface>()
                .is_some_and(|existing| existing == layer_surface)
        }) else {
            return;
        };
        let scale = self.surfaces[index].scale;
        self.surfaces[index].configure(configure.new_size, scale);
        self.surfaces[index].request_redraw(qh);
    }
}

impl ExtensionRole for LayerSurface {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn parent_popup(&self, popup: &XdgPopup) -> bool {
        self.get_popup(popup);
        true
    }
}

delegate_dispatch!(Runtime: [zwlr_layer_shell_v1::ZwlrLayerShellV1: sctk::globals::GlobalData] => LayerShell);
delegate_dispatch!(Runtime: [zwlr_layer_surface_v1::ZwlrLayerSurfaceV1: LayerSurfaceData] => LayerShell);
