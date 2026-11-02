use freya_core::{
    integration::AppComponent,
    prelude::{State, WritableUtils},
};
use sctk::{
    reexports::{
        calloop::{EventLoop, ping::Ping},
        calloop_wayland_source::WaylandSource,
        client::{Connection, Proxy, QueueHandle, backend::ObjectId},
        protocols::xdg::shell::client::xdg_toplevel,
    },
    shell::{
        WaylandSurface,
        xdg::window::{DecorationMode, Window, WindowConfigure, WindowDecorations, WindowHandler},
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;

use super::{
    RunError,
    app::AppInstance,
    popup::Popups,
    runtime::{Connected, Runtime, WindowContext, insert_ping, make_sender},
    surface::{SurfaceRole, SurfaceWindow},
    timer::{PingWake, Timers, WakeHandle},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decorations {
    #[default]
    Client,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationResponse {
    Client,
    Server,
}

pub type MenuActionHandler = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone)]
pub struct WindowOptions {
    pub title: String,
    pub app_name: String,
    pub app_id: Option<String>,
    pub decorations: Decorations,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub parent: Option<ObjectId>,
    pub menu: Option<crate::app_menu::AppMenu>,
    pub on_menu_action: Option<MenuActionHandler>,
}

impl std::fmt::Debug for WindowOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowOptions")
            .field("title", &self.title)
            .field("app_name", &self.app_name)
            .field("app_id", &self.app_id)
            .field("decorations", &self.decorations)
            .field("size", &self.size)
            .field("min_size", &self.min_size)
            .field("max_size", &self.max_size)
            .field("parent", &self.parent)
            .field("menu", &self.menu)
            .field("on_menu_action", &self.on_menu_action.is_some())
            .finish()
    }
}

impl WindowOptions {
    pub fn new(title: impl Into<String>, app_name: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_name: app_name.into(),
            app_id: None,
            decorations: Decorations::Client,
            size: (640, 420),
            min_size: None,
            max_size: None,
            parent: None,
            menu: None,
            on_menu_action: None,
        }
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    pub fn decorations(mut self, decorations: Decorations) -> Self {
        self.decorations = decorations;
        self
    }

    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.size = size;
        self
    }

    pub fn min_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.min_size = size;
        self
    }

    pub fn max_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.max_size = size;
        self
    }

    pub fn parent(mut self, parent: Option<ObjectId>) -> Self {
        self.parent = parent;
        self
    }

    pub fn menu(mut self, menu: crate::app_menu::AppMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    pub fn on_menu_action(mut self, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_menu_action = Some(Arc::new(callback));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStatus {
    pub activated: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub decoration: DecorationResponse,
    pub fixed: bool,
}

impl Default for WindowStatus {
    fn default() -> Self {
        Self {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed: false,
        }
    }
}

#[derive(Clone)]
pub struct WindowControls {
    pub(super) queue: Rc<RefCell<Vec<WindowCommand>>>,
    pub(super) ping: Ping,
    pub(super) pointer_serial: Rc<std::cell::Cell<u32>>,
    pub status: State<WindowStatus>,
    pub title: String,
    pub fixed: bool,
    pub(super) surface_id: ObjectId,
    pub focus_bar: State<bool>,
    pub focus_overlay: bool,
    pub focus_bar_height: State<f32>,
}

pub(crate) fn window_activated() -> bool {
    freya_core::prelude::try_consume_root_context::<WindowControls>()
        .is_none_or(|controls| controls.status.read().activated)
}

impl WindowControls {
    pub fn traffic_lights_shown(&self) -> bool {
        !self.status.read().fullscreen || *self.focus_bar.read()
    }

    pub fn titlebar_overlaid(&self) -> bool {
        self.focus_overlay && self.status.read().fullscreen
    }

    pub fn fullscreen_content_top(&self) -> f32 {
        if self.titlebar_overlaid() {
            *self.focus_bar_height.read() + vow_design::FULLSCREEN_GLASS.content_top
        } else {
            0.0
        }
    }

    pub fn titlebar_overlay_top(&self) -> Option<f32> {
        (self.titlebar_overlaid() && *self.focus_bar.read()).then(|| *self.focus_bar_height.read())
    }

    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn present(&self, activation_token: String) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::Present(activation_token));
        self.ping.ping();
    }

    pub fn close(&self) {
        self.queue.borrow_mut().push(WindowCommand::Close);
        self.ping.ping();
    }

    pub fn minimize(&self) {
        self.queue.borrow_mut().push(WindowCommand::Minimize);
        self.ping.ping();
    }

    pub fn toggle_maximize(&self) {
        self.queue.borrow_mut().push(WindowCommand::ToggleMaximize);
        self.ping.ping();
    }

    pub fn toggle_vertical_maximize(&self) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::ToggleVerticalMaximize);
        self.ping.ping();
    }

    pub fn toggle_fullscreen(&self) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::ToggleFullscreen);
        self.ping.ping();
    }

    pub fn move_window(&self) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::Move(self.pointer_serial.get()));
        self.ping.ping();
    }

    pub fn resize(&self, edge: WindowEdge) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::Resize(self.pointer_serial.get(), edge));
        self.ping.ping();
    }
}

