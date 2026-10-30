use super::{
    ObjectId, RunError,
    app_menu_export::AppMenuAddress,
    egl::GraphicsShared,
    extension::Extension,
    input::seat::{RepeatState, cursor_shape_from_icon},
    popup::PopupCommand,
    surface::SurfaceWindow,
    timer::{TimerQueue, Wake},
    window::{WindowCommand, WindowControls},
};
use freya_core::{integration::AppComponent, prelude::State};

use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    os::fd::AsFd,
    rc::Rc,
    sync::{Arc, Mutex},
};

use freya_core::integration::{UserEvent, WritableUtils};
use sctk::{
    compositor::CompositorState,
    delegate_compositor, delegate_output, delegate_registry, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            self, EventLoop, Interest, LoopSignal, Mode, PostAction,
            generic::Generic,
            ping::{Ping, PingSource},
        },
        client::{
            Connection, Proxy, QueueHandle,
            globals::{GlobalList, registry_queue_init},
            protocol::{
                wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface::WlSurface, wl_touch,
            },
        },
        protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        SeatState,
        keyboard::{Modifiers as SctkModifiers, RepeatInfo},
        pointer::cursor_shape::CursorShapeManager,
    },
    shell::xdg::XdgShell,
};
use vow_appearance::{Resolved, load_appearance, load_dominant, resolve, watch::AppearanceWatch};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

/// The result of connecting to the Wayland server before any shell extension
/// is installed.
pub(crate) struct Connected {
    pub(crate) conn: Connection,
    pub(crate) event_queue: sctk::reexports::client::EventQueue<Runtime>,
    pub(crate) runtime: Runtime,
    // Solo lo usa shell/ (feature `shell`); sin ella no se lee.
    #[allow(dead_code)]
    pub(crate) globals: GlobalList,
}

pub(crate) struct Runtime {
    pub(crate) registry: RegistryState,
    pub(crate) compositor: CompositorState,
    pub(crate) output: OutputState,
    pub(crate) xdg_shell: Option<XdgShell>,
    pub(crate) surfaces: Vec<SurfaceWindow>,
    pub(crate) graphics: Rc<GraphicsShared>,
    pub(crate) seat: SeatState,
    pub(crate) pointer: Option<wl_pointer::WlPointer>,
    pub(crate) cursor_shape: Option<CursorShapeManager>,
    pub(crate) cursor_shape_device: Option<WpCursorShapeDeviceV1>,
    pub(crate) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(crate) touch: Option<wl_touch::WlTouch>,
    pub(crate) keyboard_focus: Option<WlSurface>,
    pub(crate) repeat: Option<RepeatState>,
    pub(crate) repeat_info: Option<RepeatInfo>,
    pub(crate) repeat_generation: u64,
    pub(crate) modifiers: SctkModifiers,
    pub(crate) loop_signal: Option<LoopSignal>,
    // Solo lo usa shell/ (feature `shell`); sin ella no se lee.
    #[allow(dead_code)]
    pub(crate) app: AppComponent,
    pub(crate) needs_exit: bool,
    pub(crate) request_redraw_all: bool,
    pub(crate) queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
    pub(crate) ping: Ping,
    pub(crate) ping_source: Option<PingSource>,
    pub(crate) wake: Wake,
    pub(crate) wake_source: Option<PingSource>,
    pub(crate) qh: QueueHandle<Runtime>,
    pub(crate) extension: Option<Extension>,
    pub(crate) appmenu_manager: Option<OrgKdeKwinAppmenuManager>,
    pub(crate) fractional_manager: Option<WpFractionalScaleManagerV1>,
    pub(crate) viewporter: Option<WpViewporter>,
    pub(crate) timers: TimerQueue,
    pub(crate) seat_handle: Option<wl_seat::WlSeat>,
    pub(crate) last_input_serial: Rc<Cell<u32>>,
    pub(crate) last_pointer_serial: Cell<u32>,
    pub(crate) touch_surfaces: std::collections::HashMap<i32, ObjectId>,
    pub(crate) popup_commands: Rc<RefCell<Vec<PopupCommand>>>,
    pub(crate) popup_next_id: Rc<Cell<u64>>,
    pub(crate) surface_count: Rc<Cell<usize>>,
    pub(crate) window_contexts: Vec<WindowContext>,
    pub(crate) decoration_manager: bool,
    pub(crate) decoration_warned: bool,
    pub(crate) appearance_watch: Option<AppearanceWatch>,
    pub(crate) resolved: State<Resolved>,
    /// Addresses delivered asynchronously by the D-Bus thread, waiting for the
    /// UI loop to call `org_kde_kwin_appmenu.set_address`.
    pub(crate) pending_menu_addresses: Arc<Mutex<Vec<(ObjectId, AppMenuAddress)>>>,
}

#[derive(Clone)]
pub(crate) struct WindowContext {
    pub(crate) controls: WindowControls,
    pub(crate) queue: Rc<RefCell<Vec<WindowCommand>>>,
}

