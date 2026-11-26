use freya_core::{
    integration::AppComponent,
    prelude::{State, WritableUtils},
};
use sctk::{
    reexports::{
        client::{Connection, Proxy, QueueHandle},
        protocols::xdg::shell::client::xdg_toplevel,
    },
    shell::{
        WaylandSurface,
        xdg::window::{DecorationMode, Window, WindowConfigure, WindowDecorations, WindowHandler},
    },
};

use crate::platform::{
    ObjectId, RunError,
    app::{AppInstance, InstanceRole},
    linux::{dbusmenu::server::AddressCallback, window_style::wire},
    runtime::{Connected, Runtime, WindowContext},
    surface::{SurfaceRole, SurfaceWindow},
    timer::WakeHandle,
    window::{
        DecorationResponse, WindowCommand, WindowControls, WindowEdge, WindowOptions, WindowStatus,
        below_bar,
    },
};

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
pub(super) fn resize_edge_at(
    (x, y): (f64, f64),
    (width, height): (u32, u32),
) -> Option<WindowEdge> {
    let (width, height) = (f64::from(width), f64::from(height));
    let edge = f64::from(ctdesign::WINDOW_RESIZE.edge);
    let corner = f64::from(ctdesign::WINDOW_RESIZE.corner);
    let near = |value: f64, end: f64, band: f64| (value < band, value >= end - band);
    let (left, right) = near(x, width, edge);
    let (top, bottom) = near(y, height, edge);
    let (left_corner, right_corner) = near(x, width, corner);
    let (top_corner, bottom_corner) = near(y, height, corner);
    match () {
        _ if (top && left_corner) || (left && top_corner) => Some(WindowEdge::TopLeft),
        _ if (top && right_corner) || (right && top_corner) => Some(WindowEdge::TopRight),
        _ if (bottom && left_corner) || (left && bottom_corner) => Some(WindowEdge::BottomLeft),
        _ if (bottom && right_corner) || (right && bottom_corner) => Some(WindowEdge::BottomRight),
        _ if top => Some(WindowEdge::Top),
        _ if bottom => Some(WindowEdge::Bottom),
        _ if left => Some(WindowEdge::Left),
        _ if right => Some(WindowEdge::Right),
        _ => None,
    }
}

impl Runtime {
    pub(super) fn resize_edge(&self, index: usize, position: (f64, f64)) -> Option<WindowEdge> {
        let surface = &self.surfaces[index];
        let status = *surface.context.as_ref()?.controls.status.peek();
        let resizable = status.decoration == DecorationResponse::Client
            && !status.maximized
            && !status.fullscreen
            && !status.fixed;
        resizable
            .then(|| resize_edge_at(position, surface.logical_size))
            .flatten()
    }
}

#[derive(Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct SavedSize {
    width: u32,
    height: u32,
    maximized: bool,
}

fn window_state(app_id: &str, scene: Option<&str>) -> String {
    match scene {
        Some(scene) => format!("{app_id}.window.{scene}"),
        None => format!("{app_id}.window"),
    }
}

fn within(size: (u32, u32), min: Option<(u32, u32)>, max: Option<(u32, u32)>) -> (u32, u32) {
    let (low_w, low_h) = min.unwrap_or((1, 1));
    let unbounded = |limit: u32| if limit == 0 { u32::MAX } else { limit };
    let (high_w, high_h) = max.map_or((u32::MAX, u32::MAX), |(w, h)| (unbounded(w), unbounded(h)));
    (
        size.0.clamp(low_w, high_w.max(low_w)),
        size.1.clamp(low_h, high_h.max(low_h)),
    )
}

pub(crate) fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
    let Connected {
        conn,
        event_queue,
        runtime: mut shell,
        ..
    } = Runtime::connect()?;
    shell
        .create_window(app, options, None)
        .ok_or(RunError::MissingGlobal("xdg_wm_base"))?;

    shell.run(conn, event_queue, |_, _| {})
}