pub(crate) enum WindowCommand {
    Close,
    Minimize,
    ToggleMaximize,
    ToggleVerticalMaximize,
    ToggleFullscreen,
    Move(u32),
    Resize(u32, WindowEdge),
    Present(String),
}

pub(super) fn resize_edge(edge: WindowEdge) -> xdg_toplevel::ResizeEdge {
    match edge {
        WindowEdge::Top => xdg_toplevel::ResizeEdge::Top,
        WindowEdge::Bottom => xdg_toplevel::ResizeEdge::Bottom,
        WindowEdge::Left => xdg_toplevel::ResizeEdge::Left,
        WindowEdge::Right => xdg_toplevel::ResizeEdge::Right,
        WindowEdge::TopLeft => xdg_toplevel::ResizeEdge::TopLeft,
        WindowEdge::TopRight => xdg_toplevel::ResizeEdge::TopRight,
        WindowEdge::BottomLeft => xdg_toplevel::ResizeEdge::BottomLeft,
        WindowEdge::BottomRight => xdg_toplevel::ResizeEdge::BottomRight,
    }
}
pub(super) fn clamp_toplevel_size_dimension(value: u32) -> u32 {
    if value > i32::MAX as u32 { 0 } else { value }
}
pub fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
    let Connected {
        conn,
        event_queue,
        runtime: mut shell,
        ..
    } = Runtime::connect()?;
    shell
        .create_window(app, options)
        .ok_or(RunError::MissingGlobal("xdg_wm_base"))?;

    let mut event_loop: EventLoop<Runtime> = EventLoop::try_new()
        .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut shell)?;
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::EventLoop("inserting the Wayland source"))?;
    let signal = event_loop.get_signal();
    shell.loop_signal = Some(signal);
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
impl Runtime {
    pub(super) fn create_window(
        &mut self,
        app: AppComponent,
        options: WindowOptions,
    ) -> Option<WindowControls> {
        let xdg_shell = self.xdg_shell.as_ref()?;
        let qh = &self.qh;
        let surface = self.compositor.create_surface(qh);
        let decorations = match options.decorations {
            Decorations::Client => WindowDecorations::RequestClient,
            Decorations::Server => WindowDecorations::RequestServer,
        };
        let window = xdg_shell.create_window(surface.clone(), decorations, qh);
        let app_id = options
            .app_id
            .clone()
            .unwrap_or_else(|| format!("{}{}", vow_design::NATIVE_APP_ID_PREFIX, options.app_name));
        window.set_app_id(app_id);
        window.set_title(options.title.clone());
        if let Some(min) = options.min_size {
            window.set_min_size(Some((
                clamp_toplevel_size_dimension(min.0),
                clamp_toplevel_size_dimension(min.1),
            )));
        }
        if let Some(max) = options.max_size {
            window.set_max_size(Some((
                clamp_toplevel_size_dimension(max.0),
                clamp_toplevel_size_dimension(max.1),
            )));
        }
        if let Some(parent_id) = options.parent.clone()
            && let Some(parent_window) = self.surfaces.iter().find_map(|s| {
                if s.surface.id() == parent_id
                    && let SurfaceRole::Window(parent) = &s.role
                {
                    return Some(parent);
                }
                None
            })
        {
            window.set_parent(Some(parent_window));
        }
        window.commit();

        let menu = options.menu.clone();
        let mut menu_handle: Option<crate::app_menu::MenuHandle> = None;
        if let Some(menu) = menu {
            let handle = crate::app_menu::MenuHandle::new(
                menu.clone(),
                Some(WakeHandle::from_ping(self.wake.ping.clone())),
            );
            if self.appmenu_manager.is_some() {
                let surface_id = surface.id();
                let pending_addresses = self.pending_menu_addresses.clone();
                let wake = self.wake.ping.clone();
                handle.set_address_callback(move |address| {
                    pending_addresses
                        .lock()
                        .expect("pending menu addresses poisoned")
                        .push((surface_id.clone(), address));
                    wake.ping();
                });
                crate::app_menu::server::register(handle.clone());
            }
            menu_handle = Some(handle);
        }

        let appmenu_object: Option<OrgKdeKwinAppmenu> = None;

        let queue: Rc<RefCell<Vec<WindowCommand>>> = Rc::new(RefCell::new(Vec::new()));
        let pointer_serial = Rc::new(std::cell::Cell::new(0));
        let fixed = options.min_size.is_some() && options.min_size == options.max_size;
        let decoration = match options.decorations {
            Decorations::Client => DecorationResponse::Client,
            Decorations::Server if self.decoration_manager => DecorationResponse::Server,
            Decorations::Server => {
                if !self.decoration_warned {
                    self.decoration_warned = true;
                    tracing::info!(
                        "zxdg_decoration_manager_v1 no disponible; uso decoración Client (CSD)"
                    );
                }
                DecorationResponse::Client
            }
        };
        let status = State::create_global(WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration,
            fixed,
        });
        let surface_id = surface.id();
        let controls = WindowControls {
            queue: queue.clone(),
            ping: self.ping.clone(),
            pointer_serial,
            status,
            title: options.title.clone(),
            fixed,
            surface_id,
            focus_bar: State::create_global(false),
            focus_overlay: self
                .focus_manager
                .as_ref()
                .is_some_and(|manager| manager.version() >= 2),
            focus_bar_height: State::create_global(vow_design::MENUBAR_HEIGHT as f32),
        };
        if let Some(manager) = &self.focus_manager {
            let notice = manager.get_focus_surface(&surface, qh, surface.id());
            if controls.focus_overlay {
                notice.set_overlay();
            }
        }
        self.window_contexts.push(WindowContext {
            controls: controls.clone(),
            queue,
        });

