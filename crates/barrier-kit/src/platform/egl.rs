use super::RunError;
use freya_engine::prelude::{
    ColorType, DirectContext, FontCollection, FontMgr, Format, FramebufferInfo, Interface,
    Surface as SkiaSurface, SurfaceOrigin, backend_render_targets, direct_contexts,
    wrap_backend_render_target,
};
use khronos_egl::{self as egl, DynamicInstance};
use sctk::reexports::client::{Proxy, protocol::wl_surface::WlSurface};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    rc::Rc,
};
use wayland_egl::WlEglSurface;

/// The Wayland EGL platform (`EGL_PLATFORM_WAYLAND_KHR`/`EXT`).
const EGL_PLATFORM_WAYLAND: egl::Enum = 0x31D8;

/// The EGL + Skia GL state shared by every surface in the process.
pub(crate) struct GraphicsShared {
    egl: DynamicInstance<egl::EGL1_5>,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    pub(crate) gr: RefCell<DirectContext>,
    pub(crate) font_collection: FontCollection,
    pub(crate) font_manager: FontMgr,
    /// Counter of `wl_egl_window` creations, for resize tests.
    pub(crate) egl_window_creations: Cell<usize>,
}

// Safety: EGL display/context handles are used only on the thread that created
// them (the calloop loop); the raw pointers are not shared across threads.
unsafe impl Send for GraphicsShared {}

impl GraphicsShared {
    pub(crate) fn new(display_ptr: *mut c_void) -> Result<Self, RunError> {
        // SAFETY: called once on the calloop thread; libEGL is present because
        // the compositor already initialized EGL. `load_required` validates the
        // minimum version (EGL 1.5) before returning.
        let egl = unsafe { DynamicInstance::<egl::EGL1_5>::load_required() }
            .map_err(|_| RunError::Egl("libEGL could not be loaded"))?;
        // Safety: the caller passes the live `wl_display*` of the connection.
        let display = unsafe {
            egl.get_platform_display(
                EGL_PLATFORM_WAYLAND,
                display_ptr,
                &[egl::NONE as egl::Attrib],
            )
        }
        .map_err(|_| RunError::Egl("eglGetPlatformDisplay failed"))?;
        egl.initialize(display)
            .map_err(|_| RunError::Egl("eglInitialize failed"))?;
        let config = egl
            .choose_first_config(
                display,
                &[
                    egl::SURFACE_TYPE,
                    egl::WINDOW_BIT,
                    egl::RED_SIZE,
                    8,
                    egl::GREEN_SIZE,
                    8,
                    egl::BLUE_SIZE,
                    8,
                    egl::ALPHA_SIZE,
                    8,
                    egl::RENDERABLE_TYPE,
                    egl::OPENGL_ES2_BIT,
                    egl::NONE,
                ],
            )
            .map_err(|_| RunError::Egl("eglChooseConfig failed"))?
            .ok_or(RunError::Egl("no suitable EGL config"))?;
        let context = egl
            .create_context(
                display,
                config,
                None,
                &[egl::CONTEXT_CLIENT_VERSION, 3, egl::NONE],
            )
            .map_err(|_| RunError::Egl("eglCreateContext failed"))?;
        // Skia's GL direct context needs a current EGL context to query the
        // driver; the window surface is made current later in `configure`.
        egl.make_current(display, None, None, Some(context))
            .map_err(|_| RunError::Egl("eglMakeCurrent(surfaceless) failed"))?;

        // The Skia GL interface loads every function through EGL.
        let interface = Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            egl.get_proc_address(name)
                .map_or(std::ptr::null(), |f| f as *const c_void)
        })
        .ok_or(RunError::Skia)?;
        let mut gr = direct_contexts::make_gl(interface, None).ok_or(RunError::Skia)?;
        gr.set_resource_cache_limit(128 * 1024 * 1024);

        Ok(Self {
            egl,
            display,
            config,
            context,
            gr: RefCell::new(gr),
            font_collection: crate::fonts::collection(),
            font_manager: crate::fonts::manager(),
            egl_window_creations: Cell::new(0),
        })
    }
}

impl Drop for GraphicsShared {
    fn drop(&mut self) {
        let _ = self.egl.make_current(self.display, None, None, None);
        self.gr.borrow_mut().abandon();
        let _ = self.egl.destroy_context(self.display, self.context);
    }
}

/// The per-surface EGL window/surface and Skia render target. The expensive
/// process-wide state (EGL context, `DirectContext`, fonts) lives in
/// [`GraphicsShared`].
pub(crate) struct Graphics {
    pub(crate) shared: Rc<GraphicsShared>,
    window: Option<WlEglSurface>,
    surface: Option<egl::Surface>,
    pub(crate) skia: Option<SkiaSurface>,
    size: (u32, u32),
    scale: f64,
}

