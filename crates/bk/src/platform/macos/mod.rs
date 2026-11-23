use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::rc::Rc;
use std::time::Duration;

use calloop::{EventLoop, ping::make_ping};
use ctappearance::{Resolved, load_appearance, load_dominant, resolve};
use dispatch2::{DispatchQueue, DispatchTime};
use freya_core::integration::{
    AppComponent, GlobalContexts, KeyboardEventName, NodeId, Platform, PlatformEvent, UserEvent,
};
use freya_core::prelude::{
    AccessibilityFocusStrategy, Code, CursorIcon, Key, Modifiers, State, WritableUtils,
};
use freya_engine::prelude::{
    ColorType, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts, mtl,
    wrap_backend_render_target,
};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSApplicationActivationPolicy, NSApplicationPresentationOptions, NSColor,
    NSEvent, NSEventModifierFlags, NSEventType, NSTitlebarSeparatorStyle, NSToolbar, NSWindow,
    NSWindowDelegate, NSWindowOrderingMode, NSWindowStyleMask, NSWindowTitleVisibility,
    NSWindowToolbarStyle,
};
use objc2_core_foundation::{
    CFFileDescriptor, CFFileDescriptorContext, CFRunLoop, CGSize, kCFRunLoopCommonModes,
};
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable,
    MTLPixelFormat,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer, kCACornerCurveContinuous};

use super::app::{AppInstance, Host, InstanceRole};
use super::backdrop::{Pass, render_pass, render_passes};
use super::clipboard::Clipboard;
use super::popup::PopupCommand;
use super::timer::{Wake, WakeQueue};
use super::window::{
    OpenUtility, WindowCommand, WindowControls, WindowOptions, WindowStatus, WindowStyle,
};
use super::{EditShortcut, ObjectId, RunError, Sent, calloop::ping::Ping};

mod frame;
mod input;
mod menu;
mod place;
mod popups;
mod view;

use popups::PopupState;
use view::{ContentView, Toplevel, WindowDelegate};

thread_local! {
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    static LOOP: RefCell<Option<EventLoop<'static, ()>>> = const { RefCell::new(None) };
}

// Callbacks of AppKit reach the runner through this cell. A call made while it is borrowed (a window the runner is
// closing tells its delegate) finds it busy and does nothing.
pub(super) fn with_runtime<R>(f: impl FnOnce(&mut Runtime) -> R) -> Option<R> {
    RUNTIME.with(|cell| cell.try_borrow_mut().ok()?.as_mut().map(f))
}

struct Gpu {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    context: DirectContext,
}

struct Window {
    id: u64,
    window: Retained<NSWindow>,
    view: Retained<ContentView>,
    layer: Retained<CAMetalLayer>,
    _delegate: Retained<ProtocolObject<dyn NSWindowDelegate>>,
    app: AppInstance,
    controls: Option<WindowControls>,
    popup: Option<PopupState>,
    // The frame a borderless window had before it filled the screen.
    restore: Option<NSRect>,
    ime_area: Option<ImeArea>,
    // A window the process lives for: not a popup, a sheet or a utility window.
    main: bool,
    needs_redraw: bool,
}

pub(super) struct Runtime {
    mtm: MainThreadMarker,
    gpu: Gpu,
    windows: Vec<Window>,
    next_window: u64,
    queue: Rc<RefCell<Vec<(ObjectId, Sent)>>>,
    ping: Ping,
    wake: Wake,
    timers: super::timer::TimerQueue,
    scheduled: HashMap<u64, Rc<dyn Fn()>>,
    next_timer: u64,
    popup_commands: Rc<RefCell<Vec<PopupCommand>>>,
    popup_next_id: Rc<Cell<u64>>,
    last_input_serial: Rc<Cell<u32>>,
    resolved: State<Resolved>,
    defaults_revision: State<u64>,
    clipboard: Clipboard,
    primary: Clipboard,
    pointer_window: Option<u64>,
    cursor: CursorIcon,
}

// Where the focused text field wants the input method's candidates, in the window's logical pixels.
type ImeArea = (i32, i32, i32, i32);

enum Native {
    Close,
    Minimize,
    Zoom,
    FullScreen,
    Front,
    Title(String),
    Size(u32, u32),
    Drag,
}

