use std::{any::Any, fmt::Debug, rc::Rc};

use freya_core::{integration::RenderPipeline, prelude::Color};
use sctk::{
    compositor::CompositorHandler,
    reexports::{
        client::{
            Connection, Dispatch, Proxy, QueueHandle,
            backend::ObjectId,
            protocol::{wl_callback::WlCallback, wl_output, wl_surface::WlSurface},
        },
        protocols::wp::fractional_scale::v1::client::{
            wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
            wp_fractional_scale_v1::{Event as FractionalScaleEvent, WpFractionalScaleV1},
        },
        protocols::wp::viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
        protocols::xdg::shell::client::xdg_popup::XdgPopup,
    },
    shell::xdg::{
        XdgSurface,
        popup::Popup,
        window::{Window, WindowConfigure},
    },
};
use torin::prelude::CursorPoint;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;

use super::{
    app::AppInstance,
    app_menu_export::AppMenuAddress,
    egl::{Graphics, GraphicsShared},
    popup::PopupInfo,
    runtime::Runtime,
};

/// A surface plus its Freya app and GL context.
pub(crate) struct SurfaceWindow {
    pub(crate) surface: WlSurface,
    pub(crate) role: SurfaceRole,
    pub(crate) popup: Option<PopupInfo>,
    pub(crate) shared: Rc<GraphicsShared>,
    pub(crate) graphics: Option<Graphics>,
    pub(crate) app: AppInstance,
    pub(crate) logical_size: (u32, u32),
    pub(crate) buffer_size: (u32, u32),
    pub(crate) scale: f64,
    pub(crate) output: Option<ObjectId>,
    pub(crate) needs_redraw: bool,
    pub(crate) frame_pending: bool,
    pub(crate) draw_count: u64,
    pub(crate) configure_count: u64,
    /// The most recent window configure not yet applied.
    pub(crate) pending_configure: Option<(WindowConfigure, u32)>,
    pub(crate) appmenu: Option<OrgKdeKwinAppmenu>,
    pub(crate) appmenu_address: Option<AppMenuAddress>,
    pub(crate) menu_handle: Option<crate::app_menu::MenuHandle>,
    pub(crate) fractional: Option<WpFractionalScaleV1>,
    pub(crate) viewport: Option<WpViewport>,
}

/// A surface role provided by an extension (layer-shell, session-lock, etc.).
pub(crate) trait ExtensionRole: Debug {
    // Solo lo usa shell/ (feature `shell`); sin ella no se lee.
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;

    /// Tries to set this role as the implicit parent of an xdg_popup.
    /// Returns `true` when it knows how to do it.
    fn parent_popup(&self, _popup: &XdgPopup) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) enum SurfaceRole {
    // Solo lo usa shell/ (feature `shell`); sin ella no se lee.
    #[allow(dead_code)]
    Extension(Box<dyn ExtensionRole>),
    Window(Window),
    Popup(Popup),
    None,
}

impl SurfaceRole {
    /// Tries to set this role as the implicit parent of an xdg_popup.
    pub(crate) fn parent_popup(&self, popup: &XdgPopup) -> bool {
        match self {
            SurfaceRole::Extension(role) => role.parent_popup(popup),
            _ => false,
        }
    }

    /// Downcasts the extension role to a concrete type.
    // Solo lo usa shell/ (feature `shell`); sin ella no se lee.
    #[allow(dead_code)]
    pub(crate) fn as_extension_role<T: 'static>(&self) -> Option<&T> {
        match self {
            SurfaceRole::Extension(role) => role.as_any().downcast_ref::<T>(),
            _ => None,
        }
    }
}

impl SurfaceWindow {
    /// Schedules a compositor frame callback without marking the surface dirty.
    pub(crate) fn request_frame_callback(&mut self, qh: &QueueHandle<Runtime>) {
        if !self.frame_pending {
            self.frame_pending = true;
            let _callback: WlCallback = self.surface.frame(qh, self.surface.clone());
            self.surface.commit();
        }
    }

