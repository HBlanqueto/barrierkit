use freya_core::integration::AppComponent;
use sctk::{
    reexports::{
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

use crate::platform::{
    RunError,
    app::AppInstance,
    runtime::{Connected, Runtime},
    surface::{ExtensionRole, SurfaceRole},
};
use crate::shell::ShellState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Anchors {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Anchors {
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

#[derive(Clone, Debug)]
pub struct LayerSpec {
    pub namespace: String,
    pub layer: Layer,
    pub anchors: Anchors,
    pub exclusive_zone: i32,
    pub keyboard: KeyboardInteractivity,
    pub size: Option<(u32, u32)>,
}

pub fn run_layer(app: AppComponent, spec: LayerSpec) -> Result<(), RunError> {
    let LayerSpec {
        namespace,
        layer,
        anchors,
        exclusive_zone,
        keyboard,
        size,
    } = spec;

    let Connected {
        conn,
        event_queue,
        runtime: mut shell,
        globals,
    } = Runtime::connect()?;
    let qh = shell.qh.clone();
    ShellState::install(&mut shell, app, &globals, &qh);
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

    let app = shell
        .extension::<ShellState>()
        .expect("ShellState not installed after install")
        .app
        .clone();
    let app_instance = AppInstance::new(
        &shell,
        surface.id(),
        app,
        1.0,
        size.unwrap_or((1, 1)),
        None,
        None,
        None,
    );
    let mut window = shell.surface_window(
        surface.clone(),
        SurfaceRole::Extension(Box::new(layer_surface)),
        app_instance,
    );
    window.logical_size = size.unwrap_or((0, 0));
    shell.surfaces.push(window);

    shell.run(conn, event_queue, |_, _| {})
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
        self.surfaces[index].needs_redraw = true;
        self.surfaces[index].draw_after_configure(&self.compositor, qh);
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