fn native(id: u64, command: Native) {
    let Some(window) =
        with_runtime(|runtime| runtime.window(id).map(|w| w.window.clone())).flatten()
    else {
        return;
    };
    match command {
        Native::Close => {
            if let Some(parent) = window.sheetParent() {
                parent.endSheet(&window);
            }
            window.close();
        }
        Native::Minimize => window.miniaturize(None),
        Native::Zoom => window.zoom(None),
        Native::FullScreen if window.styleMask().contains(NSWindowStyleMask::Titled) => {
            window.toggleFullScreen(None)
        }
        Native::FullScreen => {
            with_runtime(|runtime| runtime.toggle_borderless_fullscreen(id));
        }
        Native::Front => window.makeKeyAndOrderFront(None),
        Native::Title(title) => window.setTitle(&NSString::from_str(&title)),
        Native::Size(width, height) => {
            window.setContentSize(NSSize::new(f64::from(width), f64::from(height)))
        }
        Native::Drag => {
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            if let Some(press) = NSApplication::sharedApplication(mtm).currentEvent()
                && press.r#type() == NSEventType::LeftMouseDown
            {
                window.performWindowDragWithEvent(&press);
            }
        }
    }
}

fn round_corners(layer: &CAMetalLayer) {
    layer.setOpaque(false);
    layer.setCornerRadius(f64::from(ctdesign::CORNER_WINDOW.fitted));
    // SAFETY: AppKit's own constant.
    layer.setCornerCurve(unsafe { kCACornerCurveContinuous });
    layer.setMasksToBounds(true);
}

fn later(id: u64, command: Native) {
    DispatchQueue::main().exec_async(move || native(id, command));
}

impl Runtime {
    fn new(mtm: MainThreadMarker, ping: Ping, wake: Wake) -> Result<Self, RunError> {
        let device = MTLCreateSystemDefaultDevice().ok_or(RunError::Skia)?;
        let queue = device.newCommandQueue().ok_or(RunError::Skia)?;
        // SAFETY: the pointers are the live device and queue; Skia retains both.
        let backend = unsafe {
            mtl::BackendContext::new(
                Retained::as_ptr(&device) as mtl::Handle,
                Retained::as_ptr(&queue) as mtl::Handle,
            )
        };
        let context = direct_contexts::make_metal(&backend, None).ok_or(RunError::Skia)?;
        let contexts = GlobalContexts::register();
        contexts.insert_context(Platform::new(|event| {
            tracing::debug!(?event, "BarrierKit: Freya's global event has no handler");
        }));
        let clipboard = Clipboard::system();
        contexts.insert_context(freya_clipboard::prelude::Clipboard::create(Some(
            clipboard.provider(),
        )));
        Ok(Self {
            mtm,
            gpu: Gpu {
                device,
                queue,
                context,
            },
            windows: Vec::new(),
            next_window: 1,
            queue: Rc::new(RefCell::new(Vec::new())),
            ping,
            wake,
            timers: Rc::new(RefCell::new(Vec::new())),
            scheduled: HashMap::new(),
            next_timer: 0,
            popup_commands: Rc::new(RefCell::new(Vec::new())),
            popup_next_id: Rc::new(Cell::new(1)),
            last_input_serial: Rc::new(Cell::new(0)),
            resolved: State::create_global(resolve(&load_appearance(), load_dominant())),
            defaults_revision: State::create_global(0),
            clipboard,
            primary: Clipboard::primary(),
            pointer_window: None,
            cursor: CursorIcon::Default,
        })
    }

    fn host(&self) -> Host {
        Host {
            resolved: self.resolved,
            ping: self.ping.clone(),
            queue: self.queue.clone(),
            popup_commands: self.popup_commands.clone(),
            popup_next_id: self.popup_next_id.clone(),
            last_input_serial: self.last_input_serial.clone(),
            timers: self.timers.clone(),
            wake: self.wake.clone(),
            font_collection: crate::fonts::collection(),
            font_manager: crate::fonts::manager(),
            defaults_revision: self.defaults_revision,
            glass: false,
            backdrop: false,
            knobs: false,
        }
    }