    pub(crate) fn request_redraw(&mut self, qh: &QueueHandle<Runtime>) {
        self.needs_redraw = true;
        self.request_frame_callback(qh);
    }

    pub(crate) fn configure(&mut self, size: (u32, u32), scale: f64) {
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        let buffer_size = buffer_size(size, scale);

        if self.graphics.is_none() {
            self.graphics = Some(Graphics::new(self.shared.clone()));
        }
        let Some(graphics) = self.graphics.as_mut() else {
            return;
        };
        if let Err(error) = graphics.configure(&self.surface, buffer_size, scale) {
            tracing::error!("BarrierKit: {error}");
            // Keep the previous size coherent, but do not push a 0×0 size into
            // the app on the first configure; that panics inside Torin.
            if self.buffer_size.0 > 0 && self.buffer_size.1 > 0 {
                self.app.resize(self.buffer_size, self.scale);
            }
            self.needs_redraw = true;
            return;
        }
        self.logical_size = size;
        self.scale = scale;
        self.buffer_size = buffer_size;
        if fractional_scale_enabled(self.fractional.is_some(), self.viewport.is_some(), scale) {
            self.surface.set_buffer_scale(1);
            if let Some(viewport) = &self.viewport {
                viewport.set_destination(size.0 as i32, size.1 as i32);
            }
        } else {
            self.surface.set_buffer_scale(scale.round().max(1.0) as i32);
        }
        self.app.resize(self.buffer_size, scale);
        if let SurfaceRole::Window(window) = &self.role {
            window
                .xdg_surface()
                .set_window_geometry(0, 0, size.0 as i32, size.1 as i32);
        }
        self.needs_redraw = true;
    }

    pub(crate) fn draw(&mut self) -> bool {
        let Some(graphics) = self.graphics.as_mut() else {
            return false;
        };
        if graphics.make_current().is_err() {
            return false;
        }
        self.draw_count += 1;
        tracing::debug!(
            target: "barrier_kit::resize",
            surface = ?self.surface.id(),
            draws = self.draw_count,
            role = ?self.role,
            "draw start"
        );
        self.app.layout();
        let background = if matches!(&self.role, SurfaceRole::Window(_)) {
            let resolved = *self.app.resolved.read();
            crate::theme::color(crate::theme::surface_background(
                &resolved,
                crate::theme::SurfaceKind::Toplevel,
            ))
        } else {
            Color::TRANSPARENT
        };
        {
            let Some(skia) = graphics.skia.as_mut() else {
                return false;
            };
            let canvas = skia.canvas();
            RenderPipeline {
                font_collection: &mut self.app.font_collection,
                font_manager: &self.app.font_manager,
                canvas,
                tree: &self.app.tree,
                scale_factor: self.scale,
                background,
            }
            .render();
        }
        graphics.shared.gr.borrow_mut().flush_and_submit();
        let swap_start = std::time::Instant::now();
        if graphics.swap().is_err() {
            return false;
        }
        let swap_us = swap_start.elapsed().as_micros() as u64;
        tracing::debug!(
            target: "barrier_kit::resize",
            surface = ?self.surface.id(),
            draws = self.draw_count,
            swap_us,
            "swap done (buffer committed)"
        );
        let _ = self.app.ticker_sender.send(());
        self.needs_redraw = false;
        true
    }
}

impl SurfaceWindow {
    /// Destroys the fractional-scale and viewport objects before the surface so
    /// the compositor sees them go away first.
    pub(crate) fn destroy_fractional(&mut self) {
        destroy_fractional_objects(&mut self.fractional, &mut self.viewport);
    }
}

impl Drop for SurfaceWindow {
    fn drop(&mut self) {
        self.destroy_fractional();
    }
}

pub(crate) fn destroy_fractional_objects(
    fractional: &mut Option<WpFractionalScaleV1>,
    viewport: &mut Option<WpViewport>,
) {
    if let Some(fractional) = fractional.take() {
        fractional.destroy();
    }
    if let Some(viewport) = viewport.take() {
        viewport.destroy();
    }
}

