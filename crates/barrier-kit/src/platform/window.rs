use super::*;

/// Who draws a MalaysiaDreams app's titlebar (ADR 0035).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decorations {
    /// BarrierKit draws its 44 px titlebar.
    #[default]
    Client,
    /// MalaysiaDreams's compositor draws the 32 px titlebar.
    Server,
}

/// Which edge of a client-side window is being resized.
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

/// How the compositor answered the decoration request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationResponse {
    Client,
    Server,
}

/// Handler called when a global menu or in-app shortcut fires an action.
pub type MenuActionHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Public options for a normal BarrierKit window, without SCTK types.
#[derive(Clone)]
pub struct WindowOptions {
    pub title: String,
    pub app_name: String,
    /// Optional override for the Wayland `app_id`; defaults to
    /// `NATIVE_APP_ID_PREFIX + app_name`.
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
    /// A 640×420 window with client-side chrome and the given title/app name.
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

/// The state a BarrierKit window's titlebar reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStatus {
    pub activated: bool,
    pub maximized: bool,
    pub decoration: DecorationResponse,
    pub fixed: bool,
}

/// Actions a titlebar can request from the runner.
#[derive(Clone)]
pub struct WindowControls {
    pub(super) queue: Rc<RefCell<Vec<WindowCommand>>>,
    pub(super) ping: Ping,
    pub(crate) pointer_serial: Rc<std::cell::Cell<u32>>,
    pub status: State<WindowStatus>,
    pub title: String,
    pub fixed: bool,
    pub(super) surface_id: ObjectId,
    pub(super) menu_handle: Option<crate::app_menu::MenuHandle>,
}

impl WindowControls {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn pointer_serial(&self) -> u32 {
        self.pointer_serial.get()
    }

    pub fn minimize(&self) {
        self.queue.borrow_mut().push(WindowCommand::Minimize);
        self.ping.ping();
    }

    pub fn toggle_maximize(&self) {
        self.queue.borrow_mut().push(WindowCommand::ToggleMaximize);
        self.ping.ping();
    }

    /// Toggle vertical maximise for fixed-width windows.
    ///
    /// The kit still sends the standard maximise/restore request; the compositor
    /// recognizes fixed-width windows and performs the vertical fill itself.
    /// The stored restore rectangle is the window's current size.
    pub fn toggle_vertical_maximize(&self) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::ToggleVerticalMaximize);
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

    /// The Wayland object id of the toplevel surface these controls belong to.
    pub fn surface_id(&self) -> ObjectId {
        self.surface_id.clone()
    }

    /// The global menu handle for this window, if it was configured with one.
    pub fn menu_handle(&self) -> Option<crate::app_menu::MenuHandle> {
        self.menu_handle.clone()
    }

    /// Opens another toplevel in the same process, attached to this window as
    /// its parent when `options.parent` is `Some(surface_id())`.
    pub fn open_window(&self, app: AppComponent, options: WindowOptions) {
        self.queue
            .borrow_mut()
            .push(WindowCommand::Open(app, options));
        self.ping.ping();
    }
}

pub(super) enum WindowCommand {
    Minimize,
    ToggleMaximize,
    ToggleVerticalMaximize,
    Move(u32),
    Resize(u32, WindowEdge),
    Open(AppComponent, WindowOptions),
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
/// xdg_toplevel.set_min_size and set_max_size take signed 32-bit ints where
/// 0 means "unbounded". Values that do not fit in i32 must be sent as 0 to
/// avoid the `invalid_size` protocol error.
pub(super) fn clamp_toplevel_size_dimension(value: u32) -> u32 {
    if value > i32::MAX as u32 { 0 } else { value }
}
/// Runs a normal BarrierKit window (`xdg_toplevel`, ADR 0035).
pub fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
    let (conn, event_queue, qh, mut shell) =
        Runtime::connect(app.clone(), SurfaceSpec::Window(options.clone()))?;
    shell.qh = qh.clone();
    shell
        .create_window(app, options)
        .ok_or(RunError::MissingGlobal("xdg_wm_base"))?;

    let mut event_loop: EventLoop<Runtime> =
        EventLoop::try_new().map_err(|_| RunError::Egl("calloop EventLoop::try_new failed"))?;
    insert_ping(&mut event_loop, &mut shell)?;
    WaylandSource::new(conn, event_queue)
        .insert(event_loop.handle())
        .map_err(|_| RunError::Egl("inserting the Wayland source"))?;
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
        .map_err(|_| RunError::Egl("calloop event loop failed"))?;
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
                    // Only fails if a panic already poisoned the mutex; no external data here.
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

