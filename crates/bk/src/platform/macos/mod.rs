use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::rc::Rc;
use std::time::Duration;

use calloop::{EventLoop, ping::make_ping};
use ctappearance::material::Material;
use ctappearance::{Resolved, load_dominant, resolve};
use dispatch2::{DispatchQueue, DispatchTime};
use freya_core::integration::{
    AppComponent, GlobalContexts, KeyboardEventName, NodeId, Platform, PlatformEvent, UserEvent,
};
use freya_core::prelude::{
    AccessibilityFocusStrategy, Code, CursorIcon, Key, Modifiers, State, WritableUtils,
};
use freya_engine::prelude::{
    ColorType, DirectContext, Surface, SurfaceOrigin, backend_render_targets, direct_contexts, mtl,
    wrap_backend_render_target,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSApplicationActivationPolicy, NSApplicationPresentationOptions,
    NSAutoresizingMaskOptions, NSColor, NSEvent, NSEventModifierFlags, NSEventType,
    NSLayoutAttribute, NSTitlebarAccessoryViewController, NSTitlebarSeparatorStyle, NSToolbar,
    NSView, NSVisualEffectBlendingMode, NSWindow, NSWindowButton, NSWindowDelegate,
    NSWindowOrderingMode, NSWindowStyleMask, NSWindowTitleVisibility, NSWindowToolbarStyle,
};
use objc2_core_foundation::{
    CFFileDescriptor, CFFileDescriptorContext, CFRunLoop, CGSize, kCFRunLoopCommonModes,
};
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_foundation::{NSPoint, NSRect, NSRunLoop, NSRunLoopCommonModes, NSSize, NSString};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable,
    MTLPixelFormat,
};
use objc2_quartz_core::{CADisplayLink, CAMetalDrawable, CAMetalLayer, kCACornerCurveContinuous};

use super::app::{AppInstance, Host, InstanceRole};
use super::backdrop::{
    Effect as Declared, Pass, PlacedGlass, Placement, Requests, render_pass, render_passes,
};
use super::clipboard::Clipboard;
use super::popup::PopupCommand;
use super::timer::{Wake, WakeQueue};
use super::window::{
    OpenUtility, WindowCommand, WindowControls, WindowOptions, WindowStatus, WindowStyle,
};
use super::window_style::{Bar as TitlebarBar, ScrollEdgeEffectStyle};
use super::{EditShortcut, ObjectId, RunError, Sent, calloop::ping::Ping};

#[cfg(feature = "a11y")]
pub(super) mod accessibility;
mod arrow;
mod blur;
pub(crate) mod clipboard;
mod delegate;
mod frame;
mod input;
pub(crate) mod instance;
pub(crate) mod keychain;
mod menu;
pub(crate) mod notifications;
pub(crate) mod panels;
pub(crate) mod pasteboard;
mod place;
mod popups;
mod status;
mod system;
mod view;

use popups::PopupState;
use view::{BarView, ChromeView, ContentView, Effect, Toplevel, WindowDelegate};

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

