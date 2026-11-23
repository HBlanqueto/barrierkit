use std::cell::{Cell, RefCell};

use freya_core::integration::{KeyboardEventName, MouseEventName, PlatformEvent};
use freya_core::prelude::CursorIcon;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::runtime::{AnyObject, Bool};
use objc2::{
    AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
};
use objc2_app_kit::{
    NSBackingStoreType, NSEvent, NSEventModifierFlags, NSPanel, NSTextInputClient, NSTrackingArea,
    NSTrackingAreaOptions, NSView, NSVisualEffectView, NSWindow, NSWindowDelegate,
    NSWindowStyleMask,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSNotFound, NSNotification, NSObject,
    NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString, NSUInteger,
};
use objc2_quartz_core::{CALayer, CAMetalLayer};
use torin::prelude::CursorPoint;

use super::{input, with_runtime};

pub(super) struct ViewIvars {
    id: u64,
    wheel_started: Cell<bool>,
    // The text an input method is composing, and the keys it took, whose release is not the app's.
    marked: RefCell<String>,
    routed: RefCell<Vec<u16>>,
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
            if self.for_input_method(event)
                && let Some(context) = self.inputContext()
                && context.handleEvent(event)
            {
                self.ivars().routed.borrow_mut().push(event.keyCode());
                return;
            }
            self.send(input::key(event, KeyboardEventName::KeyDown));
        }

        // A key that edits text goes to the focused field, not to the menu bar's equivalent of the same chord.
        #[unsafe(method(performKeyEquivalent:))]
        fn perform_key_equivalent(&self, event: &NSEvent) -> Bool {
            let chord = input::edit_chord(event);
            let edits = chord.is_some()
                && with_runtime(|runtime| runtime.text_field_focused(self.ivars().id))
                    .unwrap_or(false);
            if let Some(chord) = chord.filter(|_| edits) {
                self.send(chord);
            }
            Bool::new(edits)
        }

        #[unsafe(method(keyUp:))]
        fn key_up(&self, event: &NSEvent) {
            let mut routed = self.ivars().routed.borrow_mut();
            if let Some(index) = routed.iter().position(|code| *code == event.keyCode()) {
                routed.remove(index);
                return;
            }
            drop(routed);
            self.send(input::key(event, KeyboardEventName::KeyUp));
        }

        #[unsafe(method(flagsChanged:))]
        fn flags_changed(&self, event: &NSEvent) {
            super::super::input::store_modifier_flags(input::modifiers(event.modifierFlags()));
        }
    }

    unsafe impl NSTextInputClient for ContentView {
        #[unsafe(method(insertText:replacementRange:))]
        fn insert_text(&self, string: &AnyObject, _: NSRange) {
            self.ivars().marked.borrow_mut().clear();
            let text = text_of(string);
            with_runtime(|runtime| runtime.commit_text(self.ivars().id, text));
        }

        #[unsafe(method(setMarkedText:selectedRange:replacementRange:))]
        fn set_marked_text(&self, string: &AnyObject, _: NSRange, _: NSRange) {
            let text = text_of(string);
            self.ivars().marked.replace(text.clone());
            with_runtime(|runtime| runtime.set_preedit(self.ivars().id, text));
        }

        #[unsafe(method(unmarkText))]
        fn unmark_text(&self) {
            self.ivars().marked.borrow_mut().clear();
            with_runtime(|runtime| runtime.set_preedit(self.ivars().id, String::new()));
        }

        #[unsafe(method(selectedRange))]
        fn selected_range(&self) -> NSRange {
            NSRange::new(self.ivars().marked.borrow().encode_utf16().count(), 0)
        }

        #[unsafe(method(markedRange))]
        fn marked_range(&self) -> NSRange {
            match self.ivars().marked.borrow().encode_utf16().count() {
                0 => NSRange::new(NSNotFound as usize, 0),
                length => NSRange::new(0, length),
            }
        }

        #[unsafe(method(hasMarkedText))]
        fn has_marked_text(&self) -> bool {
            !self.ivars().marked.borrow().is_empty()
        }

        #[unsafe(method(attributedSubstringForProposedRange:actualRange:))]
        fn attributed_substring(
            &self,
            _: NSRange,
            _: *mut NSRange,
        ) -> *mut NSAttributedString {
            std::ptr::null_mut()
        }

        #[unsafe(method(validAttributesForMarkedText))]
        fn valid_attributes_for_marked_text(&self) -> *mut NSArray<NSAttributedStringKey> {
            Retained::autorelease_return(NSArray::new())
        }

        #[unsafe(method(firstRectForCharacterRange:actualRange:))]
        fn first_rect(&self, _: NSRange, _: *mut NSRange) -> NSRect {
            let (x, y, width, height) = with_runtime(|runtime| runtime.ime_area(self.ivars().id))
                .flatten()
                .unwrap_or((0, 0, 0, 0));
            let area = NSRect::new(
                NSPoint::new(f64::from(x), f64::from(y)),
                NSSize::new(f64::from(width.max(1)), f64::from(height.max(1))),
            );
            let in_window = self.convertRect_toView(area, None);
            self.window()
                .map_or(in_window, |window| window.convertRectToScreen(in_window))
        }

        #[unsafe(method(characterIndexForPoint:))]
        fn character_index_for_point(&self, _: NSPoint) -> NSUInteger {
            NSNotFound as NSUInteger
        }
    }
);