impl Runtime {
    pub(super) fn create_window(
        &mut self,
        app: AppComponent,
        mut options: WindowOptions,
        parent: Option<&ObjectId>,
    ) -> Option<WindowControls> {
        let parent_id = parent.cloned();
        let parent =
            parent
                .and_then(|id| self.surface_index_by_id(id))
                .and_then(|index| match &self.surfaces[index].role {
                    SurfaceRole::Window(window) => Some((
                        window.clone(),
                        self.surfaces[index].context.as_ref()?.app_id.clone(),
                    )),
                    _ => None,
                });
        let xdg_shell = self.xdg_shell.as_ref()?;
        let qh = &self.qh;
        let surface = self.compositor.create_surface(qh);
        let window = xdg_shell.create_window(surface.clone(), WindowDecorations::RequestClient, qh);
        let app_id = match &parent {
            Some((_, app_id)) => app_id.clone(),
            None => options.app_id.clone(),
        };
        let utility = parent.is_some();
        // A sheet is not a panel: it takes the keyboard and deactivates its parent, like AppKit.
        let panel = utility && !options.modal;
        if let Some((parent, _)) = &parent {
            window.set_parent(Some(parent));
        }
        // The xdg_dialog_v1 object lives as long as the toplevel; destroying it would drop the hint.
        if let Some(manager) = self
            .dialog_manager
            .as_ref()
            .filter(|_| options.modal && parent.is_some())
        {
            manager
                .get_xdg_dialog(window.xdg_toplevel(), qh, ())
                .set_modal();
        }
        if let Some(style) = self
            .window_style
            .as_ref()
            .filter(|style| style.version() >= 3)
            && panel
        {
            style.set_panel(window.wl_surface(), options.panel.flags());
        }
        let bar = options.window_style.inner_bar_height(false);
        options.min_size = below_bar(options.min_size, bar);
        options.max_size = below_bar(options.max_size, bar);
        let state = window_state(&app_id, options.scene);
        let saved: SavedSize = if utility {
            SavedSize::default()
        } else {
            crate::app::read_toml(&crate::app::defaults::restoration_file(&app_id, &state))
        };
        if saved.width > 0 && saved.height > 0 {
            options.size = within(
                (saved.width, saved.height),
                options.min_size,
                options.max_size,
            );
        }
        window.set_app_id(app_id.clone());
        window.set_title(options.title.clone());
        if saved.maximized {
            window.set_maximized();
        }
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
        window.commit();

        let menu = options.menu.clone();
        let mut menu_handle: Option<crate::main_menu::MenuHandle> = None;
        if let Some(menu) = menu {
            let handle = crate::main_menu::MenuHandle::new(
                menu.clone(),
                Some(WakeHandle::from_ping(self.wake.ping.clone())),
            );
            let on_address = self.appmenu_manager.is_some().then(|| {
                let surface_id = surface.id();
                let pending_addresses = self.pending_menu_addresses.clone();
                let wake = self.wake.ping.clone();
                Box::new(move |address| {
                    if let Ok(mut pending) = pending_addresses.lock() {
                        pending.push((surface_id.clone(), address));
                    }
                    wake.ping();
                }) as AddressCallback
            });
            crate::platform::linux::dbusmenu::server::register(handle.clone(), on_address);
            menu_handle = Some(handle);
        }

        let controls = WindowControls {
            focus_overlay: self
                .focus_manager
                .as_ref()
                .is_some_and(|manager| manager.version() >= 2),
            ..WindowControls::new(
                &options,
                DecorationResponse::Client,
                utility,
                surface.id(),
                self.ping.clone(),
                self.primary.clone(),
            )
        };
        let window_resolved = State::create_global(*self.resolved.peek());
        let focus_notice = self.focus_manager.as_ref().map(|manager| {
            let notice = manager.get_focus_surface(&surface, qh, surface.id());
            if controls.focus_overlay && notice.version() >= 2 {
                notice.set_overlay();
            }
            notice
        });

        let app_instance = AppInstance::new(
            &self.host(),
            surface.id(),
            app,
            1.0,
            options.size,
            InstanceRole::Window {
                controls: Box::new(controls.clone()),
                menu: menu_handle.clone(),
                theme: options.theme.clone(),
                resolved: window_resolved,
            },
            |runner| self.provide_extension_contexts(runner),
        );
        let mut window = self.surface_window(surface, SurfaceRole::Window(window), app_instance);
        window.logical_size = options.size;
        window.menu_handle = menu_handle;
        window.status_handle = options.status_item.map(|item| {
            let menu = crate::main_menu::MainMenu::new(vec![crate::main_menu::Menu::new(
                "",
                item.items.clone(),
            )]);
            let handle = crate::main_menu::MenuHandle::new(
                menu,
                Some(WakeHandle::from_ping(self.wake.ping.clone())),
            );
            crate::platform::linux::dbusmenu::server::register_status_item(handle.clone(), item);
            handle
        });
        window.context = Some(WindowContext {
            controls: controls.clone(),
            app_id,
            state,
            focus_notice,
            resolved: window_resolved,
            wallpaper: None,
            wallpaper_fade: None,
            parent: parent_id.filter(|_| utility),
            key_only_if_needed: panel && options.panel.key_only_if_needed,
            modal: utility && options.modal,
        });
        self.surfaces.push(window);
        Some(controls)
    }