    // `toggleFullScreen` needs a titled window: a borderless one fills the screen and hides the menu bar and the Dock itself.
    fn toggle_borderless_fullscreen(&mut self, id: u64) {
        let mtm = self.mtm;
        let Some(window) = self.window(id) else {
            return;
        };
        let Some(controls) = &window.controls else {
            return;
        };
        let mut status = controls.status;
        let (frame, radius, options) = match window.restore.take() {
            Some(frame) => (
                frame,
                f64::from(ctdesign::CORNER_WINDOW.fitted),
                NSApplicationPresentationOptions::Default,
            ),
            None => {
                let Some(screen) = window.window.screen() else {
                    return;
                };
                window.restore = Some(window.window.frame());
                (
                    screen.frame(),
                    0.0,
                    NSApplicationPresentationOptions::HideMenuBar
                        | NSApplicationPresentationOptions::HideDock,
                )
            }
        };
        status.write().fullscreen = window.restore.is_some();
        window.layer.setCornerRadius(radius);
        window.window.setFrame_display(frame, true);
        NSApplication::sharedApplication(mtm).setPresentationOptions(options);
        self.resized(id);
    }

    fn window(&mut self, id: u64) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }

    fn create_window(&mut self, app: AppComponent, options: WindowOptions, parent: Option<u64>) {
        let parent = parent.and_then(|id| self.window(id).map(|w| w.window.clone()));
        let sheet = parent.is_some() && options.modal;
        let utility = parent.is_some();
        let id = self.next_window;
        self.next_window += 1;
        let fixed = options.min_size.is_some() && options.min_size == options.max_size;
        let fixed_width = matches!(
            (options.min_size, options.max_size),
            (Some((min, _)), Some((max, _))) if min > 0 && min == max
        );
        let window_style = if utility {
            WindowStyle::TitleBar
        } else {
            options.window_style
        };
        let (mut style, decoration) = frame::frame(window_style, !fixed);
        if sheet {
            style = NSWindowStyleMask::Titled;
        }
        let (width, height) = options.size;
        let frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(f64::from(width), f64::from(height)),
        );
        let window = Retained::into_super(Toplevel::new(self.mtm, frame, style));
        // SAFETY: the runner owns the window; `setReleasedWhenClosed(false)` keeps it valid after close.
        unsafe { window.setReleasedWhenClosed(false) };
        window.setTitle(&NSString::from_str(&options.title));
        match window_style {
            WindowStyle::TitleBar => {}
            WindowStyle::HiddenTitleBar => {
                window.setTitlebarAppearsTransparent(true);
                window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
                window.setToolbar(Some(&NSToolbar::new(self.mtm)));
                window.setToolbarStyle(NSWindowToolbarStyle::Unified);
                window.setTitlebarSeparatorStyle(NSTitlebarSeparatorStyle::None);
            }
            WindowStyle::Plain => {
                window.setOpaque(false);
                window.setBackgroundColor(Some(&NSColor::clearColor()));
                window.setHasShadow(true);
            }
        }
        if let Some((w, h)) = options.min_size {
            window.setContentMinSize(NSSize::new(f64::from(w), f64::from(h)));
        }
        if let Some((w, h)) = options.max_size.filter(|(w, h)| *w > 0 && *h > 0) {
            window.setContentMaxSize(NSSize::new(f64::from(w), f64::from(h)));
        }
        let delegate = WindowDelegate::new(self.mtm, id);
        window.setDelegate(Some(&delegate));
        let view = ContentView::new(self.mtm, id, frame);
        let layer = self.configure_layer(&view);
        if window_style == WindowStyle::Plain {
            round_corners(&layer);
        }
        window.setContentView(Some(&view));
        self.follow_scheme(&window);
        window.center();

        let status = State::create_global(WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration,
            fixed,
        });
        let controls = WindowControls {
            queue: Rc::new(RefCell::new(Vec::new())),
            ping: self.ping.clone(),
            pointer_serial: Rc::new(Cell::new(0)),
            status,
            title: options.title.clone(),
            fixed,
            fixed_width,
            surface_id: ObjectId::from_raw(id),
            focus_bar: State::create_global(false),
            focus_bar_progress: State::create_global(0.0),
            focus_overlay: false,
            focus_bar_height: State::create_global(ctdesign::MENUBAR_HEIGHT as f32),
            desktop: State::create_global(None),
            fullscreen_bar: State::create_global(options.fullscreen_bar),
            fullscreen_push: State::create_global(options.fullscreen_push),
            overlay_titlebar_height: State::create_global(0.0),
            hud: options.hud,
            panel: utility,
            ime_preedit: State::create_global(String::new()),
            primary: self.primary.clone(),
            clipboard: self.clipboard.clone(),
        };
        let menu = options.menu.clone().map(|menu| {
            crate::main_menu::MenuHandle::new(
                menu,
                Some(crate::platform::WakeHandle::from_ping(
                    self.wake.ping.clone(),
                )),
            )
        });
        let scale = window.backingScaleFactor();
        let physical = (
            (f64::from(width) * scale) as u32,
            (f64::from(height) * scale) as u32,
        );
        let instance = AppInstance::new(
            &self.host(),
            ObjectId::from_raw(id),
            app,
            scale,
            physical,
            InstanceRole::Window {
                controls: Box::new(controls.clone()),
                menu: menu.clone(),
                theme: options.theme.clone(),
                resolved: State::create_global(*self.resolved.peek()),
            },
            |_| {},
        );
        self.windows.push(Window {
            id,
            window: window.clone(),
            view,
            layer,
            _delegate: delegate,
            app: instance,
            controls: Some(controls),
            popup: None,
            restore: None,
            ime_area: None,
            main: !utility,
            needs_redraw: true,
        });
        if let Some(menu) = menu {
            menu::register(id, menu);
        }
        match &parent {
            Some(parent) if sheet => {
                parent.beginSheet_completionHandler(&window, None);
            }
            Some(parent) => {
                // SAFETY: both are live windows the runner owns.
                unsafe { parent.addChildWindow_ordered(&window, NSWindowOrderingMode::Above) };
                window.makeKeyAndOrderFront(None);
            }
            None => window.makeKeyAndOrderFront(None),
        }
        self.resized(id);
    }

    // The system chrome and the effect views follow the kit's scheme, not the system's, until M4 reads it.
    fn follow_scheme(&self, window: &NSWindow) {
        // SAFETY: both names are AppKit's own constants.
        let name = unsafe {
            if self.resolved.peek().scheme.is_dark() {
                NSAppearanceNameDarkAqua
            } else {
                NSAppearanceNameAqua
            }
        };
        window.setAppearance(NSAppearance::appearanceNamed(name).as_deref());
    }

    fn configure_layer(&self, view: &ContentView) -> Retained<CAMetalLayer> {
        let layer = view.layer().expect("the content view is layer-backed");
        // SAFETY: `ContentView::makeBackingLayer` returns a CAMetalLayer.
        let layer: Retained<CAMetalLayer> =
            unsafe { Retained::cast_unchecked::<CAMetalLayer>(layer) };
        layer.setDevice(Some(&self.gpu.device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        // Skia's blend modes need a texture it may read.
        layer.setFramebufferOnly(false);
        layer.setPresentsWithTransaction(false);
        let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
        layer.setColorspace(srgb.as_deref());
        layer
    }

    pub(super) fn resized(&mut self, id: u64) {
        let Some(window) = self.window(id) else {
            return;
        };
        let size = window.view.bounds().size;
        let scale = window.window.backingScaleFactor();
        let physical = (
            (size.width * scale).round() as u32,
            (size.height * scale).round() as u32,
        );
        if physical.0 == 0 || physical.1 == 0 {
            return;
        }
        window.layer.setContentsScale(scale);
        window
            .layer
            .setDrawableSize(CGSize::new(f64::from(physical.0), f64::from(physical.1)));
        window.app.resize(physical, scale);
        window.needs_redraw = true;
        self.draw(id);
    }

    pub(super) fn push(&mut self, id: u64, event: PlatformEvent) {
        if matches!(event, PlatformEvent::Mouse { .. }) {
            self.pointer_window = Some(id);
        }
        if let Some(window) = self.window(id) {
            window.app.push(event);
        }
    }

    pub(super) fn text_field_focused(&mut self, id: u64) -> bool {
        self.window(id)
            .is_some_and(|window| window.app.text_field_focused())
    }

    pub(super) fn ime_area(&mut self, id: u64) -> Option<ImeArea> {
        self.window(id)?.ime_area
    }

    pub(super) fn set_preedit(&mut self, id: u64, text: String) {
        if let Some(controls) = self.window(id).and_then(|window| window.controls.as_ref()) {
            let mut preedit = controls.ime_preedit;
            preedit.set_if_modified(text);
        }
        self.ping.ping();
    }

    // An input method's committed text reaches the field as the key that types it, as on Wayland.
    pub(super) fn commit_text(&mut self, id: u64, text: String) {
        self.set_preedit(id, String::new());
        let Some(window) = self.window(id) else {
            return;
        };
        for name in [KeyboardEventName::KeyDown, KeyboardEventName::KeyUp] {
            window.app.push(PlatformEvent::Keyboard {
                name,
                key: Key::Character(text.clone()),
                code: Code::Unidentified,
                modifiers: Modifiers::empty(),
            });
        }
    }

    pub(super) fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    // A toplevel looks active while it is the main window, whichever of its popups holds the keyboard.
    pub(super) fn main_changed(&mut self, id: u64) {
        let Some(window) = self.window(id) else {
            return;
        };
        let Some(controls) = &window.controls else {
            return;
        };
        let activated = window.window.isMainWindow();
        if controls.status.peek().activated != activated {
            let mut status = controls.status;
            status.write().activated = activated;
            window.app.set_focused(activated);
            window.needs_redraw = true;
        }
    }

    pub(super) fn key_changed(&mut self, id: u64, key: bool) {
        if key {
            menu::show(id);
        }
        let popup = self.window(id).is_some_and(|window| window.popup.is_some());
        if popup && !key {
            DispatchQueue::main().exec_async(|| {
                with_runtime(|runtime| runtime.dismiss_grab_without_keyboard());
            });
        }
    }

    pub(super) fn closed(&mut self, id: u64) {
        let owned: Vec<_> = self
            .windows
            .iter()
            .filter(|window| window.popup.as_ref().is_some_and(|popup| popup.owner == id))
            .filter_map(|window| window.popup.as_ref().map(|popup| popup.id))
            .collect();
        for popup in owned {
            self.close_popup(popup);
        }
        self.windows.retain(|window| window.id != id);
        menu::forget(id);
        if !self.windows.iter().any(|window| window.main) {
            stop(self.mtm);
        }
    }

    fn draw(&mut self, id: u64) {
        let Some(index) = self.windows.iter().position(|window| window.id == id) else {
            return;
        };
        let gpu = &mut self.gpu;
        let window = &mut self.windows[index];
        window.needs_redraw = false;
        let scale = window.window.backingScaleFactor();
        let resolved = *window.app.resolved.read();
        let background = if window.popup.is_some() {
            crate::theme::color(ctdesign::Rgba::TRANSPARENT)
        } else {
            let base =
                crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel);
            crate::theme::color(base)
        };
        window.app.layout();
        window.app.settle_sized_reactions();
        let Some(drawable) = window.layer.nextDrawable() else {
            window.needs_redraw = true;
            return;
        };
        let size = window.layer.drawableSize();
        // SAFETY: the texture is the drawable's own and lives through this frame; Skia retains it.
        let info =
            unsafe { mtl::TextureInfo::new(Retained::as_ptr(&drawable.texture()) as mtl::Handle) };
        let target =
            backend_render_targets::make_mtl((size.width as i32, size.height as i32), &info);
        let Some(mut surface) = wrap_backend_render_target(
            &mut gpu.context,
            &target,
            SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            None,
            None,
        ) else {
            return;
        };
        let app = &mut window.app;
        let no_chromes: [Vec<NodeId>; 0] = [];
        render_passes(&mut app.tree, &no_chromes, |pass, tree| {
            if pass == Pass::Content {
                render_pass(
                    &mut app.font_collection,
                    &app.font_manager,
                    tree,
                    surface.canvas(),
                    scale,
                    (0, 0),
                    background,
                );
            }
        });
        gpu.context.flush_and_submit();
        drop(surface);
        if let Some(buffer) = gpu.queue.commandBuffer() {
            let drawable: Retained<ProtocolObject<dyn MTLDrawable>> = (&drawable).into();
            buffer.presentDrawable(&drawable);
            buffer.commit();
        }
        app.ticker_sender.notify();
    }

    fn redraw_all(&mut self) {
        let pending: Vec<u64> = self
            .windows
            .iter()
            .filter(|window| window.needs_redraw)
            .map(|window| window.id)
            .collect();
        for id in pending {
            self.draw(id);
        }
    }

    fn poll_all(&mut self) {
        for window in &mut self.windows {
            if (window.app.poll() || window.app.accessibility_pending()) && window.app.update() {
                window.needs_redraw = true;
            }
        }
        self.process_window_commands();
        self.process_popup_commands();
        self.sync_cursor();
        menu::sync();
    }

    fn sync_cursor(&mut self) {
        let Some(id) = self.pointer_window else {
            return;
        };
        let Some(window) = self.window(id) else {
            return;
        };
        let icon = window.app.cursor_icon();
        let (native, view) = (window.window.clone(), window.view.clone());
        if icon == self.cursor {
            return;
        }
        self.cursor = icon;
        native.invalidateCursorRectsForView(&view);
    }

    fn process_window_commands(&mut self) {
        let mut open = Vec::new();
        let mut areas = Vec::new();
        for window in &self.windows {
            let Some(controls) = &window.controls else {
                continue;
            };
            let commands = std::mem::take(&mut *controls.queue.borrow_mut());
            for command in commands {
                let id = window.id;
                match command {
                    WindowCommand::Close => later(id, Native::Close),
                    WindowCommand::Minimize => later(id, Native::Minimize),
                    WindowCommand::Move(_) => later(id, Native::Drag),
                    WindowCommand::ToggleMaximize => later(id, Native::Zoom),
                    WindowCommand::ToggleFullscreen => later(id, Native::FullScreen),
                    WindowCommand::Present(_) | WindowCommand::MakeKey => later(id, Native::Front),
                    WindowCommand::SetTitle(title) => later(id, Native::Title(title.to_string())),
                    WindowCommand::Fit((width, height)) => later(id, Native::Size(width, height)),
                    WindowCommand::ImeCursor(area) => areas.push((id, area)),
                    WindowCommand::Open(window) => open.push((window, None)),
                    WindowCommand::Sheet(window) => open.push((window, Some(id))),
                    _ => {}
                }
            }
        }
        for (id, area) in areas {
            if let Some(window) = self.window(id) {
                window.ime_area = Some(area);
            }
        }
        for (window, parent) in open {
            let (app, options) = *window;
            self.create_window(app, options, parent);
        }
    }

    fn install_timers(&mut self) {
        let pending = std::mem::take(&mut *self.timers.borrow_mut());
        for (delay, callback) in pending {
            let id = self.next_timer;
            self.next_timer += 1;
            self.scheduled.insert(id, callback);
            let when = DispatchTime::NOW.time(delay.as_nanos().min(i64::MAX as u128) as i64);
            let _ = DispatchQueue::main().after(when, move || fire(id));
        }
    }

    fn on_ping(&mut self) {
        let events = std::mem::take(&mut *self.queue.borrow_mut());
        for (owner, event) in events {
            if let Sent::Kit(event) = event {
                if event.is::<OpenUtility>() {
                    if let Ok(open) = event.downcast::<OpenUtility>() {
                        let open = *open;
                        self.create_window(open.app, open.options, Some(owner.raw()));
                    }
                    continue;
                }
                if let Ok(shortcut) = event.downcast::<EditShortcut>()
                    && let Some(window) = self.window(owner.raw())
                {
                    window.app.requested_focus =
                        Some(AccessibilityFocusStrategy::Node(shortcut.field));
                    window
                        .app
                        .pending_shortcuts
                        .push(Key::Character(shortcut.key));
                    window.needs_redraw = true;
                }
                continue;
            }
            let Some(window) = self.window(owner.raw()) else {
                continue;
            };
            match event {
                Sent::User(UserEvent::RequestRedraw) => window.needs_redraw = true,
                Sent::User(UserEvent::FocusAccessibilityNode(strategy)) => {
                    window.app.requested_focus = Some(strategy);
                    window.needs_redraw = true;
                }
                _ => {}
            }
        }
        self.poll_all();
        self.install_timers();
        self.redraw_all();
    }
}