fn text_of(string: &AnyObject) -> String {
    if let Some(attributed) = string.downcast_ref::<NSAttributedString>() {
        attributed.string().to_string()
    } else if let Some(plain) = string.downcast_ref::<NSString>() {
        plain.to_string()
    } else {
        String::new()
    }
}

impl ContentView {
    pub(super) fn new(mtm: MainThreadMarker, id: u64, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ViewIvars {
            id,
            wheel_started: Cell::new(false),
            marked: RefCell::new(String::new()),
            routed: RefCell::new(Vec::new()),
        });
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        this.setWantsLayer(true);
        this
    }

    // Keys that make text go to the input method: while it composes, and for the characters of a focused text field.
    fn for_input_method(&self, event: &NSEvent) -> bool {
        if !self.ivars().marked.borrow().is_empty() {
            return true;
        }
        let chord = event
            .modifierFlags()
            .intersects(NSEventModifierFlags::Command | NSEventModifierFlags::Control);
        !chord
            && !input::is_named(event.keyCode())
            && with_runtime(|runtime| runtime.text_field_focused(self.ivars().id)).unwrap_or(false)
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
    // A native effect above the content lets every event through to it.
    #[unsafe(super(NSVisualEffectView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKEffectView"]
    pub(super) struct Effect;

    impl Effect {
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, _: NSPoint) -> *mut NSView {
            std::ptr::null_mut()
        }
    }
);

impl Effect {
    pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSVisualEffectView's designated initializer.
        unsafe { msg_send![super(this), init] }
    }
}

define_class!(
    // The chrome of a declared glass: a Metal layer above the effect, which never takes an event.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKChromeView"]
    pub(super) struct ChromeView;

    impl ChromeView {
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

        #[unsafe(method(hitTest:))]
        fn hit_test(&self, _: NSPoint) -> *mut NSView {
            std::ptr::null_mut()
        }
    }
);

impl ChromeView {
    pub(super) fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSView's designated initializer.
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        this.setWantsLayer(true);
        this
    }
}

define_class!(
    // A borderless window refuses the keyboard unless it says otherwise.
    #[unsafe(super(NSWindow))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKToplevel"]
    pub(super) struct Toplevel;

    impl Toplevel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            true
        }
    }
);

impl Toplevel {
    pub(super) fn new(
        mtm: MainThreadMarker,
        frame: NSRect,
        style: NSWindowStyleMask,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSWindow's designated initializer.
        unsafe {
            msg_send![
                super(this),
                initWithContentRect: frame,
                styleMask: style,
                backing: NSBackingStoreType::Buffered,
                defer: false
            ]
        }
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