impl Runtime {
    /// Creates the fractional-scale and viewport objects for a surface when the
    /// compositor advertises the corresponding globals.
    pub(crate) fn create_fractional(
        &self,
        surface: &WlSurface,
    ) -> (Option<WpFractionalScaleV1>, Option<WpViewport>) {
        let fractional = self
            .fractional_manager
            .as_ref()
            .map(|manager| manager.get_fractional_scale(surface, &self.qh, surface.id()));
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(surface, &self.qh, surface.id()));
        (fractional, viewport)
    }
}

impl CompositorHandler for Runtime {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: &WlSurface,
        scale_factor: i32,
    ) {
        let Some(index) = self.surface_for(surface) else {
            return;
        };
        // While fractional scaling is active the compositor drives the scale
        // through wp_fractional_scale_v1.preferred_scale; integer scales are
        // ignored so the buffer and viewport stay in sync.
        if self.surfaces[index].fractional.is_some() {
            return;
        }
        let scale = scale_factor.max(1) as f64;
        let window = &mut self.surfaces[index];
        if window.scale != scale {
            window.scale = scale;
            if window.logical_size.0 > 0 {
                let size = window.logical_size;
                window.configure(size, scale);
            }
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, surface: &WlSurface, _: u32) {
        let Some(index) = self.surface_for(surface) else {
            return;
        };
        self.surfaces[index].frame_pending = false;
        // Collapse any window configures that arrived since the last frame.
        self.apply_pending_window_configures(qh);
        if self.surfaces[index].needs_redraw {
            let _ = self.surfaces[index].draw();
        }
        if self.surfaces[index].needs_redraw {
            self.surfaces[index].request_redraw(qh);
        }
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

/// Converts a fractional-scale wire value (120 = 1.0) to a logical scale factor.
pub(crate) fn scale_from_wire(scale: u32) -> f64 {
    scale as f64 / 120.0
}

/// Computes the buffer size for a logical surface size and scale.
pub(crate) fn buffer_size(logical: (u32, u32), scale: f64) -> (u32, u32) {
    (
        (logical.0 as f64 * scale).round().max(1.0) as u32,
        (logical.1 as f64 * scale).round().max(1.0) as u32,
    )
}

/// Returns true when the surface should use fractional scaling.
///
/// Both the fractional-scale manager and viewporter must be present; the
/// protocol is only useful when the compositor can scale the buffer back down.
pub(crate) fn fractional_scale_enabled(
    has_fractional: bool,
    has_viewporter: bool,
    _scale: f64,
) -> bool {
    has_fractional && has_viewporter
}

pub(crate) fn physical_point(position: (f64, f64), scale: f64) -> CursorPoint {
    CursorPoint::from((position.0 * scale, position.1 * scale))
}
impl Dispatch<WpFractionalScaleManagerV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &WpFractionalScaleManagerV1,
        _: <WpFractionalScaleManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}

impl Dispatch<WpFractionalScaleV1, ObjectId, Runtime> for Runtime {
    fn event(
        state: &mut Runtime,
        _: &WpFractionalScaleV1,
        event: FractionalScaleEvent,
        surface_id: &ObjectId,
        _: &Connection,
        qh: &QueueHandle<Runtime>,
    ) {
        let FractionalScaleEvent::PreferredScale { scale } = event else {
            return;
        };
        let scale = scale_from_wire(scale);
        let Some(index) = state.surface_index_by_id(surface_id) else {
            return;
        };
        let window = &mut state.surfaces[index];
        window.scale = scale;
        if window.logical_size.0 > 0 {
            let size = window.logical_size;
            window.configure(size, scale);
        }
        window.request_redraw(qh);
    }
}

impl Dispatch<WpViewporter, (), Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &WpViewporter,
        _: <WpViewporter as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}

impl Dispatch<WpViewport, ObjectId, Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &WpViewport,
        _: <WpViewport as Proxy>::Event,
        _: &ObjectId,
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}
