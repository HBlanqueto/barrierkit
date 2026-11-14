use super::{
    EditShortcut, ObjectId, RunError, Sent,
    clipboard::{Clipboard, ClipboardDevice, PendingDrop},
    extension::Extension,
    input::seat::{RepeatState, cursor_shape_from_icon},
    main_menu_export::MainMenuAddress,
    popup::PopupCommand,
    surface::SurfaceWindow,
    timer::{TimerQueue, Wake},
    vulkan::{GraphicsShared, IDLE_RESOURCE_RELEASE},
    window::WindowControls,
};
use freya_core::prelude::{AccessibilityFocusStrategy, CursorIcon, Key, State};
use torin::prelude::CursorPoint;

use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    os::fd::AsFd,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ctappearance::{
    Resolved, load_appearance, load_dominant, resolve,
    watch::{AppearanceEvent, AppearanceWatch},
};
use freya_core::integration::{GlobalContexts, Platform, UserEvent, WritableUtils};
use sctk::{
    compositor::CompositorState,
    data_device_manager::DataDeviceManagerState,
    delegate_compositor, delegate_output, delegate_registry, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            self, EventLoop, Interest, LoopSignal, Mode, PostAction,
            generic::Generic,
            ping::{Ping, PingSource},
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, Proxy, QueueHandle,
            globals::registry_queue_init,
            protocol::{
                wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface::WlSurface, wl_touch,
            },
        },
        protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
        protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1,
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        SeatState,
        keyboard::{Modifiers as SctkModifiers, RepeatInfo},
        pointer::cursor_shape::CursorShapeManager,
    },
    shell::xdg::XdgShell,
};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1;
use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_manager_v3::ZwpTextInputManagerV3;
use wayland_protocols::wp::viewporter::client::wp_viewporter::WpViewporter;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

pub(crate) struct Connected {
    pub(crate) conn: Connection,
    pub(crate) event_queue: sctk::reexports::client::EventQueue<Runtime>,
    pub(crate) runtime: Runtime,
    #[cfg(feature = "shell")]
    pub(crate) globals: sctk::reexports::client::globals::GlobalList,
}

pub(crate) struct Runtime {
    pub(crate) registry: RegistryState,
    pub(crate) compositor: CompositorState,
    pub(crate) output: OutputState,
    pub(crate) xdg_shell: Option<XdgShell>,
    pub(crate) surfaces: Vec<SurfaceWindow>,
    pub(crate) graphics: Rc<GraphicsShared>,
    // SAFETY: this pointer is the live wl_display of the connection that owns this Runtime;
    // it must outlive every surface and every GraphicsShared recreated from it.
    pub(crate) display: *mut c_void,
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
    pub(crate) keyboard_layouts: Vec<String>,
    pub(crate) keyboard_layout_index: u32,
    pub(crate) keyboard_layout: Option<freya_core::prelude::State<Option<String>>>,
    pub(crate) needs_exit: bool,
    pub(crate) recover_failed: bool,
    pub(crate) request_redraw_all: bool,
    pub(crate) queue: Rc<RefCell<Vec<(ObjectId, super::Sent)>>>,
    pub(crate) ping: Ping,
    pub(crate) ping_source: Option<PingSource>,
    pub(crate) wake: Wake,
    pub(crate) wake_source: Option<PingSource>,
    pub(crate) qh: QueueHandle<Runtime>,
    pub(crate) extension: Option<Extension>,
    pub(crate) appmenu_manager: Option<OrgKdeKwinAppmenuManager>,
    pub(crate) focus_manager: Option<super::focus::InfinityFocusManagerV1>,
    pub(crate) dialog_manager:
        Option<wayland_protocols::xdg::dialog::v1::client::xdg_wm_dialog_v1::XdgWmDialogV1>,
    pub(crate) window_style: Option<super::window_style::InfinityWindowStyleV1>,
    pub(crate) fractional_manager: Option<WpFractionalScaleManagerV1>,
    pub(crate) viewporter: Option<WpViewporter>,
    pub(crate) timers: TimerQueue,
    pub(crate) seat_handle: Option<wl_seat::WlSeat>,
    pub(crate) last_input_serial: Rc<Cell<u32>>,
    pub(crate) last_input_surface: Option<WlSurface>,
    pub(crate) last_pointer_serial: Cell<u32>,
    pub(crate) resize_hover: Option<super::WindowEdge>,
    pub(crate) pointer_pressed: bool,
    pub(crate) pointer_surface: Option<WlSurface>,
    pub(crate) cursor_icon: CursorIcon,
    pub(crate) touchpad_scrolling: bool,
    pub(crate) touch_surfaces: std::collections::HashMap<i32, (ObjectId, (f64, f64))>,
    pub(crate) popup_commands: Rc<RefCell<Vec<PopupCommand>>>,
    pub(crate) popup_next_id: Rc<Cell<u64>>,
    pub(crate) appearance_watch: Option<AppearanceWatch>,
    pub(crate) resolved: State<Resolved>,
    pub(crate) pending_menu_addresses: Arc<Mutex<Vec<(ObjectId, MainMenuAddress)>>>,
    pub(crate) clipboard: Clipboard,
    pub(crate) text_input_manager: Option<ZwpTextInputManagerV3>,
    pub(crate) primary: Clipboard,
    pub(crate) primary_device: Option<super::primary::PrimaryDevice>,
    pub(crate) ime: super::ime::Ime,
    pub(crate) activation: Option<XdgActivationV1>,
    pub(crate) defaults_revision: State<u64>,
    pub(crate) clipboard_device: Option<ClipboardDevice>,
    pub(crate) drag_target: Option<(ObjectId, CursorPoint)>,
    pub(crate) pending_drop: Option<PendingDrop>,
}