fn fire(id: u64) {
    let Some(callback) = with_runtime(|runtime| runtime.scheduled.remove(&id)).flatten() else {
        return;
    };
    callback();
    with_runtime(|runtime| {
        runtime.poll_all();
        runtime.install_timers();
        runtime.redraw_all();
    });
}

fn woken() {
    let Some(callbacks) = with_runtime(|runtime| runtime.wake.callbacks.borrow().clone()) else {
        return;
    };
    for callback in callbacks {
        callback();
    }
    with_runtime(|runtime| {
        runtime.poll_all();
        runtime.redraw_all();
    });
}

// `NSApplication::stop` takes effect with the next event, so a closed last window needs one posted.
fn stop(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    app.stop(None);
    if let Some(event) = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
        NSEventType::ApplicationDefined,
        NSPoint::new(0.0, 0.0),
        NSEventModifierFlags::empty(),
        0.0,
        0,
        None,
        0,
        0,
        0,
    ) {
        app.postEvent_atStart(&event, true);
    }
}

// The loop of `calloop` serves the pings; AppKit's run loop wakes it when its poll descriptor is readable.
fn serve_loop(event_loop: EventLoop<'static, ()>) -> Result<(), RunError> {
    let fd = event_loop.as_raw_fd();
    LOOP.with(|slot| *slot.borrow_mut() = Some(event_loop));
    extern "C-unwind" fn dispatch(
        descriptor: *mut CFFileDescriptor,
        _: usize,
        _: *mut std::ffi::c_void,
    ) {
        LOOP.with(|slot| {
            if let Some(event_loop) = slot.borrow_mut().as_mut() {
                let _ = event_loop.dispatch(Some(Duration::ZERO), &mut ());
            }
        });
        // SAFETY: CoreFoundation passes the descriptor that fired, alive for the call.
        unsafe { (*descriptor).enable_call_backs(1) };
    }
    let mut context = CFFileDescriptorContext {
        version: 0,
        info: std::ptr::null_mut(),
        retain: None,
        release: None,
        copyDescription: None,
    };
    // SAFETY: the descriptor belongs to the loop kept in `LOOP` for the whole run, and `close_on_invalidate` is false.
    let descriptor =
        unsafe { CFFileDescriptor::new(None, fd, false, Some(dispatch), &mut context) }
            .ok_or(RunError::EventLoop("CFFileDescriptor::new failed"))?;
    descriptor.enable_call_backs(1);
    let source = CFFileDescriptor::new_run_loop_source(None, Some(&descriptor), 0)
        .ok_or(RunError::EventLoop("the run loop source was not created"))?;
    CFRunLoop::current()
        .ok_or(RunError::EventLoop("no current run loop"))?
        .add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
    std::mem::forget((descriptor, source));
    Ok(())
}