// Safety: the EGLSurface is only used on the calloop thread.
unsafe impl Send for Graphics {}

impl Graphics {
    pub(crate) fn new(shared: Rc<GraphicsShared>) -> Self {
        Self {
            shared,
            window: None,
            surface: None,
            skia: None,
            size: (0, 0),
            scale: 1.0,
        }
    }

    /// Builds the `wl_egl_window` and EGL surface on a real configure, then
    /// recreates the Skia surface at the new size.
    ///
    /// Reuses the existing `wl_egl_window` via `resize` when only the size
    /// changed; only recreates it when the scale changed or the resize failed.
    pub(crate) fn configure(
        &mut self,
        surface: &WlSurface,
        size: (u32, u32),
        scale: f64,
    ) -> Result<(), RunError> {
        if size.0 == 0 || size.1 == 0 {
            return Ok(());
        }
        let shared = self.shared.clone();
        let size_changed = self.size != size;
        let scale_changed = self.scale != scale;
        let needs_window = self.window.is_none() || size_changed || scale_changed;
        if needs_window {
            if let Some(window) = self.window.as_ref() {
                // Try to resize the existing wl_egl_window without destroying
                // the EGL surface. dx/dy are zero because the Wayland surface
                // origin does not move.
                window.resize(size.0 as i32, size.1 as i32, 0, 0);
                let (w, h) = window.get_size();
                if w == size.0 as i32 && h == size.1 as i32 && !scale_changed {
                    self.size = size;
                } else {
                    self.recreate_window(surface, size)?;
                }
            } else {
                self.recreate_window(surface, size)?;
            }
        }

        let egl_surface = *self.surface.as_ref().expect("surface created above");
        shared
            .egl
            .make_current(
                shared.display,
                Some(egl_surface),
                Some(egl_surface),
                Some(shared.context),
            )
            .map_err(|_| RunError::Egl("eglMakeCurrent failed"))?;

        let fb_info = FramebufferInfo {
            fboid: 0,
            format: Format::RGBA8.into(),
            ..Default::default()
        };
        let render_target =
            backend_render_targets::make_gl((size.0 as i32, size.1 as i32), 0, 0, fb_info);
        self.skia = wrap_backend_render_target(
            &mut shared.gr.borrow_mut(),
            &render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        );
        self.scale = scale;
        Ok(())
    }

    /// Recreates the `wl_egl_window` and its EGL surface.
    fn recreate_window(&mut self, surface: &WlSurface, size: (u32, u32)) -> Result<(), RunError> {
        let shared = self.shared.clone();
        self.window = Some(
            WlEglSurface::new(surface.id(), size.0 as i32, size.1 as i32)
                .map_err(|_| RunError::Egl("wl_egl_window_create failed"))?,
        );
        shared
            .egl_window_creations
            .set(shared.egl_window_creations.get() + 1);
        if let Some(old) = self.surface.take() {
            let _ = shared.egl.destroy_surface(shared.display, old);
        }
        let window_ptr = self
            .window
            .as_ref()
            .map(|w| w.ptr() as *mut c_void)
            .expect("window created above");
        // Safety: the wl_egl_window is owned here and lives as long as the
        // EGL surface.
        let egl_surface = unsafe {
            shared
                .egl
                .create_window_surface(shared.display, shared.config, window_ptr, None)
        }
        .map_err(|_| RunError::Egl("eglCreateWindowSurface failed"))?;
        self.surface = Some(egl_surface);
        self.size = size;
        Ok(())
    }

    pub(crate) fn make_current(&self) -> Result<(), RunError> {
        let Some(surface) = self.surface else {
            return Err(RunError::Egl("no window surface"));
        };
        self.shared
            .egl
            .make_current(
                self.shared.display,
                Some(surface),
                Some(surface),
                Some(self.shared.context),
            )
            .map_err(|_| RunError::Egl("eglMakeCurrent failed"))
    }

    pub(crate) fn swap(&self) -> Result<(), RunError> {
        let Some(surface) = self.surface else {
            return Err(RunError::Egl("no window surface"));
        };
        self.shared
            .egl
            .swap_buffers(self.shared.display, surface)
            .map_err(|_| RunError::Egl("eglSwapBuffers failed"))
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        // shortcut: per-popup GPU cleanup is deferred to the shared context's
        // abandon() at process exit. Flushing here requires borrowing the
        // shared DirectContext, which panics if a frame is currently being
        // rendered, and free_gpu_resources() would free textures that other
        // surfaces still need. We only destroy the EGL surface so the
        // compositor sees popup_destroyed before the wl_egl_window is gone.
        if let Some(surface) = self.surface.take() {
            let _ = self
                .shared
                .egl
                .destroy_surface(self.shared.display, surface);
        }
        self.window = None;
    }
}