        let waker = futures_util::task::waker(Arc::new(PingWake(self.ping.clone())));
        let sender = make_sender(self.queue.clone(), self.ping.clone(), surface.id());
        let popups = Popups {
            queue: self.popup_commands.clone(),
            ping: self.ping.clone(),
            next_id: self.popup_next_id.clone(),
            last_serial: self.last_input_serial.clone(),
            owner: surface.id(),
            current: None,
            opens_left: Rc::new(Cell::new(false)),
            scroll_shift: Rc::new(Cell::new(0.0)),
        };
        let app_instance = AppInstance::new(
            app,
            1.0,
            options.size,
            sender,
            waker,
            Some(controls.clone()),
            Timers {
                queue: self.timers.clone(),
                ping: self.ping.clone(),
                wake: self.wake.clone(),
            },
            popups,
            None,
            self.graphics.font_collection.clone(),
            self.graphics.font_manager.clone(),
            options.menu.clone(),
            options.on_menu_action.clone(),
            self.resolved,
            self.clipboard.provider(),
            self.defaults_revision,
            |r| {
                if let Some(hooks) = self.extension_hooks() {
                    (hooks.provide_contexts)(self, r);
                }
            },
        );
        let (fractional, viewport) = self.create_fractional(&surface);
        self.surfaces.push(SurfaceWindow {
            surface,
            role: SurfaceRole::Window(window),
            popup: None,
            shared: self.graphics.clone(),
            graphics: None,
            app: app_instance,
            logical_size: options.size,
            buffer_size: (0, 0),
            scale: 1.0,
            output: None,
            needs_redraw: true,
            frame_pending: false,
            draw_count: 0,
            opaque_size: None,
            configure_count: 0,
            pending_configure: None,
            appmenu: appmenu_object,
            appmenu_address: None,
            menu_handle,
            fractional,
            viewport,
        });
        Some(controls)
    }

    pub(super) fn process_window_commands(&mut self) {
        let contexts = self.window_contexts.clone();
        for context in contexts {
            let commands = std::mem::take(&mut *context.queue.borrow_mut());
            for command in commands {
                if let WindowCommand::Close = command {
                    self.close_window(context.controls.surface_id.clone());
                    continue;
                }
                let Some(index) = self.surfaces.iter().position(|surface| {
                    matches!(surface.role, SurfaceRole::Window(_))
                        && surface.surface.id() == context.controls.surface_id
                }) else {
                    continue;
                };
                let SurfaceRole::Window(window) = &self.surfaces[index].role else {
                    continue;
                };
                match command {
                    WindowCommand::Minimize => window.set_minimized(),
                    WindowCommand::ToggleMaximize | WindowCommand::ToggleVerticalMaximize => {
                        if context.controls.status.read().maximized {
                            window.unset_maximized();
                        } else {
                            window.set_maximized();
                        }
                    }
                    WindowCommand::ToggleFullscreen => {
                        if context.controls.status.read().fullscreen {
                            window.unset_fullscreen();
                        } else {
                            window.set_fullscreen(None);
                        }
                    }
                    WindowCommand::Move(serial) => {
                        if let Some(seat) = &self.seat_handle {
                            window.move_(seat, serial);
                        }
                    }
                    WindowCommand::Resize(serial, edge) => {
                        if let Some(seat) = &self.seat_handle {
                            window.resize(seat, serial, resize_edge(edge));
                        }
                    }
                    WindowCommand::Present(token) => {
                        if let Some(activation) = &self.activation {
                            activation.activate(token, window.wl_surface());
                        }
                    }
                    WindowCommand::Close => unreachable!(),
                }
                self.surfaces[index].needs_redraw = true;
            }
        }
    }
}