    pub(super) fn process_window_commands(&mut self) {
        let pending: Vec<_> = self
            .surfaces
            .iter()
            .filter_map(|surface| surface.context.as_ref())
            .map(|context| {
                let commands = std::mem::take(&mut *context.controls.queue.borrow_mut());
                (context.controls.clone(), commands)
            })
            .collect();
        for (controls, commands) in pending {
            for command in commands {
                let id = &controls.surface_id;
                match command {
                    WindowCommand::Close => {
                        if !crate::document::intercept_close(id) {
                            self.close_window(id.clone());
                        }
                    }
                    WindowCommand::ImeCursor(area) => self.set_ime_cursor(Some(id.clone()), area),
                    WindowCommand::FieldCursor(area) => self.set_ime_cursor(None, area),
                    WindowCommand::Open(window) => {
                        let (app, options) = *window;
                        self.create_window(app, options, None);
                    }
                    WindowCommand::Sheet(window) => {
                        let (app, options) = *window;
                        self.create_window(app, options, Some(id));
                    }
                    WindowCommand::Fit(size) => self.fit_window(id, size),
                    WindowCommand::BackgroundAlpha(alpha) => {
                        if let Some(index) = self.surface_index_by_id(id) {
                            self.surfaces[index].background_alpha = alpha;
                            self.surfaces[index].needs_redraw = true;
                        }
                    }
                    command => {
                        let Some(index) = self.surface_index_by_id(id) else {
                            continue;
                        };
                        let SurfaceRole::Window(window) = &self.surfaces[index].role else {
                            continue;
                        };
                        match command {
                            WindowCommand::Minimize => window.set_minimized(),
                            WindowCommand::ToggleMaximize => {
                                if controls.status.read().maximized {
                                    window.unset_maximized();
                                } else {
                                    window.set_maximized();
                                }
                            }
                            WindowCommand::ToggleFullscreen => {
                                if controls.status.read().fullscreen {
                                    window.unset_fullscreen();
                                } else {
                                    window.set_fullscreen(None);
                                }
                            }
                            WindowCommand::Move(serial) => {
                                if let Some(seat) = &self.seat_handle {
                                    window.move_(seat, serial);
                                }
                                self.request_keyboard(index, serial);
                            }
                            WindowCommand::Resize(serial, edge) => {
                                if let Some(seat) = &self.seat_handle {
                                    window.resize(seat, serial, resize_edge(edge));
                                }
                            }
                            WindowCommand::Present(Some(token)) => {
                                if let Some(activation) = &self.activation {
                                    activation.activate(token, window.wl_surface());
                                }
                            }
                            WindowCommand::Present(None) => {
                                self.request_activation(window.wl_surface())
                            }
                            WindowCommand::MakeKey => {
                                self.request_activation_after_press(window.wl_surface())
                            }
                            WindowCommand::SetTitle(title) => window.set_title(title),
                            // The kit draws the title row on Linux, subtitle included.
                            WindowCommand::SetSubtitle(_subtitle) => {}
                            WindowCommand::Bar(bar) => {
                                if let Some(style) = &self.window_style {
                                    style.set_bar(window.wl_surface(), wire(bar as u32));
                                }
                            }
                            WindowCommand::Close
                            | WindowCommand::BackgroundAlpha(_)
                            | WindowCommand::Open(_)
                            | WindowCommand::Sheet(_)
                            | WindowCommand::Fit(_)
                            | WindowCommand::ImeCursor(_)
                            | WindowCommand::FieldCursor(_) => {}
                        }
                        self.surfaces[index].needs_redraw = true;
                    }
                }
            }
        }
    }
}

