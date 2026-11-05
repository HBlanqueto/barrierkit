use freya_core::{
    integration::AppComponent,
    prelude::{State, WritableUtils},
};
use sctk::{
    reexports::{
        calloop::ping::Ping,
        client::{Connection, Proxy, QueueHandle, backend::ObjectId},
        protocols::xdg::shell::client::xdg_toplevel,
    },
    shell::{
        WaylandSurface,
        xdg::window::{DecorationMode, Window, WindowConfigure, WindowDecorations, WindowHandler},
    },
};
use std::{cell::RefCell, rc::Rc};

use super::{
    RunError,
    app::AppInstance,
    runtime::{Connected, Runtime, WindowContext},
    surface::SurfaceRole,
    timer::WakeHandle,
    window_style::Bar,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FullscreenBar {
    #[default]
    Solid,
    Glass,
    ProgressiveGlass,
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

#[derive(Clone, Debug)]
pub struct WindowOptions {
    pub title: String,
    pub app_name: String,
    pub app_id: Option<String>,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub menu: Option<crate::app_menu::AppMenu>,
    pub fullscreen_bar: FullscreenBar,
    pub fullscreen_push: bool,
    pub hud: bool,
    pub status_item: Option<crate::status_item::StatusItem>,
}

impl WindowOptions {
    pub fn new(title: impl Into<String>, app_name: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_name: app_name.into(),
            app_id: None,
            size: vow_design::WINDOW_DEFAULT_SIZE,
            min_size: None,
            max_size: None,
            menu: None,
            fullscreen_bar: FullscreenBar::Solid,
            fullscreen_push: false,
            hud: false,
            status_item: None,
        }
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
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

    pub fn menu(mut self, menu: crate::app_menu::AppMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    pub fn fullscreen_bar(mut self, bar: FullscreenBar) -> Self {
        self.fullscreen_bar = bar;
        self
    }

    pub fn fullscreen_push(mut self, push: bool) -> Self {
        self.fullscreen_push = push;
        self
    }

    pub fn hud(mut self, hud: bool) -> Self {
        self.hud = hud;
        self
    }

    pub fn status_item(mut self, item: crate::status_item::StatusItem) -> Self {
        self.status_item = Some(item);
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
    pub focus_bar_progress: State<f32>,
    pub focus_overlay: bool,
    pub focus_bar_height: State<f32>,
    pub fullscreen_bar: State<FullscreenBar>,
    pub fullscreen_push: State<bool>,
    pub(crate) overlay_titlebar_height: State<f32>,
    pub hud: bool,
}

pub(crate) fn window_activated() -> bool {
    freya_core::prelude::try_consume_root_context::<WindowControls>()
        .is_none_or(|controls| controls.status.read().activated)
}

impl WindowControls {
    pub fn traffic_lights_shown(&self) -> bool {
        !self.status.read().fullscreen || *self.focus_bar_progress.read() > 0.0
    }

    pub fn titlebar_overlaid(&self) -> bool {
        self.focus_overlay && self.status.read().fullscreen
    }

    pub fn fullscreen_content_top(&self) -> f32 {
        let below_bar = match *self.fullscreen_bar.read() {
            FullscreenBar::ProgressiveGlass => vow_design::FULLSCREEN_GLASS.content_top,
            FullscreenBar::Glass => vow_design::FULLSCREEN_BAND.content_top,
            FullscreenBar::Solid => vow_design::FULLSCREEN_BAR.content_top,
        };
        if self.titlebar_overlaid() && !*self.fullscreen_push.read() {
            *self.focus_bar_height.read() + below_bar
        } else {
            0.0
        }
    }

    pub(crate) fn push_offset(&self) -> f32 {
        if *self.fullscreen_push.read() && self.titlebar_overlaid() {
            (*self.focus_bar_height.read() + *self.overlay_titlebar_height.read())
                * *self.focus_bar_progress.read()
        } else {
            0.0
        }
    }

    pub fn titlebar_overlay_top(&self) -> Option<f32> {
        let progress = *self.focus_bar_progress.read();
        let hidden_by = vow_design::TITLEBAR_CLIENT_HEIGHT as f32 * (1.0 - progress);
        (self.titlebar_overlaid() && progress > 0.0)
            .then(|| *self.focus_bar_height.read() * progress - hidden_by)
    }

    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    fn send(&self, command: WindowCommand) {
        self.queue.borrow_mut().push(command);
        self.ping.ping();
    }

    pub fn present(&self, activation_token: String) {
        self.send(WindowCommand::Present(activation_token));
    }

    pub fn close(&self) {
        self.send(WindowCommand::Close);
    }

    pub(crate) fn set_bar(&self, bar: Bar) {
        self.send(WindowCommand::Bar(bar));
    }

    pub fn set_title(&self, title: impl Into<String>) {
        self.send(WindowCommand::SetTitle(title.into()));
    }

    pub fn minimize(&self) {
        self.send(WindowCommand::Minimize);
    }

    pub fn toggle_maximize(&self) {
        self.send(WindowCommand::ToggleMaximize);
    }

    pub fn toggle_fullscreen(&self) {
        self.send(WindowCommand::ToggleFullscreen);
    }

    pub fn move_window(&self) {
        self.send(WindowCommand::Move(self.pointer_serial.get()));
    }

    pub fn resize(&self, edge: WindowEdge) {
        self.send(WindowCommand::Resize(self.pointer_serial.get(), edge));
    }
}

pub(crate) enum WindowCommand {
    Close,
    Minimize,
    ToggleMaximize,
    ToggleFullscreen,
    Move(u32),
    Resize(u32, WindowEdge),
    Present(String),
    Bar(Bar),
    SetTitle(String),
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
pub(super) fn resize_edge_at(
    (x, y): (f64, f64),
    (width, height): (u32, u32),
) -> Option<WindowEdge> {
    let (width, height) = (f64::from(width), f64::from(height));
    let edge = f64::from(vow_design::WINDOW_RESIZE.edge);
    let corner = f64::from(vow_design::WINDOW_RESIZE.corner);
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

fn window_state(app_id: &str) -> String {
    format!("{app_id}.window")
}

fn within(size: (u32, u32), min: Option<(u32, u32)>, max: Option<(u32, u32)>) -> (u32, u32) {
    let (low_w, low_h) = min.unwrap_or((1, 1));
    let (high_w, high_h) = max.unwrap_or((u32::MAX, u32::MAX));
    (
        size.0.clamp(low_w, high_w.max(low_w)),
        size.1.clamp(low_h, high_h.max(low_h)),
    )
}

pub(crate) struct OpenUtility {
    pub(crate) app: AppComponent,
    pub(crate) options: WindowOptions,
}

pub fn open_utility_window(app: AppComponent, options: WindowOptions) {
    freya_core::prelude::Platform::get().send(freya_core::integration::UserEvent::Erased(
        freya_core::integration::SingleThreadErasedEvent(Box::new(OpenUtility { app, options })),
    ));
}

pub fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
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
            None => options.app_id.clone().unwrap_or_else(|| {
                format!("{}{}", vow_design::NATIVE_APP_ID_PREFIX, options.app_name)
            }),
        };
        let utility = parent.is_some();
        if let Some((parent, _)) = &parent {
            window.set_parent(Some(parent));
        }
        let saved: SavedSize = if utility {
            SavedSize::default()
        } else {
            crate::app::load_restoration(&window_state(&app_id))
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

        let pointer_serial = Rc::new(std::cell::Cell::new(0));
        let fixed = options.min_size.is_some() && options.min_size == options.max_size;
        let status = State::create_global(WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed,
        });
        let surface_id = surface.id();
        let controls = WindowControls {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: self.ping.clone(),
            pointer_serial,
            status,
            title: options.title.clone(),
            fixed,
            surface_id,
            focus_bar: State::create_global(false),
            focus_bar_progress: State::create_global(0.0),
            focus_overlay: self
                .focus_manager
                .as_ref()
                .is_some_and(|manager| manager.version() >= 2),
            focus_bar_height: State::create_global(vow_design::MENUBAR_HEIGHT as f32),
            fullscreen_bar: State::create_global(options.fullscreen_bar),
            fullscreen_push: State::create_global(options.fullscreen_push),
            overlay_titlebar_height: State::create_global(0.0),
            hud: options.hud,
        };
        let focus_notice = self.focus_manager.as_ref().map(|manager| {
            let notice = manager.get_focus_surface(&surface, qh, surface.id());
            if controls.focus_overlay {
                notice.set_overlay();
            }
            notice
        });

        let app_instance = AppInstance::new(
            self,
            surface.id(),
            app,
            1.0,
            options.size,
            Some(controls.clone()),
            None,
            menu_handle.clone(),
        );
        let mut window = self.surface_window(surface, SurfaceRole::Window(window), app_instance);
        window.logical_size = options.size;
        window.menu_handle = menu_handle;
        window.status_handle = options.status_item.map(|item| {
            let menu = crate::app_menu::AppMenu::new(vec![crate::app_menu::Menu::new(
                "",
                item.items.clone(),
            )]);
            let handle = crate::app_menu::MenuHandle::new(
                menu,
                Some(WakeHandle::from_ping(self.wake.ping.clone())),
            );
            crate::app_menu::server::register_status_item(handle.clone(), item);
            handle
        });
        window.context = Some(WindowContext {
            controls: controls.clone(),
            app_id,
            focus_notice,
            utility,
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
                if let WindowCommand::Close = command {
                    self.close_window(controls.surface_id.clone());
                    continue;
                }
                let Some(index) = self.surface_index_by_id(&controls.surface_id) else {
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
                    WindowCommand::SetTitle(title) => window.set_title(title),
                    WindowCommand::Bar(bar) => {
                        if let Some(style) = &self.window_style {
                            style.set_bar(window.wl_surface(), bar);
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
        if self.surfaces[index].draw_count == 0 {
            self.apply_pending_window_configures(qh);
        }
        self.surfaces[index].draw_after_configure(&self.compositor, qh);
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
            self.surfaces[index].configure(size, scale);
            let decoration = match configure.decoration_mode {
                DecorationMode::Server => DecorationResponse::Server,
                DecorationMode::Client => DecorationResponse::Client,
            };
            if let Some(context) = &self.surfaces[index].context {
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
        let Some(index) = self.surface_index_by_id(&surface_id) else {
            return;
        };
        if let Some(context) = &self.surfaces[index].context
            && !context.utility
        {
            let mut status = context.controls.status;
            let shown = *status.peek();
            let state = window_state(&context.app_id);
            let previous: SavedSize = crate::app::load_restoration(&state);
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
                && let Err(error) = crate::app::save_restoration(&state, &saved)
            {
                tracing::warn!(%error, "barrier-kit: window size not saved");
            }
            status.set(WindowStatus {
                activated: false,
                ..*status.read()
            });
        }
        if !self.needs_exit && matches!(self.surfaces[index].role, SurfaceRole::Window(_)) {
            if let Some(address) = self.surfaces[index].appmenu_address.take() {
                crate::app_menu::server::unregister(&address.path);
            }
            if let Some(appmenu) = self.surfaces[index].appmenu.take() {
                appmenu.release();
            }
            self.surfaces.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::within;

    #[test]
    fn a_restored_size_stays_within_the_window_limits() {
        assert_eq!(
            within((300, 900), Some((480, 320)), Some((800, 600))),
            (480, 600)
        );
        assert_eq!(within((1000, 700), None, None), (1000, 700));
        assert_eq!(
            within((500, 500), Some((640, 420)), Some((640, 420))),
            (640, 420)
        );
    }
}