pub(crate) struct WindowContext {
    pub(crate) controls: WindowControls,
    pub(crate) app_id: String,
    pub(crate) focus_notice: Option<super::focus::InfinityFocusSurfaceV1>,
    pub(crate) resolved: State<Resolved>,
    pub(crate) wallpaper: Option<ctdesign::Rgba>,
    pub(crate) wallpaper_fade: Option<WallpaperFade>,
    pub(crate) parent: Option<ObjectId>,
    pub(crate) key_only_if_needed: bool,
    pub(crate) modal: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct WallpaperFade {
    from: ctdesign::Rgba,
    to: ctdesign::Rgba,
    start: std::time::Instant,
}

impl WindowContext {
    pub(crate) fn fade_wallpaper(&mut self, to: ctdesign::Rgba, resolved: Resolved) {
        self.wallpaper_fade = self
            .wallpaper
            .filter(|from| *from != to && !resolved.reduced_motion)
            .map(|from| WallpaperFade {
                from,
                to,
                start: std::time::Instant::now(),
            });
        if self.wallpaper_fade.is_none() {
            self.wallpaper = Some(to);
            self.restyle(resolved);
        }
    }

    pub(crate) fn step_wallpaper_fade(&mut self, resolved: Resolved) -> bool {
        let Some(fade) = self.wallpaper_fade else {
            return false;
        };
        let t =
            (fade.start.elapsed().as_secs_f32() * 1000.0 / ctdesign::DURATION_BASE as f32).min(1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        self.wallpaper = Some(fade.from.mix(fade.to, eased));
        if t >= 1.0 {
            self.wallpaper_fade = None;
        }
        self.restyle(resolved);
        true
    }

    pub(crate) fn restyle(&mut self, resolved: Resolved) {
        let window = self
            .wallpaper
            .map_or(resolved, |wallpaper| resolved.behind_window(wallpaper));
        if *self.resolved.peek() != window {
            self.resolved.set(window);
        }
    }
}

impl Runtime {
    pub(crate) fn run(
        mut self,
        conn: Connection,
        event_queue: sctk::reexports::client::EventQueue<Self>,
        on_signal: impl FnOnce(&mut Self, LoopSignal),
    ) -> Result<(), RunError> {
        let mut event_loop: EventLoop<Self> = EventLoop::try_new()
            .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
        insert_ping(&mut event_loop, &mut self)?;
        WaylandSource::new(conn, event_queue)
            .insert(event_loop.handle())
            .map_err(|_| RunError::EventLoop("inserting the Wayland source"))?;
        let signal = event_loop.get_signal();
        on_signal(&mut self, signal.clone());
        event_loop
            .run(None, &mut self, |runtime| {
                if runtime.needs_exit {
                    signal.stop();
                } else if runtime.graphics.lost.get() {
                    while runtime.graphics.lost.get() {
                        if !runtime.recover_graphics() {
                            runtime.recover_failed = true;
                            signal.stop();
                            break;
                        }
                    }
                }
            })
            .map_err(|_| RunError::EventLoop("calloop event loop failed"))?;
        if self.recover_failed || self.graphics.lost.get() {
            return Err(RunError::DeviceLost);
        }
        Ok(())
    }

