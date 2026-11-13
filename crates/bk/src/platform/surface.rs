use std::{fmt::Debug, rc::Rc};

#[cfg(feature = "shell")]
use sctk::reexports::protocols::xdg::shell::client::xdg_popup::XdgPopup;

use freya_core::{integration::RenderPipeline, prelude::Color};

use super::RunError;
use sctk::{
    compositor::{CompositorHandler, CompositorState, Region},
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
    glass::Rects,
    popup::PopupInfo,
    runtime::{Runtime, WindowContext},
    vulkan::{Graphics, GraphicsShared},
};

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
    failing: bool,
    pub(crate) opaque_region: Option<((u32, u32), Rects)>,
    pub(crate) glass_holes: Rects,
    pub(crate) background_alpha: f32,
    pub(crate) pending_configure: Option<(WindowConfigure, u32)>,
    pub(crate) appmenu: Option<OrgKdeKwinAppmenu>,
    pub(crate) menu_handle: Option<crate::main_menu::MenuHandle>,
    pub(crate) status_handle: Option<crate::main_menu::MenuHandle>,
    pub(crate) fractional: Option<WpFractionalScaleV1>,
    pub(crate) viewport: Option<WpViewport>,
    pub(crate) context: Option<WindowContext>,
}

#[cfg(feature = "shell")]
pub(crate) trait ExtensionRole: Debug {
    fn as_any(&self) -> &dyn std::any::Any;

    fn parent_popup(&self, _popup: &XdgPopup) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) enum SurfaceRole {
    #[cfg(feature = "shell")]
    Extension(Box<dyn ExtensionRole>),
    Window(Window),
    Popup(Popup),
}

#[cfg(feature = "shell")]
impl SurfaceRole {
    pub(crate) fn parent_popup(&self, popup: &XdgPopup) -> bool {
        match self {
            SurfaceRole::Extension(role) => role.parent_popup(popup),
            _ => false,
        }
    }

    pub(crate) fn as_extension_role<T: 'static>(&self) -> Option<&T> {
        match self {
            SurfaceRole::Extension(role) => role.as_any().downcast_ref::<T>(),
            _ => None,
        }
    }
}

#[cfg(debug_assertions)]
fn simulated_device_loss(draw_count: u64) -> bool {
    static LOSE_AT: std::sync::LazyLock<Option<u64>> = std::sync::LazyLock::new(|| {
        std::env::var("BARRIERKIT_LOSE_DEVICE_AT")
            .ok()?
            .parse()
            .ok()
    });
    static FIRED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    LOSE_AT.is_some_and(|at| draw_count >= at)
        && !FIRED.swap(true, std::sync::atomic::Ordering::Relaxed)
}

impl SurfaceWindow {
    pub(crate) fn request_frame_callback(&mut self, qh: &QueueHandle<Runtime>) {
        // ext-session-lock-v1: a commit without a buffer is a protocol error (null_buffer).
        if !self.frame_pending && self.draw_count > 0 {
            self.frame_pending = true;
            let _callback: WlCallback = self.surface.frame(qh, self.surface.clone());
            self.surface.commit();
        }
    }

    pub(crate) fn request_redraw(&mut self, qh: &QueueHandle<Runtime>) {
        self.needs_redraw = true;
        self.request_frame_callback(qh);
    }

