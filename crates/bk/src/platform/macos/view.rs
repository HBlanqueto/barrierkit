use std::cell::Cell;

use freya_core::integration::{KeyboardEventName, MouseEventName, PlatformEvent};
use freya_core::prelude::CursorIcon;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{
    AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
};
use objc2_app_kit::{
    NSBackingStoreType, NSEvent, NSPanel, NSTrackingArea, NSTrackingAreaOptions, NSView,
    NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSRect, NSSize};
use objc2_quartz_core::{CALayer, CAMetalLayer};
use torin::prelude::CursorPoint;

use super::{input, with_runtime};

pub(super) struct ViewIvars {
    id: u64,
    wheel_started: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKContentView"]
    #[ivars = ViewIvars]
    pub(super) struct ContentView;

    impl ContentView {
        #[unsafe(method(makeBackingLayer))]
        fn make_backing_layer(&self) -> *mut CALayer {
            Retained::into_raw(Retained::into_super(CAMetalLayer::new()))
        }

        #[unsafe(method(wantsUpdateLayer))]
        fn wants_update_layer(&self) -> bool {
            true
        }

        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _: Option<&NSEvent>) -> bool {
            true
        }

        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, size: NSSize) {
            let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
            with_runtime(|runtime| runtime.resized(self.ivars().id));
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn view_did_change_backing_properties(&self) {
            let _: () = unsafe { msg_send![super(self), viewDidChangeBackingProperties] };
            with_runtime(|runtime| runtime.resized(self.ivars().id));
        }

        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
            for area in self.trackingAreas().iter() {
                self.removeTrackingArea(&area);
            }
            let options = NSTrackingAreaOptions::MouseMoved
                | NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveInKeyWindow
                | NSTrackingAreaOptions::InVisibleRect;
            let area = unsafe {
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    NSTrackingArea::alloc(),
                    NSRect::ZERO,
                    options,
                    Some(self),
                    None,
                )
            };
            self.addTrackingArea(&area);
        }

        #[unsafe(method(resetCursorRects))]
        fn reset_cursor_rects(&self) {
            let icon = with_runtime(|runtime| runtime.cursor()).unwrap_or(CursorIcon::Default);
            self.addCursorRect_cursor(self.bounds(), &input::cursor(icon));
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseDown);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseUp);
        }

        #[unsafe(method(rightMouseDown:))]
        fn right_mouse_down(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseDown);
        }

        #[unsafe(method(rightMouseUp:))]
        fn right_mouse_up(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseUp);
        }

        #[unsafe(method(otherMouseDown:))]
        fn other_mouse_down(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseDown);
        }

        #[unsafe(method(otherMouseUp:))]
        fn other_mouse_up(&self, event: &NSEvent) {
            self.mouse(event, MouseEventName::MouseUp);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.motion(event);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            self.motion(event);
        }

        #[unsafe(method(rightMouseDragged:))]
        fn right_mouse_dragged(&self, event: &NSEvent) {
            self.motion(event);
        }

        #[unsafe(method(otherMouseDragged:))]
        fn other_mouse_dragged(&self, event: &NSEvent) {
            self.motion(event);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _: &NSEvent) {
            self.send(input::moved(CursorPoint::from((-1.0, -1.0))));
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            let position = self.position(event);
            let mut started = self.ivars().wheel_started.get();
            let wheel = input::wheel(event, position, &mut started);
            self.ivars().wheel_started.set(started);
            self.send(wheel);
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            self.send(input::key(event, KeyboardEventName::KeyDown));
        }

        #[unsafe(method(keyUp:))]
        fn key_up(&self, event: &NSEvent) {
            self.send(input::key(event, KeyboardEventName::KeyUp));
        }
    }
);

impl ContentView {
    pub(super) fn new(mtm: MainThreadMarker, id: u64, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ViewIvars {
            id,
            wheel_started: Cell::new(false),
        });
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        this.setWantsLayer(true);
        this
    }

    fn scale(&self) -> f64 {
        self.window()
            .map_or(1.0, |window| window.backingScaleFactor())
    }

    fn position(&self, event: &NSEvent) -> CursorPoint {
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let scale = self.scale();
        CursorPoint::from((point.x * scale, point.y * scale))
    }

    fn send(&self, event: PlatformEvent) {
        with_runtime(|runtime| runtime.push(self.ivars().id, event));
    }

    fn mouse(&self, event: &NSEvent, name: MouseEventName) {
        self.send(input::mouse(name, event, self.position(event)));
    }

    fn motion(&self, event: &NSEvent) {
        self.send(input::moved(self.position(event)));
    }
}

pub(super) struct DelegateIvars {
    id: u64,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKWindowDelegate"]
    #[ivars = DelegateIvars]
    pub(super) struct WindowDelegate;

    unsafe impl NSObjectProtocol for WindowDelegate {}

    unsafe impl NSWindowDelegate for WindowDelegate {
        #[unsafe(method(windowDidBecomeKey:))]
        fn window_did_become_key(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.key_changed(self.ivars().id, true));
        }

        #[unsafe(method(windowDidResignKey:))]
        fn window_did_resign_key(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.key_changed(self.ivars().id, false));
        }

        #[unsafe(method(windowDidBecomeMain:))]
        fn window_did_become_main(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.main_changed(self.ivars().id));
        }

        #[unsafe(method(windowDidResignMain:))]
        fn window_did_resign_main(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.main_changed(self.ivars().id));
        }

        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.closed(self.ivars().id));
        }
    }
);

impl WindowDelegate {
    pub(super) fn new(
        mtm: MainThreadMarker,
        id: u64,
    ) -> Retained<ProtocolObject<dyn NSWindowDelegate>> {
        let this = Self::alloc(mtm).set_ivars(DelegateIvars { id });
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        ProtocolObject::from_retained(this)
    }
}

define_class!(
    // A popup that grabs takes the keyboard, so it must be able to become key; it never becomes main.
    #[unsafe(super(NSPanel))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKPanel"]
    pub(super) struct Panel;

    impl Panel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }
    }
);

impl Panel {
    pub(super) fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
        // SAFETY: NSWindow's designated initializer, with a borderless style.
        let this: Retained<Self> = unsafe {
            msg_send![
                super(this),
                initWithContentRect: frame,
                styleMask: style,
                backing: NSBackingStoreType::Buffered,
                defer: false
            ]
        };
        unsafe { this.setReleasedWhenClosed(false) };
        this
    }
}
