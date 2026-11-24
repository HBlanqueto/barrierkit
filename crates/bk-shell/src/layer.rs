use bk::{
    extension::{
        AppInstance, Connected, ExtensionRole, InstanceRole, KitSender, Runtime, SurfaceRole,
        packed,
    },
    platform::RunError,
};
use freya_core::integration::AppComponent;
use sctk::{
    compositor::Region,
    reexports::{
        client::{
            Connection, Dispatch, Proxy, QueueHandle, backend::ObjectId,
            protocol::wl_surface::WlSurface,
        },
        protocols::xdg::shell::client::xdg_popup::XdgPopup,
        protocols_wlr::layer_shell::v1::client::{
            zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
            zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
        },
    },
};

use crate::{Shell, ShellState};

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

fn anchors_to_wire(a: Anchors) -> zwlr_layer_surface_v1::Anchor {
    use zwlr_layer_surface_v1::Anchor;
    [
        (a.top, Anchor::Top),
        (a.bottom, Anchor::Bottom),
        (a.left, Anchor::Left),
        (a.right, Anchor::Right),
    ]
    .into_iter()
    .filter(|(on, _)| *on)
    .fold(Anchor::empty(), |anchor, (_, edge)| anchor | edge)
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
    KitSender::get().send(geometry);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardFrames {
    pub radius: u32,
    pub cards: Vec<(i32, i32, i32, i32)>,
}

pub fn set_card_frames(frames: CardFrames) {
    KitSender::get().send(frames);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlateFrames {
    pub corner: u32,
    pub plates: Vec<(i32, i32, i32, i32)>,
}

pub fn set_plate_frames(frames: PlateFrames) {
    KitSender::get().send(frames);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputRegion {
    pub rects: Vec<(i32, i32, i32, i32)>,
}

pub fn set_input_region(region: InputRegion) {
    KitSender::get().send(region);
}

fn keyboard_to_wire(
    keyboard: KeyboardInteractivity,
) -> zwlr_layer_surface_v1::KeyboardInteractivity {
    match keyboard {
        KeyboardInteractivity::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
        KeyboardInteractivity::Exclusive => zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
        KeyboardInteractivity::OnDemand => zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
    }
}

pub(crate) fn apply_layer_geometry(
    rt: &mut Runtime,
    surface_id: &ObjectId,
    geometry: LayerGeometry,
) {
    if !size_is_valid(geometry.anchors, Some(geometry.size)) {
        tracing::warn!(
            ?geometry,
            "BarrierKit: ignored a layer geometry with an unanchored zero size"
        );
        return;
    }
    let Some(role) = rt
        .surface_index_by_id(surface_id)
        .and_then(|index| rt.surfaces[index].role.as_extension_role::<LayerRole>())
    else {
        return;
    };
    role.layer.set_anchor(anchors_to_wire(geometry.anchors));
    role.layer.set_size(geometry.size.0, geometry.size.1);
    role.layer.set_exclusive_zone(geometry.exclusive_zone);
    role.layer
        .set_keyboard_interactivity(keyboard_to_wire(geometry.keyboard));
    role.surface.commit();
}

pub(crate) fn apply_input_region(rt: &mut Runtime, surface_id: &ObjectId, region: InputRegion) {
    let Some(index) = rt.surface_index_by_id(surface_id) else {
        return;
    };
    let Ok(wl_region) = Region::new(&rt.compositor) else {
        return;
    };
    for &(x, y, width, height) in &region.rects {
        wl_region.add(x, y, width, height);
    }
    let surface = &rt.surfaces[index].surface;
    surface.set_input_region(Some(wl_region.wl_region()));
    surface.commit();
}

pub(crate) fn apply_card_frames(rt: &mut Runtime, surface_id: &ObjectId, frames: CardFrames) {
    let Some(style) = rt
        .window_style
        .as_ref()
        .filter(|style| style.version() >= 4)
    else {
        return;
    };
    let Some(index) = rt.surface_index_by_id(surface_id) else {
        return;
    };
    style.set_cards(
        &rt.surfaces[index].surface,
        frames.radius,
        packed(&frames.cards),
    );
}

pub(crate) fn apply_plate_frames(rt: &mut Runtime, surface_id: &ObjectId, frames: PlateFrames) {
    let Some(style) = rt
        .window_style
        .as_ref()
        .filter(|style| style.version() >= 8)
    else {
        return;
    };
    let Some(index) = rt.surface_index_by_id(surface_id) else {
        return;
    };
    style.set_plates(
        &rt.surfaces[index].surface,
        frames.corner,
        packed(&frames.plates),
    );
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
    let layer = shell
        .extension::<ShellState>()
        .expect("ShellState not installed after install")
        .layer_shell
        .as_ref()
        .ok_or_else(|| {
            tracing::info!("zwlr_layer_shell_v1 unavailable; Layer surfaces do not work here");
            RunError::MissingGlobal("zwlr_layer_shell_v1")
        })?
        .get_layer_surface(
            &surface,
            None,
            match layer {
                Layer::Background => zwlr_layer_shell_v1::Layer::Background,
                Layer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
                Layer::Top => zwlr_layer_shell_v1::Layer::Top,
                Layer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
            },
            namespace,
            &qh,
            Shell,
        );
    layer.set_anchor(anchors_to_wire(anchors));
    layer.set_exclusive_zone(exclusive_zone);
    layer.set_keyboard_interactivity(keyboard_to_wire(keyboard));
    if let Some((w, h)) = size {
        layer.set_size(w, h);
    }
    surface.commit();

    let app = shell
        .extension::<ShellState>()
        .expect("ShellState not installed after install")
        .app
        .clone();
    let app_instance = AppInstance::new(
        &shell.host(),
        surface.id(),
        app,
        1.0,
        size.unwrap_or((1, 1)),
        InstanceRole::Surface,
        |runner| shell.provide_extension_contexts(runner),
    );
    let role = LayerRole {
        layer,
        surface: surface.clone(),
    };
    let mut window = shell.surface_window(
        surface,
        SurfaceRole::Extension(Box::new(role)),
        app_instance,
    );
    window.logical_size = size.unwrap_or((0, 0));
    shell.surfaces.push(window);

    shell.run(conn, event_queue, |_, _| {})
}

#[derive(Debug)]
struct LayerRole {
    layer: ZwlrLayerSurfaceV1,
    surface: WlSurface,
}

// zwlr_layer_surface_v1: the role object is destroyed before its wl_surface.
impl Drop for LayerRole {
    fn drop(&mut self) {
        self.layer.destroy();
        self.surface.destroy();
    }
}

impl ExtensionRole for LayerRole {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn parent_popup(&self, popup: &XdgPopup) -> bool {
        self.layer.get_popup(popup);
        true
    }
}

impl Dispatch<ZwlrLayerShellV1, Shell> for Runtime {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, Shell> for Runtime {
    fn event(
        rt: &mut Self,
        layer: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &Shell,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                layer.ack_configure(serial);
                let Some(index) = rt.surfaces.iter().position(|s| {
                    s.role
                        .as_extension_role::<LayerRole>()
                        .is_some_and(|role| role.layer == *layer)
                }) else {
                    return;
                };
                let scale = rt.surfaces[index].scale;
                rt.surfaces[index].configure((width, height), scale);
                rt.surfaces[index].needs_redraw = true;
                rt.surfaces[index].draw_after_configure(&rt.compositor, qh);
            }
            zwlr_layer_surface_v1::Event::Closed => rt.needs_exit = true,
            _ => {}
        }
    }
}