// A declared glass: native effect views over the content and, above them, a Metal layer with what the kit draws over the glass.
struct ChromeLayer {
    wanted: Wanted,
    view: Retained<ChromeView>,
    layer: Retained<CAMetalLayer>,
    effects: Vec<Retained<NSView>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Backing {
    Visual(Material),
    Glass,
}

#[derive(Clone, Debug, PartialEq)]
struct Wanted {
    placement: Placement,
    requests: Requests,
    native: Option<Backing>,
    glasses: Vec<PlacedGlass>,
}

// The title bar row AppKit carries down with the menu bar in full screen, as a titlebar accessory.
struct Bar {
    accessory: Retained<NSTitlebarAccessoryViewController>,
    view: Retained<BarView>,
    revealed: bool,
}

struct Window {
    id: u64,
    window: Retained<NSWindow>,
    view: Retained<ContentView>,
    // The content view's parent: the effect view of a sidebar sits below the view.
    container: Option<Retained<NSView>>,
    bar: Option<Bar>,
    behind: Option<Retained<Effect>>,
    chromes: Vec<ChromeLayer>,
    layer: Retained<CAMetalLayer>,
    // Runs while draws are due, at most one per refresh of the window's display.
    link: Retained<CADisplayLink>,
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

impl ChromeLayer {
    fn remove(self) {
        self.view.removeFromSuperview();
        for effect in self.effects {
            effect.removeFromSuperview();
        }
    }
}

impl Window {
    fn place_bar(&self) {
        let (Some(bar), Some(container)) = (&self.bar, &self.container) else {
            return;
        };
        bar.view.align(container.bounds().size.height);
        for chrome in &self.chromes {
            let (_, top) = chrome.wanted.placement.origin;
            let (_, strip) = chrome.wanted.placement.size;
            let riding = bar.revealed && crate::titlebar::in_bar_row(top as f32, strip as f32);
            let parent: &NSView = if riding { &bar.view } else { container };
            let views = chrome.effects.iter().map(|effect| &**effect);
            for view in views.chain([&**chrome.view]) {
                // SAFETY: the view is alive; its parent is only compared, never kept.
                let current = unsafe { view.superview() };
                if current.is_none_or(|current| !std::ptr::eq(&*current, parent)) {
                    parent.addSubview(view);
                }
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.link.invalidate();
    }
}

pub(super) struct Runtime {
    mtm: MainThreadMarker,
    gpu: Gpu,
    windows: Vec<Window>,
    statuses: Vec<(u64, status::StatusBar)>,
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
    Present,
    Title(String),
    Size(u32, u32),
    Toolbar(TitlebarBar),
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
        Native::Present => {
            window.makeKeyAndOrderFront(None);
            if let Some(mtm) = MainThreadMarker::new() {
                // `activate()` leaves the app that is in front still in front (checked on macOS 27); this does not.
                #[allow(deprecated)]
                NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
            }
        }
        Native::Title(title) => window.setTitle(&NSString::from_str(&title)),
        // The empty toolbar of a HiddenTitleBar window centers the native lights in the kit's bar, as SwiftUI's `windowToolbarStyle` does.
        Native::Toolbar(TitlebarBar::Unified) if window.toolbar().is_some() => {
            window.setToolbarStyle(NSWindowToolbarStyle::Unified)
        }
        Native::Toolbar(TitlebarBar::UnifiedCompact) if window.toolbar().is_some() => {
            window.setToolbarStyle(NSWindowToolbarStyle::UnifiedCompact)
        }
        Native::Toolbar(_) => {}
        Native::Size(width, height) => {
            window.setContentSize(NSSize::new(f64::from(width), f64::from(height)))
        }
    }
}

fn frame_link(window: &NSWindow, view: &ContentView) -> Retained<CADisplayLink> {
    // SAFETY: `drawFrame:` is ContentView's method and takes the display link.
    let link = unsafe { window.displayLinkWithTarget_selector(view.as_ref(), sel!(drawFrame:)) };
    link.setPaused(true);
    // SAFETY: the main run loop outlives the link, which `Window` invalidates when it drops.
    unsafe { link.addToRunLoop_forMode(&NSRunLoop::mainRunLoop(), NSRunLoopCommonModes) };
    link
}

fn configure_metal_layer(
    device: &Retained<ProtocolObject<dyn MTLDevice>>,
    view: &NSView,
) -> Retained<CAMetalLayer> {
    let layer = view.layer().expect("the view is layer-backed");
    // SAFETY: the content and chrome views return a CAMetalLayer from `makeBackingLayer`.
    let layer: Retained<CAMetalLayer> = unsafe { Retained::cast_unchecked::<CAMetalLayer>(layer) };
    layer.setDevice(Some(device));
    layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    // Skia's blend modes need a texture it may read.
    layer.setFramebufferOnly(false);
    layer.setPresentsWithTransaction(false);
    // SAFETY: reading a CoreGraphics constant that is set when the framework loads.
    let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
    layer.setColorspace(srgb.as_deref());
    layer
}

// A frame to draw: the layer's next drawable and a Skia surface over its texture.
fn target(
    gpu: &mut Gpu,
    layer: &CAMetalLayer,
) -> Option<(Retained<ProtocolObject<dyn CAMetalDrawable>>, Surface)> {
    let drawable = layer.nextDrawable()?;
    let size = layer.drawableSize();
    // SAFETY: the texture is the drawable's own and lives through this frame; Skia retains it.
    let info =
        unsafe { mtl::TextureInfo::new(Retained::as_ptr(&drawable.texture()) as mtl::Handle) };
    let target = backend_render_targets::make_mtl((size.width as i32, size.height as i32), &info);
    let surface = wrap_backend_render_target(
        &mut gpu.context,
        &target,
        SurfaceOrigin::TopLeft,
        ColorType::BGRA8888,
        None,
        None,
    )?;
    Some((drawable, surface))
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
            statuses: Vec::new(),
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
            resolved: State::create_global(resolve(&system::appearance(), load_dominant())),
            defaults_revision: State::create_global(0),
            clipboard,
            primary: Clipboard::primary(),
            pointer_window: None,
            cursor: CursorIcon::Default,
        })
    }