impl WindowHandler for Runtime {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        // AppKit: only its own buttons dismiss a sheet; closed from outside, its parent stays dimmed.
        let sheet = self
            .surface_for(window.wl_surface())
            .and_then(|index| self.surfaces[index].context.as_ref())
            .is_some_and(|context| context.modal);
        let id = window.wl_surface().id();
        if !sheet && !crate::document::intercept_close(&id) {
            self.close_window(id);
        }
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
            target: "bk::resize",
            surface = ?self.surfaces[index].surface.id(),
            serial,
            width = size.0,
            height = size.1,
            resizing = configure.is_resizing(),
            "configure received"
        );
        self.surfaces[index].pending_configure = Some((configure, serial));
        self.surfaces[index].needs_redraw = true;
        // xdg-shell: SCTK has acked, so the next commit must carry this configure's geometry.
        if self.surfaces[index].draw_count == 0 || !self.surfaces[index].frame_pending {
            self.apply_pending_window_configures();
        }
        if self.surfaces[index].pending_configure.is_none() {
            self.surfaces[index].draw_after_configure(&self.compositor, qh);
        }
    }
}
impl Runtime {
    pub(super) fn apply_pending_window_configures(&mut self) {
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
                target: "bk::resize",
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
            self.surfaces[index].configure(size, scale);
            let decoration = match configure.decoration_mode {
                DecorationMode::Server => DecorationResponse::Server,
                DecorationMode::Client => DecorationResponse::Client,
            };
            if let Some(context) = &self.surfaces[index].context {
                let mut status = context.controls.status;
                let fixed = status.peek().fixed;
                status.set(WindowStatus {
                    activated: configure.is_activated(),
                    maximized: configure.is_maximized(),
                    fullscreen: configure.is_fullscreen(),
                    decoration,
                    fixed,
                });
            }
            if !size_changed && let SurfaceRole::Window(window) = &self.surfaces[index].role {
                window.commit();
            }
            let _ = self.surfaces[index].draw(&self.compositor);
        }
    }
}

impl Runtime {
    fn fit_window(&mut self, surface_id: &ObjectId, size: (u32, u32)) {
        let Some(index) = self.surface_index_by_id(surface_id) else {
            return;
        };
        if self.surfaces[index].logical_size == size {
            return;
        }
        if let SurfaceRole::Window(window) = &self.surfaces[index].role {
            let fixed = Some((
                clamp_toplevel_size_dimension(size.0),
                clamp_toplevel_size_dimension(size.1),
            ));
            window.set_min_size(fixed);
            window.set_max_size(fixed);
            if let Some(context) = &self.surfaces[index].context {
                let mut status = context.controls.status;
                let shown = *status.peek();
                status.set(WindowStatus {
                    fixed: true,
                    ..shown
                });
            }
        }
        let scale = self.surfaces[index].scale;
        self.surfaces[index].configure(size, scale);
        if self.surfaces[index].draw_count > 0 && !self.surfaces[index].frame_pending {
            let qh = self.qh.clone();
            self.surfaces[index].draw_after_configure(&self.compositor, &qh);
        }
    }

    fn close_window(&mut self, surface_id: ObjectId) {
        let Some(index) = self.surface_index_by_id(&surface_id) else {
            return;
        };
        let main = |surface: &SurfaceWindow| {
            matches!(surface.role, SurfaceRole::Window(_))
                && surface
                    .context
                    .as_ref()
                    .is_none_or(|context| context.parent.is_none())
        };
        if main(&self.surfaces[index]) && self.surfaces.iter().filter(|s| main(s)).count() <= 1 {
            self.needs_exit = true;
        }
        if let Some(context) = &self.surfaces[index].context
            && context.parent.is_none()
        {
            let mut status = context.controls.status;
            let shown = *status.peek();
            let state = crate::app::defaults::restoration_file(&context.app_id, &context.state);
            let previous: SavedSize = crate::app::read_toml(&state);
            let saved = match (shown.maximized, shown.fullscreen) {
                (_, true) => None,
                (true, false) => Some(SavedSize {
                    maximized: true,
                    ..previous
                }),
                (false, false) => Some(SavedSize {
                    width: self.surfaces[index].logical_size.0,
                    height: self.surfaces[index].logical_size.1,
                    maximized: false,
                }),
            };
            if let Some(saved) = saved
                && let Err(error) = crate::app::write_toml(&state, &saved)
            {
                tracing::warn!(%error, "bk: window size not saved");
            }
            status.set(WindowStatus {
                activated: false,
                ..shown
            });
        }
        if !self.needs_exit && matches!(self.surfaces[index].role, SurfaceRole::Window(_)) {
            let window = &mut self.surfaces[index];
            for handle in [window.menu_handle.take(), window.status_handle.take()]
                .into_iter()
                .flatten()
            {
                if let Some(path) = handle.path() {
                    crate::platform::linux::dbusmenu::server::unregister(&path);
                }
            }
            if let Some(appmenu) = self.surfaces[index].appmenu.take() {
                appmenu.release();
            }
            let sheets: Vec<ObjectId> = self
                .surfaces
                .iter()
                .filter_map(|surface| surface.context.as_ref())
                .filter(|context| context.parent.as_ref() == Some(&surface_id))
                .map(|context| context.controls.surface_id.clone())
                .collect();
            for sheet in sheets {
                self.close_window(sheet);
            }
            self.close_popups_of(&surface_id);
            let Some(index) = self.surface_index_by_id(&surface_id) else {
                return;
            };
            self.surfaces.remove(index);
        }
    }
}