    pub(crate) fn draw_after_configure(
        &mut self,
        compositor: &CompositorState,
        qh: &QueueHandle<Runtime>,
    ) {
        // An acked size must arrive with a buffer: an empty commit would apply it to the old one
        // (wl_surface.invalid_size, ext_session_lock_surface_v1.dimensions_mismatch).
        // A first draw that fails gets no frame callback to retry it; one swapchain recreation is free.
        if (self.draw_count == 0 || self.needs_redraw)
            && !self.draw(compositor)
            && self.draw_count == 0
        {
            let _ = self.draw(compositor);
        }
        if self.needs_redraw {
            self.request_frame_callback(qh);
        }
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
        if let Err(error) = graphics.configure(&self.surface, buffer_size) {
            tracing::error!("BarrierKit: {error}");
            if self.buffer_size.0 > 0 && self.buffer_size.1 > 0 {
                self.app.resize(self.buffer_size, self.scale);
            }
            self.needs_redraw = true;
            return;
        }
        self.logical_size = size;
        self.scale = scale;
        self.buffer_size = buffer_size;
        if self.fractional.is_some() && self.viewport.is_some() {
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

    pub(crate) fn draw(&mut self, compositor: &CompositorState) -> bool {
        let _draw = tracing::trace_span!("draw").entered();
        #[cfg(debug_assertions)]
        if simulated_device_loss(self.draw_count) {
            return self.draw_failed(RunError::DeviceLost);
        }
        let hud = self
            .context
            .as_ref()
            .is_some_and(|context| context.controls.hud);
        let background = if hud {
            crate::theme::color(ctdesign::HUD_BACKGROUND)
        } else if matches!(&self.role, SurfaceRole::Window(_)) {
            let resolved = *self.app.resolved.read();
            let base =
                crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel);
            crate::theme::color(ctdesign::Rgba {
                a: base.a * self.background_alpha,
                ..base
            })
        } else {
            Color::TRANSPARENT
        };
        self.declare_opaque(compositor, background);
        let Some(graphics) = self.graphics.as_mut() else {
            return false;
        };
        if let Err(error) = tracing::trace_span!("acquire").in_scope(|| graphics.acquire()) {
            return self.draw_failed(error);
        }
        tracing::debug!(
            target: "bk::resize",
            surface = ?self.surface.id(),
            draws = self.draw_count + 1,
            role = ?self.role,
            "draw start"
        );
        {
            let _layout = tracing::trace_span!("layout").entered();
            self.app.layout();
            self.app.settle_sized_reactions();
        }
        {
            let _render = tracing::trace_span!("render").entered();
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
        let swap_start = std::time::Instant::now();
        if let Err(error) = tracing::trace_span!("present").in_scope(|| graphics.present()) {
            return self.draw_failed(error);
        }
        self.draw_count += 1;
        #[cfg(feature = "tracy")]
        crate::logging::frame_mark();
        let swap_us = swap_start.elapsed().as_micros() as u64;
        tracing::debug!(
            target: "bk::resize",
            surface = ?self.surface.id(),
            draws = self.draw_count,
            swap_us,
            "swap done (buffer committed)"
        );
        self.app.ticker_sender.notify();
        self.needs_redraw = false;
        self.failing = false;
        true
    }

    // Vulkan: a lost surface needs a new VkSurfaceKHR and swapchain; a lost device ends the run.
    fn draw_failed(&mut self, error: RunError) -> bool {
        match error {
            RunError::SurfaceLost => {
                self.graphics = None;
                let (size, scale) = (self.logical_size, self.scale);
                self.configure(size, scale);
            }
            RunError::DeviceLost => self.shared.lost.set(true),
            _ => {}
        }
        if !self.failing && !matches!(error, RunError::Vulkan(super::vulkan::OUT_OF_DATE)) {
            tracing::warn!(%error, surface = ?self.surface.id(), "BarrierKit: drawing failed");
        }
        self.failing = true;
        false
    }
}

impl SurfaceWindow {
    fn declare_opaque(&mut self, compositor: &CompositorState, background: Color) {
        let opaque =
            (background.a() == u8::MAX).then(|| (self.logical_size, self.glass_holes.clone()));
        if opaque == self.opaque_region {
            return;
        }
        let region = opaque.as_ref().and_then(|((width, height), holes)| {
            let region = Region::new(compositor).ok()?;
            region.add(0, 0, *width as i32, *height as i32);
            for &(x, y, hole_width, hole_height) in holes {
                region.subtract(x, y, hole_width, hole_height);
            }
            Some(region)
        });
        self.surface
            .set_opaque_region(region.as_ref().map(Region::wl_region));
        self.opaque_region = opaque;
    }
}

impl Drop for SurfaceWindow {
    fn drop(&mut self) {
        // The swapchain must go before the role destroys the wl_surface (vkCreateWaylandSurfaceKHR).
        self.graphics = None;
        if let Some(fractional) = self.fractional.take() {
            fractional.destroy();
        }
        if let Some(viewport) = self.viewport.take() {
            viewport.destroy();
        }
        if let Some(notice) = self.context.take().and_then(|context| context.focus_notice) {
            notice.destroy();
        }
    }
}

impl Runtime {
    pub(crate) fn surface_window(
        &self,
        surface: WlSurface,
        role: SurfaceRole,
        app: AppInstance,
    ) -> SurfaceWindow {
        let fractional = self
            .fractional_manager
            .as_ref()
            .map(|manager| manager.get_fractional_scale(&surface, &self.qh, surface.id()));
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(&surface, &self.qh, surface.id()));
        SurfaceWindow {
            surface,
            role,
            popup: None,
            shared: self.graphics.clone(),
            graphics: None,
            app,
            logical_size: (0, 0),
            buffer_size: (0, 0),
            scale: 1.0,
            output: None,
            needs_redraw: true,
            frame_pending: false,
            draw_count: 0,
            failing: false,
            opaque_region: None,
            glass_holes: Vec::new(),
            background_alpha: 1.0,
            pending_configure: None,
            appmenu: None,
            menu_handle: None,
            status_handle: None,
            fractional,
            viewport,
            context: None,
        }
    }
}

impl CompositorHandler for Runtime {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        scale_factor: i32,
    ) {
        let Some(index) = self.surface_for(surface) else {
            return;
        };
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
                // xdg_surface: a buffer before the first configure is an error (unconfigured_buffer).
                if window.draw_count > 0 {
                    window.draw_after_configure(&self.compositor, qh);
                }
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
        self.apply_pending_window_configures();
        let fading = self.step_wallpaper_fade(index);
        if self.surfaces[index].needs_redraw {
            let _ = self.surfaces[index].draw(&self.compositor);
        }
        if fading {
            self.surfaces[index].needs_redraw = true;
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

pub(crate) fn scale_from_wire(scale: u32) -> f64 {
    scale as f64 / 120.0
}

pub(crate) fn buffer_size(logical: (u32, u32), scale: f64) -> (u32, u32) {
    (
        (logical.0 as f64 * scale).round().max(1.0) as u32,
        (logical.1 as f64 * scale).round().max(1.0) as u32,
    )
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