pub(crate) fn make_sender(
    queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
    ping: Ping,
    owner: ObjectId,
) -> Rc<dyn Fn(UserEvent)> {
    Rc::new(move |event| {
        queue.borrow_mut().push((owner.clone(), event));
        ping.ping();
    })
}

pub(crate) fn drain_user_events(
    events: &Rc<RefCell<Vec<(ObjectId, UserEvent)>>>,
) -> Vec<(ObjectId, UserEvent)> {
    std::mem::take(&mut *events.borrow_mut())
}

pub(crate) fn insert_ping(
    event_loop: &mut EventLoop<Runtime>,
    shell: &mut Runtime,
) -> Result<(), RunError> {
    let source = shell.ping_source.take().ok_or(RunError::EventLoop(
        "the Freya ping source is already installed",
    ))?;
    let handle = event_loop.handle();
    event_loop
        .handle()
        .insert_source(source, move |_, _, shell: &mut Runtime| {
            for (surface_id, event) in drain_user_events(&shell.queue) {
                match event {
                    UserEvent::RequestRedraw => {
                        shell.request_redraw_all = true;
                    }
                    UserEvent::SetCursorIcon(icon) => {
                        if let Some(device) = &shell.cursor_shape_device {
                            device.set_shape(
                                shell.last_pointer_serial.get(),
                                cursor_shape_from_icon(icon),
                            );
                        }
                    }
                    UserEvent::FocusAccessibilityNode(strategy) => {
                        if let Some(index) = shell
                            .surfaces
                            .iter()
                            .position(|s| s.surface.id() == surface_id)
                        {
                            shell.surfaces[index].app.requested_focus = Some(strategy);
                            shell.surfaces[index].needs_redraw = true;
                        }
                    }
                    UserEvent::Erased(_) => {}
                }
            }
            shell.poll_all();
            shell.process_popup_commands();
            shell.install_timers(&handle);
            shell.install_repeat(&handle);
            shell.redraw_all();
        })
        .map_err(|_| RunError::EventLoop("inserting the Freya ping source"))?;

    let wake_source = shell.wake_source.take().ok_or(RunError::EventLoop(
        "the wake ping source is already installed",
    ))?;
    let callbacks = shell.wake.callbacks.clone();
    event_loop
        .handle()
        .insert_source(wake_source, move |_, _, shell: &mut Runtime| {
            let callbacks: Vec<Rc<dyn Fn()>> = callbacks.borrow().clone();
            for callback in callbacks {
                callback();
            }
            shell.process_pending_menu_addresses();
            shell.process_menu_actions();
            shell.poll_all();
            shell.redraw_all();
        })
        .map_err(|_| RunError::EventLoop("inserting the wake ping source"))?;

    // Watch appearance.domain and wallpaper.state; when they change, resolve
    // the theme again and update the process-wide reactive state so every
    // surface that reads theme::resolved() re-executes.
    let watch = AppearanceWatch::new().map_err(|error| {
        RunError::Io(std::io::Error::other(format!(
            "creating the appearance watcher: {error}"
        )))
    })?;
    let watch_fd = watch.as_fd().try_clone_to_owned().map_err(RunError::Io)?;
    shell.appearance_watch = Some(watch);
    event_loop
        .handle()
        .insert_source(
            Generic::new(watch_fd, Interest::READ, Mode::Level),
            |_, _, shell: &mut Runtime| {
                if let Some(watch) = shell.appearance_watch.as_ref()
                    && let Ok(events) = watch.drain()
                    && !events.is_empty()
                {
                    let resolved = resolve(&load_appearance(), load_dominant());
                    if *shell.resolved.peek() != resolved {
                        shell.resolved.set(resolved);
                        shell.request_redraw_all = true;
                        shell.ping.ping();
                    }
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|_| RunError::EventLoop("inserting the appearance watch source"))?;
    Ok(())
}

impl Runtime {
    pub(crate) fn connect(app: AppComponent) -> Result<Connected, RunError> {
        let conn = Connection::connect_to_env().map_err(|_| RunError::NoDisplay)?;
        let (globals, event_queue) = registry_queue_init::<Runtime>(&conn)?;
        let qh = event_queue.handle();
        let queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>> = Rc::new(RefCell::new(Vec::new()));
        let (ping, ping_source) =
            calloop::ping::make_ping().map_err(|_| RunError::EventLoop("calloop ping failed"))?;
        let (wake_ping, wake_source) = calloop::ping::make_ping()
            .map_err(|_| RunError::EventLoop("calloop wake ping failed"))?;
        let resolved = State::create_global(resolve(&load_appearance(), load_dominant()));
        let appmenu_manager = globals
            .bind::<OrgKdeKwinAppmenuManager, _, _>(&qh, 1..=1, ())
            .ok();
        if appmenu_manager.is_none() {
            tracing::info!("org_kde_kwin_appmenu_manager no disponible; sigo sin él");
        }
        let fractional_manager = globals
            .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
            .ok();
        if fractional_manager.is_none() {
            tracing::info!(
                "wp_fractional_scale_manager_v1 no disponible; sigo con escalas enteras"
            );
        }
        let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
        if viewporter.is_none() {
            tracing::info!("wp_viewporter no disponible; sigo con escalas enteras");
        }
        // `SeatState::new` binds the seats that already exist without calling
        // `SeatHandler::new_seat`, so remember one here for popup grabs,
        // window moves and resizes. Hotplugged seats still arrive through `new_seat`.
        let seat = SeatState::new(&globals, &qh);
        let seat_handle = seat.seats().next();
        let shell = Runtime {
            registry: RegistryState::new(&globals),
            compositor: CompositorState::bind(&globals, &qh)
                .map_err(|_| RunError::MissingGlobal("wl_compositor"))?,
            output: OutputState::new(&globals, &qh),
            xdg_shell: XdgShell::bind(&globals, &qh).ok(),
            surfaces: Vec::new(),
            graphics: {
                let display_ptr = conn.display().id().as_ptr().cast::<c_void>();
                Rc::new(GraphicsShared::new(display_ptr)?)
            },
            seat,
            pointer: None,
            cursor_shape: {
                let manager = CursorShapeManager::bind(&globals, &qh).ok();
                if manager.is_none() {
                    tracing::info!(
                        "wp_cursor_shape_manager_v1 no disponible; uso cursor por defecto"
                    );
                }
                manager
            },
            cursor_shape_device: None,
            keyboard: None,
            touch: None,
            keyboard_focus: None,
            repeat: None,
            repeat_info: None,
            repeat_generation: 0,
            modifiers: SctkModifiers::default(),
            loop_signal: None,
            app,
            needs_exit: false,
            request_redraw_all: false,
            queue,
            ping: ping.clone(),
            ping_source: Some(ping_source),
            wake: Wake {
                ping: wake_ping,
                callbacks: Rc::new(RefCell::new(Vec::new())),
            },
            wake_source: Some(wake_source),
            qh: qh.clone(),
            extension: None,
            appmenu_manager,
            fractional_manager,
            viewporter,
            timers: Rc::new(RefCell::new(Vec::new())),
            seat_handle,
            last_input_serial: Rc::new(Cell::new(0)),
            last_pointer_serial: Cell::new(0),
            touch_surfaces: std::collections::HashMap::new(),
            popup_commands: Rc::new(RefCell::new(Vec::new())),
            popup_next_id: Rc::new(Cell::new(1)),
            surface_count: Rc::new(Cell::new(0)),
            window_contexts: Vec::new(),
            decoration_manager: globals.contents().with_list(|list| {
                list.iter()
                    .any(|global| global.interface == "zxdg_decoration_manager_v1")
            }),
            decoration_warned: false,
            appearance_watch: None,
            resolved,
            pending_menu_addresses: Arc::new(Mutex::new(Vec::new())),
        };
        Ok(Connected {
            conn,
            event_queue,
            runtime: shell,
            globals,
        })
    }

    pub(crate) fn surface_for(&self, surface: &WlSurface) -> Option<usize> {
        self.surfaces.iter().position(|s| s.surface == *surface)
    }

    pub(crate) fn surface_index_by_id(&self, surface_id: &ObjectId) -> Option<usize> {
        self.surfaces
            .iter()
            .position(|s| s.surface.id() == *surface_id)
    }
}

impl Runtime {
    pub(crate) fn poll_all(&mut self) {
        for window in &mut self.surfaces {
            let dirty = window.app.poll();
            if dirty {
                window.app.update();
                window.needs_redraw = true;
            }
            if self.request_redraw_all {
                window.needs_redraw = true;
            }
        }
        self.request_redraw_all = false;
        if let Some(hooks) = self.extension_hooks() {
            (hooks.poll)(self);
        }
        self.process_window_commands();
    }

    pub(crate) fn redraw_all(&mut self) {
        let qh = self.qh.clone();
        for window in &mut self.surfaces {
            if window.needs_redraw {
                // With the compositor now scheduling a repaint for any surface
                // commit that carries damage or asks for a frame callback, we
                // can go back to drawing on the frame callback instead of
                // committing immediately on every user event.
                window.request_redraw(&qh);
            }
        }
    }
}

impl ProvidesRegistryState for Runtime {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    fn runtime_add_global(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: u32,
        _: &str,
        _: u32,
    ) {
    }

    fn runtime_remove_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str) {}
}

impl OutputHandler for Runtime {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(&mut self, _: &Connection, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        if let Some(hooks) = self.extension_hooks() {
            (hooks.new_output)(self, qh, &output);
        }
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

delegate_compositor!(Runtime);
delegate_output!(Runtime);
delegate_registry!(Runtime);
delegate_xdg_shell!(Runtime);
delegate_xdg_window!(Runtime);