        // The org_kde_kwin_appmenu object is created lazily when the
        // asynchronous D-Bus registration delivers the address.
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
            menu_handle: menu_handle.clone(),
        };
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
            surface_count: self.surface_count.clone(),
            opens_left: Rc::new(Cell::new(false)),
        };
        let app_instance = AppInstance::new(
            app,
            1.0,
            options.size,
            sender,
            waker,
            Some(self.toplevels.clone()),
            None,
            self.activation.clone(),
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
            configure_count: 0,
            pending_configure: None,
            appmenu: appmenu_object,
            appmenu_address: None,
            menu_handle,
            fractional,
            viewport,
        });
        self.surface_count.set(self.surfaces.len());
        Some(controls)
    }

    pub(super) fn process_window_commands(&mut self) {
        let contexts = self.window_contexts.clone();
        for context in contexts {
            let commands = std::mem::take(&mut *context.queue.borrow_mut());
            for command in commands {
                if let WindowCommand::Open(app, options) = command {
                    self.create_window(app, options);
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
                    WindowCommand::Move(serial) => {
                        if let Some(seat) = &self.activation_seat {
                            window.move_(seat, serial);
                        }
                    }
                    WindowCommand::Resize(serial, edge) => {
                        if let Some(seat) = &self.activation_seat {
                            window.resize(seat, serial, resize_edge(edge));
                        }
                    }
                    WindowCommand::Open(..) => unreachable!(),
                }
                self.surfaces[index].needs_redraw = true;
            }
        }
    }
}

impl WindowHandler for Runtime {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        let surface_id = window.wl_surface().id();
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
        if !self.needs_exit {
            // Drop the closed surface and its controls so later commands are
            // ignored. The window_contexts entry is removed as well.
            if let Some(index) = self.surfaces.iter().position(|s| {
                matches!(s.role, SurfaceRole::Window(_)) && s.surface.id() == surface_id
            }) {
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
                self.surface_count.set(self.surfaces.len());
            }
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
            target: "barrier_kit::resize",
            surface = ?self.surfaces[index].surface.id(),
            serial,
            width = size.0,
            height = size.1,
            resizing = configure.is_resizing(),
            "configure received"
        );
        // Always keep the latest configure; older ones are overwritten. The
        // request_redraw ensures a frame callback will process it.
        self.surfaces[index].pending_configure = Some((configure, serial));
        self.surfaces[index].needs_redraw = true;
        self.surfaces[index].request_frame_callback(qh);
    }
}
impl Runtime {
    /// Applies the most recent pending window configure for each surface.
    ///
    /// Called from the frame callback so several configures that arrive between
    /// frames collapse into a single resize/draw.
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
            // A 0×0 configure leaves the size to the client: keep the size the app
            // asked for. Dropping the configure would leave the surface without a
            // buffer, so the window would never map.
            let size = if size.0 > 0 && size.1 > 0 {
                size
            } else {
                self.surfaces[index].logical_size
            };
            // Only commit immediately when the size did not change. A size-changing
            // configure will be followed by a redraw that commits a buffer at the
            // new size; committing the old buffer here makes the compositor see a
            // transient size mismatch and loop (B2).
            let size_changed = size != self.surfaces[index].logical_size;
            self.surfaces[index].configure_count += 1;
            let count = self.surfaces[index].configure_count;
            if count.is_multiple_of(30) {
                tracing::info!(
                    surface = ?self.surfaces[index].surface.id(),
                    configures = count,
                    egl_windows = self.surfaces[index].shared.egl_window_creations.get(),
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
            // MalaysiaDreams always grants an explicit Server request. Some compositors do
            // not send the decoration configure at all, so keep the requested mode
            // when they advertise the protocol; without it, fall back to the
            // client-side bar.
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
                    decoration,
                    fixed: context.controls.fixed,
                });
            }
            // Acknowledge the configure and let SCTK apply the decoration mode
            // before drawing the first frame. A size change is committed by the
            // redraw below, not here, to avoid sending the old buffer size.
            if !size_changed && let SurfaceRole::Window(window) = &self.surfaces[index].role {
                window.commit();
            }
            // A toplevel must attach its first buffer before it can map; unlike a
            // layer surface there is no frame callback to wait for yet.
            if self.surfaces[index].draw() && self.surfaces[index].needs_redraw {
                self.surfaces[index].request_redraw(qh);
            }
        }
    }
}