pub(super) fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
    let mtm = MainThreadMarker::new().ok_or(RunError::EventLoop("not on the main thread"))?;
    let application = NSApplication::sharedApplication(mtm);
    application.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    menu::install(mtm, &application, &options.app_name);

    let event_loop: EventLoop<'static, ()> = EventLoop::try_new()
        .map_err(|_| RunError::EventLoop("calloop EventLoop::try_new failed"))?;
    let (ping, ping_source) =
        make_ping().map_err(|_| RunError::EventLoop("calloop ping failed"))?;
    let (wake_ping, wake_source) =
        make_ping().map_err(|_| RunError::EventLoop("calloop wake ping failed"))?;
    event_loop
        .handle()
        .insert_source(ping_source, |_, _, _| {
            with_runtime(|runtime| runtime.on_ping());
        })
        .map_err(|_| RunError::EventLoop("inserting the Freya ping source"))?;
    event_loop
        .handle()
        .insert_source(wake_source, |_, _, _| woken())
        .map_err(|_| RunError::EventLoop("inserting the wake ping source"))?;
    let wake = Wake {
        ping: wake_ping,
        callbacks: WakeQueue::default(),
    };
    let runtime = Runtime::new(mtm, ping.clone(), wake)?;
    RUNTIME.with(|cell| *cell.borrow_mut() = Some(runtime));
    serve_loop(event_loop)?;

    with_runtime(|runtime| runtime.create_window(app, options, None));
    ping.ping();
    application.activate();
    application.run();
    Ok(())
}