impl WindowHandler for Runtime {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        self.close_window(window.wl_surface().id());
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        serial: u32,
    ) {
        let Some(index) = self.surface_for(window.wl_surface()) else {
            return;
        };
        let size = (
            configure.new_size.0.map_or(0, |v| v.get()),
            configure.new_size.1.map_or(0, |v| v.get()),
        );
        tracing::debug!(
            target: "barrier_kit::resize",
            surface = ?self.surfaces[index].surface.id(),
            serial,
            width = size.0,
            height = size.1,
            resizing = configure.is_resizing(),
            "configure received"
        );
        self.surfaces[index].pending_configure = Some((configure, serial));
        self.surfaces[index].needs_redraw = true;
        self.surfaces[index].request_frame_callback(qh);
    }
}
impl Runtime {
    pub(super) fn apply_pending_window_configures(&mut self, qh: &QueueHandle<Self>) {
        for index in 0..self.surfaces.len() {
            let Some((configure, serial)) = self.surfaces[index].pending_configure.take() else {
                continue;
            };
            let scale = self.surfaces[index].scale;
            let size = (
                configure.new_size.0.map_or(0, |value| value.get()),
                configure.new_size.1.map_or(0, |value| value.get()),
            );
            tracing::debug!(
                target: "barrier_kit::resize",
                surface = ?self.surfaces[index].surface.id(),
                serial,
                width = size.0,
                height = size.1,
                "apply pending configure"
            );
            let size = if size.0 > 0 && size.1 > 0 {
                size
            } else {
                self.surfaces[index].logical_size
            };
            let size_changed = size != self.surfaces[index].logical_size;
            self.surfaces[index].configure_count += 1;
            let count = self.surfaces[index].configure_count;
            if count.is_multiple_of(30) {
                tracing::info!(
                    surface = ?self.surfaces[index].surface.id(),
                    configures = count,
                    swapchains = self.surfaces[index].shared.swapchain_creations.get(),
                    "BarrierKit: resize configuration count"
                );
            }
            self.surfaces[index].configure(size, scale);
            let surface_id = self.surfaces[index].surface.id();
            let requested_server = self
                .window_contexts
                .iter()
                .find(|ctx| ctx.controls.surface_id == surface_id)
                .map(|ctx| ctx.controls.status.read().decoration == DecorationResponse::Server)
                .unwrap_or(false);
            let decoration = if requested_server && self.decoration_manager {
                DecorationResponse::Server
            } else {
                match configure.decoration_mode {
                    DecorationMode::Server => DecorationResponse::Server,
                    DecorationMode::Client => DecorationResponse::Client,
                }
            };
            if let Some(context) = self
                .window_contexts
                .iter()
                .find(|ctx| ctx.controls.surface_id == surface_id)
            {
                if requested_server
                    && decoration == DecorationResponse::Client
                    && !self.decoration_warned
                {
                    self.decoration_warned = true;
                    tracing::info!(
                        "the compositor did not grant server-side decorations; drawing the kit titlebar"
                    );
                }
                let mut status = context.controls.status;
                status.set(WindowStatus {
                    activated: configure.is_activated(),
                    maximized: configure.is_maximized(),
                    fullscreen: configure.is_fullscreen(),
                    decoration,
                    fixed: context.controls.fixed,
                });
            }
            if !size_changed && let SurfaceRole::Window(window) = &self.surfaces[index].role {
                window.commit();
            }
            if self.surfaces[index].draw(&self.compositor) && self.surfaces[index].needs_redraw {
                self.surfaces[index].request_redraw(qh);
            }
        }
    }
}

impl Runtime {
    fn close_window(&mut self, surface_id: ObjectId) {
        let window_count = self
            .surfaces
            .iter()
            .filter(|s| matches!(s.role, SurfaceRole::Window(_)))
            .count();
        if window_count <= 1 {
            self.needs_exit = true;
        }
        if let Some(context) = self
            .window_contexts
            .iter()
            .find(|ctx| ctx.controls.surface_id == surface_id)
        {
            let mut status = context.controls.status;
            status.set(WindowStatus {
                activated: false,
                ..*status.read()
            });
        }
        if !self.needs_exit
            && let Some(index) = self.surfaces.iter().position(|s| {
                matches!(s.role, SurfaceRole::Window(_)) && s.surface.id() == surface_id
            })
        {
            if let Some(address) = self.surfaces[index].appmenu_address.take() {
                crate::app_menu::server::unregister(&address.path);
            }
            if let Some(appmenu) = self.surfaces[index].appmenu.take() {
                appmenu.release();
            }
            let mut role = SurfaceRole::None;
            std::mem::swap(&mut self.surfaces[index].role, &mut role);
            drop(role);
            self.surfaces.remove(index);
            self.window_contexts
                .retain(|ctx| ctx.controls.surface_id != surface_id);
        }
    }
}