    // The window and every popup follow the settings the system changed.
    fn refresh_appearance(&mut self) {
        let resolved = resolve(&system::appearance(), load_dominant());
        let mut own = self.resolved;
        own.set_if_modified(resolved);
        for window in &mut self.windows {
            let mut state = window.app.resolved;
            window.needs_redraw |= *state.peek() != resolved;
            state.set_if_modified(resolved);
        }
        for window in &self.windows {
            self.follow_scheme(&window.window);
        }
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
            backdrop: true,
            native_glass: blur::glass_available(),
            native_sheets: true,
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
        let delegate = WindowDelegate::object(self.mtm, id);
        window.setDelegate(Some(&delegate));
        let view = ContentView::new(self.mtm, id, frame);
        let layer = self.configure_layer(&view);
        if window_style == WindowStyle::Plain {
            round_corners(&layer);
        }
        let container = blur::container(self.mtm, frame);
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        container.addSubview(&view);
        window.setContentView(Some(&container));
        self.follow_scheme(&window);
        // The kit's Window menu lists its windows itself; AppKit would list each one twice.
        window.setExcludedFromWindowsMenu(true);
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
        let host = Host {
            glass: !options.opaque,
            ..self.host()
        };
        #[cfg_attr(not(feature = "a11y"), allow(unused_mut))]
        let mut instance = AppInstance::new(
            &host,
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
        #[cfg(feature = "a11y")]
        instance.attach_accessibility(&view, id);
        self.windows.push(Window {
            id,
            window: window.clone(),
            link: frame_link(&window, &view),
            view,
            container: Some(container),
            behind: None,
            chromes: Vec::new(),
            bar: None,
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
        if let Some(item) = &options.status_item {
            self.statuses
                .push((id, status::StatusBar::new(self.mtm, id, item)));
        }
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
        configure_metal_layer(&self.gpu.device, view)
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

    pub(super) fn trackpad(&mut self, id: u64, event: crate::gesture::Trackpad) {
        if let Some(window) = self.window(id) {
            window.app.trackpad(event);
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

    pub(super) fn attach_bar(
        &mut self,
        id: u64,
    ) -> Option<(
        Retained<NSWindow>,
        Retained<NSTitlebarAccessoryViewController>,
    )> {
        let mtm = self.mtm;
        let window = self.window(id)?;
        window.window.toolbar()?;
        let view = BarView::new(mtm, id, window.window.screen()?.frame().size.width);
        let accessory = NSTitlebarAccessoryViewController::new(mtm);
        accessory.setView(&view);
        accessory.setLayoutAttribute(NSLayoutAttribute::Left);
        window.bar = Some(Bar {
            accessory: accessory.clone(),
            view,
            revealed: false,
        });
        Some((window.window.clone(), accessory))
    }

    pub(super) fn detach_bar(
        &mut self,
        id: u64,
    ) -> Option<Retained<NSTitlebarAccessoryViewController>> {
        self.reveal_bar(id, false);
        Some(self.window(id)?.bar.take()?.accessory)
    }

    pub(super) fn bar_window(&mut self, id: u64) -> Option<Retained<NSWindow>> {
        let window = self.window(id)?;
        window.bar.as_ref()?;
        window
            .window
            .standardWindowButton(NSWindowButton::CloseButton)?
            .window()
    }

    pub(super) fn reveal_bar(&mut self, id: u64, revealed: bool) {
        let Some(window) = self.window(id) else {
            return;
        };
        if let Some(bar) = window.bar.as_mut().filter(|bar| bar.revealed != revealed) {
            bar.revealed = revealed;
            // AppKit resizes the container with the toolbar, so the chromes are built again for its new height.
            window.chromes.drain(..).for_each(ChromeLayer::remove);
            window.needs_redraw = true;
            // The menu bar is down, as Malaysia's focus bar: the title bar makes room for the lights.
            if let Some(controls) = &window.controls {
                let mut down = controls.focus_bar;
                down.set(revealed);
            }
        }
        window.place_bar();
        self.poll_all();
        self.redraw_all();
    }

    pub(super) fn toolbar(&mut self, id: u64) -> Option<Retained<NSToolbar>> {
        self.window(id)?.window.toolbar()
    }

    // The green light of a titled window calls `toggleFullScreen` inside AppKit, so the kit learns of it here.
    pub(super) fn fullscreen_changed(&mut self, id: u64) {
        let Some(window) = self.window(id) else {
            return;
        };
        let Some(controls) = &window.controls else {
            return;
        };
        let fullscreen = window
            .window
            .styleMask()
            .contains(NSWindowStyleMask::FullScreen);
        if controls.status.peek().fullscreen != fullscreen {
            let mut status = controls.status;
            status.write().fullscreen = fullscreen;
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

    fn window_of(&mut self, id: &ObjectId) -> Option<Retained<NSWindow>> {
        self.window(id.raw()).map(|found| found.window.clone())
    }

    // A document with unsaved changes keeps its window and asks first (crate::document).
    pub(super) fn should_close(&mut self, id: u64) -> bool {
        let intercepted = crate::document::intercept_close(&ObjectId::from_raw(id));
        if intercepted {
            self.poll_all();
            self.redraw_all();
        }
        !intercepted
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
        self.statuses.retain(|(owner, _)| *owner != id);
        if !self.windows.iter().any(|window| window.main) {
            stop(self.mtm);
        }
    }

    fn draw(&mut self, id: u64) {
        let Some(index) = self.windows.iter().position(|window| window.id == id) else {
            return;
        };
        {
            let window = &mut self.windows[index];
            window.needs_redraw = false;
            window.app.layout();
            window.app.settle_sized_reactions();
        }
        self.sync_chromes(index);
        let gpu = &mut self.gpu;
        let window = &mut self.windows[index];
        let scale = window.window.backingScaleFactor();
        let resolved = *window.app.resolved.read();
        let background = if window.popup.is_some() {
            crate::theme::color(ctdesign::Rgba::TRANSPARENT)
        } else {
            let base =
                crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel);
            crate::theme::color(base)
        };
        let Some((content, mut surface)) = target(gpu, &window.layer) else {
            window.needs_redraw = true;
            return;
        };
        let mut presented = Vec::new();
        let app = &mut window.app;
        let nodes: Vec<Rc<[NodeId]>> = app
            .chromes
            .iter()
            .map(|chrome| chrome.nodes.clone())
            .collect();
        let layers = &window.chromes;
        render_passes(&mut app.tree, &nodes, |pass, tree| match pass {
            Pass::Content => render_pass(
                &mut app.font_collection,
                &app.font_manager,
                tree,
                surface.canvas(),
                scale,
                (0, 0),
                background,
            ),
            Pass::Chrome(index) => {
                let Some(chrome) = layers.get(index) else {
                    return;
                };
                let Some((drawable, mut own)) = target(gpu, &chrome.layer) else {
                    return;
                };
                render_pass(
                    &mut app.font_collection,
                    &app.font_manager,
                    tree,
                    own.canvas(),
                    scale,
                    chrome.wanted.placement.origin,
                    freya_core::prelude::Color::TRANSPARENT,
                );
                gpu.context.flush_and_submit();
                presented.push(drawable);
            }
        });
        gpu.context.flush_and_submit();
        drop(surface);
        presented.push(content);
        // The layers of a window with chromes present inside one Core Animation transaction, so that the content and the glass over it move together.
        let together = !layers.is_empty();
        window.layer.setPresentsWithTransaction(together);
        for chrome in layers {
            chrome.layer.setPresentsWithTransaction(together);
        }
        if let Some(buffer) = gpu.queue.commandBuffer() {
            if together {
                buffer.commit();
                buffer.waitUntilScheduled();
                for drawable in &presented {
                    let drawable: Retained<ProtocolObject<dyn MTLDrawable>> = drawable.into();
                    drawable.present();
                }
            } else {
                for drawable in &presented {
                    let drawable: Retained<ProtocolObject<dyn MTLDrawable>> = drawable.into();
                    buffer.presentDrawable(&drawable);
                }
                buffer.commit();
            }
        }
        app.ticker_sender.notify();
    }

    // What the kit declared since the last frame becomes native views: the effects over the content and the chrome above them.
    fn sync_chromes(&mut self, index: usize) {
        let mtm = self.mtm;
        let device = self.gpu.device.clone();
        let window = &mut self.windows[index];
        let Some(container) = window.container.clone() else {
            return;
        };
        let scale = window.window.backingScaleFactor();
        let wanted: Vec<Wanted> = window
            .app
            .chromes
            .iter()
            .map(|chrome| Wanted {
                placement: chrome.placement(scale as f32),
                requests: Requests::of(chrome, scale as f32),
                glasses: chrome.glasses(scale as f32),
                native: chrome.effects.iter().find_map(|(effect, _)| match effect {
                    Declared::Material { material, .. } => Some(Backing::Visual(*material)),
                    Declared::Glass { .. } => Some(Backing::Glass),
                    Declared::ScrollEdge { .. } => None,
                }),
            })
            .collect();
        if wanted
            == window
                .chromes
                .iter()
                .map(|chrome| chrome.wanted.clone())
                .collect::<Vec<_>>()
        {
            return;
        }
        window.chromes.drain(..).for_each(ChromeLayer::remove);
        let height = container.bounds().size.height;
        for wanted in wanted {
            let (x, y) = wanted.placement.origin;
            let (width, strip) = wanted.placement.size;
            let mut effects = Vec::new();
            if let Some(edge) = wanted.requests.scroll_edge {
                let rect = blur::edge_rect(edge.edge, (x, y), (width, strip), edge.depth);
                let effect = blur::effect(
                    mtm,
                    Material::Titlebar,
                    NSVisualEffectBlendingMode::WithinWindow,
                    0.0,
                );
                effect.setFrame(blur::frame(rect, height));
                if edge.style == ScrollEdgeEffectStyle::Soft {
                    blur::fade(&effect, edge.edge);
                }
                effects.push(Retained::into_super(Retained::into_super(effect)));
            }
            let radius = wanted.requests.backdrop.radius as f32;
            match wanted.native {
                Some(Backing::Visual(material)) => {
                    for &(left, top, w, h) in &wanted.requests.backdrop.rects {
                        let effect = blur::effect(
                            mtm,
                            material,
                            NSVisualEffectBlendingMode::WithinWindow,
                            radius,
                        );
                        effect.setFrame(blur::frame((x + left, y + top, w, h), height));
                        effects.push(Retained::into_super(Retained::into_super(effect)));
                    }
                }
                Some(Backing::Glass) => {
                    let strip_frame = blur::frame((x, y, width as i32, strip as i32), height);
                    if wanted.glasses.len() > 1 && wanted.requests.glass_spacing > 0 {
                        let members: Vec<_> = wanted
                            .glasses
                            .iter()
                            .map(|&((left, top, w, h), radius, clear)| {
                                let glass = blur::glass(mtm, radius as f32, clear);
                                glass.setFrame(blur::frame((left, top, w, h), f64::from(strip)));
                                glass
                            })
                            .collect();
                        let group = blur::glass_group(
                            mtm,
                            strip_frame,
                            &members,
                            f64::from(wanted.requests.glass_spacing),
                        );
                        effects.push(Retained::into_super(Retained::into_super(group)));
                    } else {
                        for &((left, top, w, h), radius, clear) in &wanted.glasses {
                            let glass = blur::glass(mtm, radius as f32, clear);
                            glass.setFrame(blur::frame((x + left, y + top, w, h), height));
                            effects.push(Retained::into_super(Retained::into_super(glass)));
                        }
                    }
                }
                None => {}
            }
            // A chrome is rebuilt only when its placement changes, so a taller window must keep its distance from the top.
            let anchor = NSAutoresizingMaskOptions::ViewMinYMargin;
            for effect in &effects {
                effect.setAutoresizingMask(anchor);
                container.addSubview(effect);
            }
            let view =
                ChromeView::new(mtm, blur::frame((x, y, width as i32, strip as i32), height));
            view.setAutoresizingMask(anchor);
            container.addSubview(&view);
            let layer = configure_metal_layer(&device, &view);
            layer.setOpaque(false);
            layer.setContentsScale(scale);
            layer.setDrawableSize(CGSize::new(
                f64::from(width) * scale,
                f64::from(strip) * scale,
            ));
            window.chromes.push(ChromeLayer {
                wanted,
                view,
                layer,
                effects,
            });
        }
        window.place_bar();
        window.needs_redraw = true;
    }

    // The first draw after an idle spell is immediate; later ones wait for the display link.
    fn redraw_all(&mut self) {
        let pending: Vec<u64> = self
            .windows
            .iter()
            .filter(|window| window.needs_redraw && window.link.isPaused())
            .map(|window| window.id)
            .collect();
        for id in pending {
            if let Some(window) = self.window(id) {
                window.link.setPaused(false);
            }
            self.draw(id);
        }
    }

    pub(super) fn frame_due(&mut self, id: u64) {
        let Some(window) = self.window(id) else {
            return;
        };
        if window.needs_redraw {
            self.draw(id);
        } else {
            window.link.setPaused(true);
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
        self.sync_glass();
        self.sync_cursor();
        menu::sync();
    }

    // The glass of a sidebar is a native view behind the window's own: the kit leaves its panel transparent.
    fn sync_glass(&mut self) {
        let mtm = self.mtm;
        for window in &mut self.windows {
            let (Some(container), Some(panels)) =
                (window.container.clone(), window.app.glass.take())
            else {
                continue;
            };
            if let Some(old) = window.behind.take() {
                old.removeFromSuperview();
            }
            window.needs_redraw = true;
            let Some(&rect) = panels.rects.first() else {
                continue;
            };
            let height = container.bounds().size.height;
            let effect = blur::effect(
                mtm,
                Material::Sidebar,
                NSVisualEffectBlendingMode::BehindWindow,
                panels.radius as f32,
            );
            effect.setFrame(blur::frame(rect, height));
            effect.setAutoresizingMask(blur::autoresizing(rect, height));
            container.addSubview_positioned_relativeTo(
                &effect,
                NSWindowOrderingMode::Below,
                Some(&window.view),
            );
            window.window.setOpaque(false);
            window
                .window
                .setBackgroundColor(Some(&NSColor::clearColor()));
            window.layer.setOpaque(false);
            window.behind = Some(effect);
        }
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
                    WindowCommand::Move(_) => window.view.begin_move(),
                    WindowCommand::ToggleMaximize => later(id, Native::Zoom),
                    WindowCommand::ToggleFullscreen => later(id, Native::FullScreen),
                    WindowCommand::Present(_) => later(id, Native::Present),
                    WindowCommand::MakeKey => later(id, Native::Front),
                    WindowCommand::SetTitle(title) => later(id, Native::Title(title.to_string())),
                    WindowCommand::Fit((width, height)) => later(id, Native::Size(width, height)),
                    WindowCommand::Bar(bar) => later(id, Native::Toolbar(bar)),
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
                if event.is::<crate::drag::StartDrag>() {
                    if let Ok(drag) = event.downcast::<crate::drag::StartDrag>()
                        && let Some(window) = self.window(owner.raw())
                    {
                        window.view.begin_drag(&drag.0);
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
    let context = CFFileDescriptorContext {
        version: 0,
        info: std::ptr::null_mut(),
        retain: None,
        release: None,
        copyDescription: None,
    };
    // SAFETY: the descriptor belongs to the loop kept in `LOOP` for the whole run, and `close_on_invalidate` is false.
    let descriptor = unsafe { CFFileDescriptor::new(None, fd, false, Some(dispatch), &context) }
        .ok_or(RunError::EventLoop("CFFileDescriptor::new failed"))?;
    descriptor.enable_call_backs(1);
    let source = CFFileDescriptor::new_run_loop_source(None, Some(&descriptor), 0)
        .ok_or(RunError::EventLoop("the run loop source was not created"))?;
    CFRunLoop::current()
        .ok_or(RunError::EventLoop("no current run loop"))?
        // SAFETY: reading a CoreFoundation constant that is set when the framework loads.
        .add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
    std::mem::forget((descriptor, source));
    Ok(())
}

pub(super) fn run_window(app: AppComponent, options: WindowOptions) -> Result<(), RunError> {
    let mtm = MainThreadMarker::new().ok_or(RunError::EventLoop("not on the main thread"))?;
    let application = NSApplication::sharedApplication(mtm);
    application.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    menu::install(mtm, &application, &options.app_name);
    delegate::install(mtm, &application);

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
    system::observe(&NSApplication::sharedApplication(mtm));
    serve_loop(event_loop)?;

    with_runtime(|runtime| runtime.create_window(app, options, None));
    ping.ping();
    application.activate();
    application.run();
    Ok(())
}
