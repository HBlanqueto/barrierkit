use freya_core::integration::AppComponent;
use sctk::{
    compositor::Region,
    reexports::{
        client::{Connection, Proxy, QueueHandle, backend::ObjectId, delegate_dispatch},
        protocols::xdg::shell::client::xdg_popup::XdgPopup,
        protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    },
    shell::WaylandSurface,
    shell::wlr_layer::{
        Anchor as SctkAnchor, KeyboardInteractivity as SctkKeyboardInteractivity,
        Layer as SctkLayer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        LayerSurfaceData,
    },
};

use crate::platform::{
    RunError,
    app::{AppInstance, InstanceRole},
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerGeometry {
    pub anchors: Anchors,
    pub exclusive_zone: i32,
    pub size: (u32, u32),
    pub keyboard: KeyboardInteractivity,
}

pub fn set_layer_geometry(geometry: LayerGeometry) {
    crate::platform::KitSender::get().send(geometry);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardFrames {
    pub radius: u32,
    pub cards: Vec<(i32, i32, i32, i32)>,
}

pub fn set_card_frames(frames: CardFrames) {
    crate::platform::KitSender::get().send(frames);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputRegion {
    pub rects: Vec<(i32, i32, i32, i32)>,
}

pub fn set_input_region(region: InputRegion) {
    crate::platform::KitSender::get().send(region);
}

fn keyboard_to_sctk(keyboard: KeyboardInteractivity) -> SctkKeyboardInteractivity {
    match keyboard {
        KeyboardInteractivity::None => SctkKeyboardInteractivity::None,
        KeyboardInteractivity::Exclusive => SctkKeyboardInteractivity::Exclusive,
        KeyboardInteractivity::OnDemand => SctkKeyboardInteractivity::OnDemand,
    }
}

impl Runtime {
    pub(crate) fn apply_layer_geometry(&mut self, surface_id: &ObjectId, geometry: LayerGeometry) {
        if !size_is_valid(geometry.anchors, Some(geometry.size)) {
            tracing::warn!(
                ?geometry,
                "BarrierKit: ignored a layer geometry with an unanchored zero size"
            );
            return;
        }
        let Some(layer_surface) = self.surface_index_by_id(surface_id).and_then(|index| {
            self.surfaces[index]
                .role
                .as_extension_role::<LayerSurface>()
        }) else {
            return;
        };
        layer_surface.set_anchor(anchors_to_sctk(geometry.anchors));
        layer_surface.set_size(geometry.size.0, geometry.size.1);
        layer_surface.set_exclusive_zone(geometry.exclusive_zone);
        layer_surface.set_keyboard_interactivity(keyboard_to_sctk(geometry.keyboard));
        layer_surface.wl_surface().commit();
    }
}

impl Runtime {
    pub(crate) fn apply_input_region(&mut self, surface_id: &ObjectId, region: InputRegion) {
        let Some(index) = self.surface_index_by_id(surface_id) else {
            return;
        };
        let Ok(wl_region) = Region::new(&self.compositor) else {
            return;
        };
        for &(x, y, width, height) in &region.rects {
            wl_region.add(x, y, width, height);
        }
        let surface = &self.surfaces[index].surface;
        surface.set_input_region(Some(wl_region.wl_region()));
        surface.commit();
    }
}

impl Runtime {
    pub(crate) fn apply_card_frames(&mut self, surface_id: &ObjectId, frames: CardFrames) {
        let Some(style) = self
            .window_style
            .as_ref()
            .filter(|style| style.version() >= 4)
        else {
            return;
        };
        let Some(index) = self.surface_index_by_id(surface_id) else {
            return;
        };
        let cards = frames
            .cards
            .iter()
            .flat_map(|&(x, y, width, height)| [x, y, width, height])
            .flat_map(i32::to_ne_bytes)
            .collect();
        style.set_cards(&self.surfaces[index].surface, frames.radius, cards);
    }
}

// zwlr_layer_surface_v1.set_size: a zero dimension needs the anchors on both of its edges.
fn size_is_valid(anchors: Anchors, size: Option<(u32, u32)>) -> bool {
    let (width, height) = size.unwrap_or_default();
    (width > 0 || anchors.left && anchors.right) && (height > 0 || anchors.top && anchors.bottom)
}

pub fn run_layer(app: AppComponent, spec: LayerSpec) -> Result<(), RunError> {
    if !size_is_valid(spec.anchors, spec.size) {
        return Err(RunError::LayerSize);
    }
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
    layer_surface.set_keyboard_interactivity(keyboard_to_sctk(keyboard));
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
        InstanceRole::Surface,
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

#[cfg(test)]
mod tests {
    use super::{Anchors, size_is_valid};

    #[test]
    fn a_bar_anchored_on_three_edges_needs_a_height() {
        assert!(!size_is_valid(Anchors::TOP, None));
        assert!(size_is_valid(Anchors::TOP, Some((0, 30))));
    }

    #[test]
    fn a_surface_anchored_on_every_edge_may_leave_the_size_to_the_compositor() {
        let all = Anchors {
            top: true,
            bottom: true,
            left: true,
            right: true,
        };
        assert!(size_is_valid(all, None));
        assert!(!size_is_valid(Anchors::default(), Some((0, 10))));
    }
}