    fn recover_graphics(&mut self) -> bool {
        if self.graphics.recoveries.get() >= 2 {
            return false;
        }
        let Ok(shared) = GraphicsShared::new(self.display) else {
            return false;
        };
        let shared = Rc::new(shared);
        shared.recoveries.set(self.graphics.recoveries.get() + 1);
        for surface in &mut self.surfaces {
            surface.graphics = None;
        }
        for surface in &mut self.surfaces {
            surface.shared = shared.clone();
            surface.configure(surface.logical_size, surface.scale);
        }
        self.graphics = shared;
        #[cfg(feature = "video")]
        super::dmabuf::set_current(&self.graphics);
        let qh = self.qh.clone();
        self.apply_pending_window_configures();
        for surface in &mut self.surfaces {
            if surface.logical_size.0 > 0 && surface.logical_size.1 > 0 {
                surface.draw_after_configure(&self.compositor, &qh);
            }
        }
        tracing::warn!("Vulkan device lost; the device and the surfaces were recreated");
        true
    }
}

fn insert_ping(event_loop: &mut EventLoop<Runtime>, shell: &mut Runtime) -> Result<(), RunError> {
    let source = shell.ping_source.take().ok_or(RunError::EventLoop(
        "the Freya ping source is already installed",
    ))?;
    let handle = event_loop.handle();
    event_loop
        .handle()
        .insert_source(source, move |_, _, shell: &mut Runtime| {
            let events = std::mem::take(&mut *shell.queue.borrow_mut());
            for (surface_id, event) in events {
                match event {
                    Sent::User(UserEvent::RequestRedraw) => {
                        if let Some(index) = shell.surface_index_by_id(&surface_id) {
                            shell.surfaces[index].needs_redraw = true;
                        }
                    }
                    Sent::User(UserEvent::FocusAccessibilityNode(strategy)) => {
                        if let Some(index) = shell.surface_index_by_id(&surface_id) {
                            shell.surfaces[index].app.requested_focus = Some(strategy);
                            shell.surfaces[index].needs_redraw = true;
                        }
                    }
                    Sent::User(UserEvent::SetCustomScaleFactor(_)) => {}
                    Sent::Kit(event) if event.is::<super::window::OpenUtility>() => {
                        if let Ok(open) = event.downcast::<super::window::OpenUtility>() {
                            let open = *open;
                            shell.create_window(open.app, open.options, Some(&surface_id));
                        }
                    }
                    Sent::Kit(event) if event.is::<super::glass::GlassPanels>() => {
                        if let Ok(panels) = event.downcast::<super::glass::GlassPanels>() {
                            shell.apply_glass_panels(&surface_id, *panels);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::CardFrames>() => {
                        if let Ok(frames) = event.downcast::<crate::shell::CardFrames>() {
                            shell.apply_card_frames(&surface_id, *frames);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::PlateFrames>() => {
                        if let Ok(frames) = event.downcast::<crate::shell::PlateFrames>() {
                            shell.apply_plate_frames(&surface_id, *frames);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::BlurRegion>() => {
                        if let Ok(region) = event.downcast::<crate::shell::BlurRegion>() {
                            shell.apply_blur_region(&surface_id, *region);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::CaptureWindow>() => {
                        if let Ok(request) = event.downcast::<crate::shell::CaptureWindow>() {
                            shell.capture_window(*request);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::MinimizeTarget>() => {
                        if let Ok(target) = event.downcast::<crate::shell::MinimizeTarget>() {
                            shell.apply_minimize_target(&surface_id, *target);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::InputRegion>() => {
                        if let Ok(region) = event.downcast::<crate::shell::InputRegion>() {
                            shell.apply_input_region(&surface_id, *region);
                        }
                    }
                    #[cfg(feature = "shell")]
                    Sent::Kit(event) if event.is::<crate::shell::LayerGeometry>() => {
                        if let Ok(geometry) = event.downcast::<crate::shell::LayerGeometry>() {
                            shell.apply_layer_geometry(&surface_id, *geometry);
                        }
                    }
                    Sent::Kit(event) if event.is::<crate::drag::StartDrag>() => {
                        if let Ok(drag) = event.downcast::<crate::drag::StartDrag>() {
                            shell.start_drag(&surface_id, drag.0);
                        }
                    }
                    Sent::Kit(event) => {
                        if let Ok(shortcut) = event.downcast::<EditShortcut>()
                            && let Some(index) = shell.surface_index_by_id(&surface_id)
                        {
                            let app = &mut shell.surfaces[index].app;
                            app.requested_focus =
                                Some(AccessibilityFocusStrategy::Node(shortcut.field));
                            app.pending_shortcuts.push(Key::Character(shortcut.key));
                            shell.surfaces[index].needs_redraw = true;
                        }
                    }
                }
            }
            shell.publish_clipboard();
            shell.deliver_drop();
            shell.publish_primary();
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

    event_loop
        .handle()
        .insert_source(
            Timer::from_duration(IDLE_RESOURCE_RELEASE),
            |_, _, shell: &mut Runtime| {
                shell.graphics.release_idle_resources();
                TimeoutAction::ToDuration(IDLE_RESOURCE_RELEASE)
            },
        )
        .map_err(|_| RunError::EventLoop("inserting the idle resource timer"))?;

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
                    if events.iter().any(|event| {
                        matches!(
                            event,
                            AppearanceEvent::Defaults | AppearanceEvent::Wallpaper
                        )
                    }) {
                        *shell.defaults_revision.write() += 1;
                        shell.request_redraw_all = true;
                        shell.ping.ping();
                    }
                    if events.iter().any(|event| event.restyles()) {
                        let resolved = resolve(&load_appearance(), load_dominant());
                        if *shell.resolved.peek() != resolved {
                            shell.resolved.set(resolved);
                            for context in shell
                                .surfaces
                                .iter_mut()
                                .filter_map(|surface| surface.context.as_mut())
                            {
                                context.restyle(resolved);
                            }
                            shell.request_redraw_all = true;
                            shell.ping.ping();
                        }
                    }
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|_| RunError::EventLoop("inserting the appearance watch source"))?;
    Ok(())
}

impl Runtime {
    pub(crate) fn connect() -> Result<Connected, RunError> {
        let conn = Connection::connect_to_env().map_err(|_| RunError::NoDisplay)?;
        let (globals, event_queue) = registry_queue_init::<Runtime>(&conn)?;
        let qh = event_queue.handle();
        let queue: Rc<RefCell<Vec<(ObjectId, super::Sent)>>> = Rc::new(RefCell::new(Vec::new()));
        let contexts = GlobalContexts::register();
        contexts.insert_context(Platform::new(|event| {
            tracing::debug!(?event, "BarrierKit: Freya's global event has no handler");
        }));
        let (ping, ping_source) =
            calloop::ping::make_ping().map_err(|_| RunError::EventLoop("calloop ping failed"))?;
        let (wake_ping, wake_source) = calloop::ping::make_ping()
            .map_err(|_| RunError::EventLoop("calloop wake ping failed"))?;
        let resolved = State::create_global(resolve(&load_appearance(), load_dominant()));
        let appmenu_manager = globals
            .bind::<OrgKdeKwinAppmenuManager, _, _>(&qh, 1..=1, ())
            .ok();
        if appmenu_manager.is_none() {
            tracing::info!("org_kde_kwin_appmenu_manager unavailable; continuing without it");
        }
        let focus_manager = globals
            .bind::<super::focus::InfinityFocusManagerV1, _, _>(&qh, 1..=4, ())
            .ok();
        let dialog_manager = globals.bind(&qh, 1..=1, ()).ok();
        let window_style = globals
            .bind::<super::window_style::InfinityWindowStyleV1, _, _>(&qh, 1..=8, ())
            .ok();
        let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
        // wp_fractional_scale_v1: a fractional buffer is only valid behind a wp_viewport.
        let fractional_manager = viewporter.as_ref().and_then(|_| {
            globals
                .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
                .ok()
        });
        if fractional_manager.is_none() {
            tracing::info!(
                "wp_fractional_scale_manager_v1 unavailable; continuing with integer scales"
            );
        }
        if viewporter.is_none() {
            tracing::info!("wp_viewporter unavailable; continuing with integer scales");
        }
        let seat = SeatState::new(&globals, &qh);
        let seat_handle = seat.seats().next();
        let display_ptr = conn.display().id().as_ptr().cast::<c_void>();
        let mut shell = Runtime {
            registry: RegistryState::new(&globals),
            compositor: CompositorState::bind(&globals, &qh)
                .map_err(|_| RunError::MissingGlobal("wl_compositor"))?,
            output: OutputState::new(&globals, &qh),
            xdg_shell: XdgShell::bind(&globals, &qh).ok(),
            surfaces: Vec::new(),
            graphics: {
                let graphics = Rc::new(GraphicsShared::new(display_ptr)?);
                #[cfg(feature = "video")]
                super::dmabuf::set_current(&graphics);
                graphics
            },
            display: display_ptr,
            seat,
            pointer: None,
            cursor_shape: {
                let manager = CursorShapeManager::bind(&globals, &qh).ok();
                if manager.is_none() {
                    tracing::info!(
                        "wp_cursor_shape_manager_v1 unavailable; using the default cursor"
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
            keyboard_layouts: Vec::new(),
            keyboard_layout_index: 0,
            keyboard_layout: None,
            needs_exit: false,
            recover_failed: false,
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
            focus_manager,
            dialog_manager,
            window_style,
            fractional_manager,
            viewporter,
            timers: Rc::new(RefCell::new(Vec::new())),
            seat_handle,
            last_input_serial: Rc::new(Cell::new(0)),
            last_input_surface: None,
            last_pointer_serial: Cell::new(0),
            resize_hover: None,
            pointer_pressed: false,
            pointer_surface: None,
            cursor_icon: CursorIcon::Default,
            touchpad_scrolling: false,
            touch_surfaces: std::collections::HashMap::new(),
            popup_commands: Rc::new(RefCell::new(Vec::new())),
            popup_next_id: Rc::new(Cell::new(1)),
            appearance_watch: None,
            resolved,
            pending_menu_addresses: Arc::new(Mutex::new(Vec::new())),
            clipboard: Clipboard::new(ping.clone()),
            text_input_manager: globals
                .bind::<ZwpTextInputManagerV3, _, _>(&qh, 1..=1, ())
                .ok(),
            ime: super::ime::Ime::default(),
            primary: Clipboard::new(ping.clone()),
            primary_device: sctk::primary_selection::PrimarySelectionManagerState::bind(
                &globals, &qh,
            )
            .ok()
            .map(super::primary::PrimaryDevice::new),
            activation: globals.bind::<XdgActivationV1, _, _>(&qh, 1..=1, ()).ok(),
            defaults_revision: State::create_global(0),
            clipboard_device: DataDeviceManagerState::bind(&globals, &qh)
                .ok()
                .map(ClipboardDevice::new),
            drag_target: None,
            pending_drop: None,
        };
        contexts.insert_context(freya_clipboard::prelude::Clipboard::create(Some(Box::new(
            shell.clipboard.clone(),
        ))));
        if let Some(seat) = shell.seat_handle.clone() {
            shell.attach_clipboard(&seat);
            shell.attach_text_input(&seat);
            shell.attach_primary(&seat);
        }
        Ok(Connected {
            conn,
            event_queue,
            runtime: shell,
            #[cfg(feature = "shell")]
            globals,
        })
    }

    pub(crate) fn step_wallpaper_fade(&mut self, index: usize) -> bool {
        let global = *self.resolved.peek();
        let window = &mut self.surfaces[index];
        let stepped = window
            .context
            .as_mut()
            .is_some_and(|context| context.step_wallpaper_fade(global));
        if stepped {
            window.app.poll();
            window.app.update();
            window.needs_redraw = true;
        }
        stepped
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
            if (window.app.poll() || window.app.accessibility_pending()) && window.app.update() {
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
        self.sync_ime();
        self.sync_cursor();
    }

    fn sync_cursor(&mut self) {
        if self.resize_hover.is_some() {
            return;
        }
        let Some(index) = self
            .pointer_surface
            .as_ref()
            .and_then(|surface| self.surface_for(surface))
        else {
            return;
        };
        let icon = self.surfaces[index].app.cursor_icon();
        if icon != self.cursor_icon {
            self.cursor_icon = icon;
            if let Some(device) = &self.cursor_shape_device {
                device.set_shape(self.last_pointer_serial.get(), cursor_shape_from_icon(icon));
            }
        }
    }

    pub(crate) fn redraw_all(&mut self) {
        let qh = self.qh.clone();
        for window in &mut self.surfaces {
            if window.needs_redraw {
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

    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let gone = Some(output.id());
        while let Some(index) = self
            .surfaces
            .iter()
            .position(|surface| surface.output == gone)
        {
            let owner = self.surfaces[index].surface.id();
            self.close_popups_of(&owner);
            if let Some(index) = self.surface_index_by_id(&owner) {
                self.surfaces.remove(index);
            }
        }
    }
}

delegate_compositor!(Runtime);
delegate_output!(Runtime);
delegate_registry!(Runtime);
delegate_xdg_shell!(Runtime);
delegate_xdg_window!(Runtime);
