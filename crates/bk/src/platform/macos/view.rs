use std::cell::{Cell, RefCell};

use dispatch2::{DispatchQueue, MainThreadBound};
use freya_core::integration::{KeyboardEventName, MouseEventName, PlatformEvent};
use freya_core::prelude::CursorIcon;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::runtime::{AnyObject, Bool};
use objc2::{
    AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationPresentationOptions, NSBackingStoreType, NSDragOperation,
    NSDraggingContext, NSDraggingSession, NSDraggingSource, NSEvent, NSEventModifierFlags,
    NSGlassEffectContainerView, NSGlassEffectView, NSPanel, NSResponder, NSSplitView,
    NSSplitViewDividerStyle, NSTextInputClient, NSTrackingArea, NSTrackingAreaOptions, NSView,
    NSViewFrameDidChangeNotification, NSVisualEffectView, NSWindow, NSWindowButton,
    NSWindowDelegate, NSWindowDidMoveNotification, NSWindowStyleMask,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSNotFound, NSNotification,
    NSNotificationCenter, NSObject, NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString,
    NSUInteger,
};
use objc2_quartz_core::{CADisplayLink, CALayer, CAMetalLayer};
use torin::prelude::CursorPoint;

use super::{input, with_runtime};

pub(super) struct ViewIvars {
    id: u64,
    wheel_started: Cell<bool>,
    // The text an input method is composing, and the keys it took, whose release is not the app's.
    marked: RefCell<String>,
    routed: RefCell<Vec<u16>>,
    grab: Cell<Option<NSPoint>>,
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
            Retained::autorelease_return(Retained::into_super(CAMetalLayer::new()))
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
            // SAFETY: calls NSView's own implementation with the size it received.
            let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
            with_runtime(|runtime| runtime.resized(self.ivars().id));
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn view_did_change_backing_properties(&self) {
            // SAFETY: calls NSView's own implementation, which takes no arguments.
            let _: () = unsafe { msg_send![super(self), viewDidChangeBackingProperties] };
            with_runtime(|runtime| runtime.resized(self.ivars().id));
        }

        #[unsafe(method(drawFrame:))]
        fn draw_frame(&self, _: &CADisplayLink) {
            with_runtime(|runtime| runtime.frame_due(self.ivars().id));
        }

        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            // SAFETY: calls NSView's own implementation, which takes no arguments.
            let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
            for area in self.trackingAreas().iter() {
                self.removeTrackingArea(&area);
            }
            // AppKit's toolbar items light under the pointer in an inactive window too, as SwiftUI's.
            let options = NSTrackingAreaOptions::MouseMoved
                | NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::InVisibleRect;
            // SAFETY: a fresh NSTrackingArea, with valid options and this view as its owner.
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
            self.ivars().grab.set(None);
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
            if let Some(grab) = self.ivars().grab.get()
                && let Some(window) = self.window()
            {
                let at = NSEvent::mouseLocation();
                window.setFrameOrigin(NSPoint::new(at.x - grab.x, at.y - grab.y));
                return;
            }
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

        #[unsafe(method(magnifyWithEvent:))]
        fn magnify_with_event(&self, event: &NSEvent) {
            let id = self.ivars().id;
            let event = input::trackpad(event, crate::gesture::TrackpadKind::Magnify);
            with_runtime(|runtime| runtime.trackpad(id, event));
        }

        #[unsafe(method(rotateWithEvent:))]
        fn rotate_with_event(&self, event: &NSEvent) {
            let id = self.ivars().id;
            let event = input::trackpad(event, crate::gesture::TrackpadKind::Rotate);
            with_runtime(|runtime| runtime.trackpad(id, event));
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
            if let Some(key) = input::modifier_changed(event) {
                self.send(key);
            }
        }
    }

    // SAFETY: NSObjectProtocol has no methods to implement.
    unsafe impl NSObjectProtocol for ContentView {}

    // SAFETY: the method keeps the selector and signature of NSDraggingSource.
    unsafe impl NSDraggingSource for ContentView {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn source_operation_mask(&self, _: &NSDraggingSession, _: NSDraggingContext) -> NSDragOperation {
            NSDragOperation::Copy
        }
    }

    // SAFETY: each method keeps the selector and signature of NSTextInputClient.
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
    // AppKit starts a session only from a mouse event: the drag the pointer is making.
    pub(super) fn begin_drag(&self, payload: &crate::drag::DragPayload) {
        let Some(event) = NSApplication::sharedApplication(self.mtm()).currentEvent() else {
            return;
        };
        let at = self.convertPoint_fromView(event.locationInWindow(), None);
        let items = crate::platform::pasteboard::dragging_items(payload, at);
        self.beginDraggingSessionWithItems_event_source(
            &items,
            &event,
            ProtocolObject::from_ref(self),
        );
    }

    pub(super) fn new(mtm: MainThreadMarker, id: u64, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ViewIvars {
            id,
            wheel_started: Cell::new(false),
            marked: RefCell::new(String::new()),
            routed: RefCell::new(Vec::new()),
            grab: Cell::new(None),
        });
        // SAFETY: initWithFrame: is NSView's designated initializer, called once on the allocated object.
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

    // `performWindowDragWithEvent` does not begin on macOS 27.2 ("Window move completed without beginning"); AppKit moves a titled one.
    pub(super) fn begin_move(&self) {
        let left_down = NSEvent::pressedMouseButtons() & 1 != 0;
        let titled = |window: &NSWindow| window.styleMask().contains(NSWindowStyleMask::Titled);
        if let Some(window) = self.window().filter(|window| left_down && !titled(window)) {
            let (at, origin) = (NSEvent::mouseLocation(), window.frame().origin);
            self.ivars()
                .grab
                .set(Some(NSPoint::new(at.x - origin.x, at.y - origin.y)));
        }
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

    impl WindowDelegate {
        // AppKit moves its full-screen bar down with the menu bar; the title bar row rides in it while it is down.
        #[unsafe(method(barMoved:))]
        fn bar_moved(&self, note: &NSNotification) {
            if let Some(bar) = window_of(note) {
                self.follow_bar(&bar);
            }
        }
    }

    // SAFETY: WindowDelegate is an NSObject subclass.
    unsafe impl NSObjectProtocol for WindowDelegate {}

    // SAFETY: each method keeps the selector and signature of NSWindowDelegate.
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

        #[unsafe(method(windowDidEnterFullScreen:))]
        fn window_did_enter_full_screen(&self, note: &NSNotification) {
            let id = self.ivars().id;
            if let Some(window) = window_of(note) {
                set_titlebar_fill_alpha(&window, 0.0);
            }
            with_runtime(|runtime| runtime.fullscreen_changed(id));
            let bar = with_runtime(|runtime| runtime.bar_window(id)).flatten();
            if let Some(bar) = bar {
                self.follow_bar(&bar);
                // SAFETY: the delegate outlives the observation, which `windowDidExitFullScreen:` removes.
                unsafe {
                    NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
                        self,
                        sel!(barMoved:),
                        Some(NSWindowDidMoveNotification),
                        Some(&bar),
                    )
                };
            }
        }

        #[unsafe(method(windowWillEnterFullScreen:))]
        fn window_will_enter_full_screen(&self, note: &NSNotification) {
            let id = self.ivars().id;
            // AppKit shows and hides the fill itself, so the kit only makes it clear.
            if let Some(window) = window_of(note) {
                set_titlebar_fill_alpha(&window, 0.0);
                // The full-screen bar's opaque fills come one run-loop turn later and would hide the kit's row for the whole transition.
                let window = MainThreadBound::new(window, self.mtm());
                DispatchQueue::main().exec_async(move || {
                    window.get_on_main(|window| set_titlebar_fill_alpha(window, 0.0));
                });
            }
            if let Some((window, accessory)) = with_runtime(|runtime| runtime.attach_bar(id)).flatten() {
                window.addTitlebarAccessoryViewController(&accessory);
            }
            // An expanded bar rides below AppKit's title strip, which an empty `.expanded` toolbar would make 12 taller.
            if with_runtime(|runtime| runtime.rides_beside_lights(id)) == Some(false) {
                self.show_toolbar(false);
            }
        }

        // The toolbar and the title bar's fill are back before AppKit animates the window out, so nothing in the bar moves after it lands.
        #[unsafe(method(windowWillExitFullScreen:))]
        fn window_will_exit_full_screen(&self, note: &NSNotification) {
            if let Some(window) = window_of(note) {
                set_titlebar_fill_alpha(&window, 1.0);
            }
            self.show_toolbar(true);
        }

        #[unsafe(method(windowDidExitFullScreen:))]
        fn window_did_exit_full_screen(&self, _: &NSNotification) {
            // SAFETY: removes only this delegate's observation of bar moves.
            unsafe {
                NSNotificationCenter::defaultCenter().removeObserver_name_object(
                    self,
                    Some(NSWindowDidMoveNotification),
                    None,
                )
            };
            let id = self.ivars().id;
            if let Some(accessory) = with_runtime(|runtime| runtime.detach_bar(id)).flatten() {
                accessory.removeFromParentViewController();
            }
            with_runtime(|runtime| runtime.fullscreen_changed(id));
        }

        // A borderless window gets AppKit's title strip at the top edge unless the menu bar stays hidden.
        #[unsafe(method(window:willUseFullScreenPresentationOptions:))]
        fn full_screen_presentation(
            &self,
            window: &NSWindow,
            proposed: NSApplicationPresentationOptions,
        ) -> NSApplicationPresentationOptions {
            if window.styleMask().contains(NSWindowStyleMask::Titled) {
                // SwiftUI's `.windowToolbarFullScreenVisibility(.onHover)` is AppKit's `.autoHideToolbar`.
                return match with_runtime(|runtime| runtime.toolbar_on_hover(self.ivars().id)) {
                    Some(true) => proposed | NSApplicationPresentationOptions::AutoHideToolbar,
                    _ => proposed,
                };
            }
            NSApplicationPresentationOptions::FullScreen
                | NSApplicationPresentationOptions::HideMenuBar
                | NSApplicationPresentationOptions::HideDock
        }

        #[unsafe(method(windowShouldClose:))]
        fn window_should_close(&self, _: &NSWindow) -> bool {
            with_runtime(|runtime| runtime.should_close(self.ivars().id)).unwrap_or(true)
        }

        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _: &NSNotification) {
            with_runtime(|runtime| runtime.closed(self.ivars().id));
        }
    }
);

fn window_of(note: &NSNotification) -> Option<Retained<NSWindow>> {
    note.object()?.downcast::<NSWindow>().ok()
}

// AppKit fills a full-screen bar with an opaque view whose blur sees only the bar's own window; SwiftUI's bar is clear over the content.
// With a sidebar split view AppKit splits the fill in one view per column, and builds them only once the window is in full screen.
fn titlebar_fills(window: &NSWindow) -> Vec<Retained<NSView>> {
    let Some(button) = window.standardWindowButton(NSWindowButton::CloseButton) else {
        return Vec::new();
    };
    // SAFETY: both views are alive; neither is kept.
    let Some(container) = (unsafe { button.superview().and_then(|bar| bar.superview()) }) else {
        return Vec::new();
    };
    container
        .subviews()
        .into_iter()
        .filter(|view| view.class().name() == c"NSTitlebarBackgroundView")
        .collect()
}

fn set_titlebar_fill_alpha(window: &NSWindow, alpha: f64) {
    for fill in titlebar_fills(window) {
        fill.setAlphaValue(alpha);
    }
}

impl WindowDelegate {
    // How far the bar reaches into the screen sets the soft edge it leaves; it is revealed once it leaves the top.
    fn follow_bar(&self, bar: &NSWindow) {
        let Some(screen) = bar.screen().map(|screen| screen.frame()) else {
            return;
        };
        let (frame, top) = (bar.frame(), screen.origin.y + screen.size.height);
        let revealed = frame.origin.y + frame.size.height < top - 1.0;
        // At rest the clear bar lies over the kit's own row, which takes the pointer itself.
        bar.setIgnoresMouseEvents(!revealed);
        let depth = top - frame.origin.y;
        with_runtime(|runtime| runtime.reveal_bar(self.ivars().id, revealed, depth));
    }

    fn show_toolbar(&self, shown: bool) {
        let id = self.ivars().id;
        if let Some(toolbar) = with_runtime(|runtime| runtime.toolbar(id))
            .flatten()
            .filter(|toolbar| toolbar.isVisible() != shown)
        {
            toolbar.setVisible(shown);
        }
    }

    pub(super) fn object(
        mtm: MainThreadMarker,
        id: u64,
    ) -> Retained<ProtocolObject<dyn NSWindowDelegate>> {
        let this = Self::alloc(mtm).set_ivars(DelegateIvars { id });
        // SAFETY: init is NSObject's initializer, called once on the allocated object.
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
    // Liquid Glass takes the events over it, so that its interactive glass answers a press, and hands them on to the kit's view.
    #[unsafe(super(NSGlassEffectView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKGlassView"]
    pub(super) struct Glass;

    impl Glass {
        #[unsafe(method(nextResponder))]
        fn next_responder(&self) -> *mut NSResponder {
            match kit_view(self) {
                Some(view) => {
                    Retained::autorelease_return(Retained::into_super(Retained::into_super(view)))
                }
                // SAFETY: NSView's own responder, for a glass riding outside the kit's window.
                None => unsafe { msg_send![super(self), nextResponder] },
            }
        }
    }
);

fn kit_view(view: &NSView) -> Option<Retained<ContentView>> {
    let content = view.window()?.contentView()?;
    std::iter::once(content.clone())
        .chain(content.subviews())
        .find_map(|view| view.downcast::<ContentView>().ok())
}

impl Glass {
    pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSGlassEffectView's designated initializer.
        unsafe { msg_send![super(this), init] }
    }
}

define_class!(
    // Glass views inside it merge when they are near; only a glass takes an event, never the group around them.
    #[unsafe(super(NSGlassEffectContainerView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKGlassGroup"]
    pub(super) struct GlassGroup;

    impl GlassGroup {
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> *mut NSView {
            // SAFETY: NSView's own hit test of this group and its subviews.
            let hit: Option<Retained<NSView>> = unsafe { msg_send![super(self), hitTest: point] };
            let holder = self.contentView();
            let me: &NSView = self;
            hit.filter(|view| {
                !std::ptr::eq(&**view, me)
                    && holder.as_deref().is_none_or(|holder| !std::ptr::eq(&**view, holder))
            })
            .map_or(std::ptr::null_mut(), Retained::autorelease_return)
        }
    }
);

impl GlassGroup {
    pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSGlassEffectContainerView's designated initializer.
        unsafe { msg_send![super(this), init] }
    }
}

define_class!(
    // A pane's split view lies above the kit's view, which takes every event over it.
    #[unsafe(super(NSSplitView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKPaneSplit"]
    pub(super) struct PaneSplit;

    impl PaneSplit {
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, _: NSPoint) -> *mut NSView {
            std::ptr::null_mut()
        }
    }
);

impl PaneSplit {
    pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSView's designated initializer, inherited by NSSplitView.
        let split: Retained<Self> = unsafe { msg_send![super(this), init] };
        // As NSSplitViewController's own split view: NSSplitView's defaults stack the panes and resize the window.
        split.setVertical(true);
        split.setDividerStyle(NSSplitViewDividerStyle::Thin);
        split.setArrangesAllSubviews(false);
        split.setTranslatesAutoresizingMaskIntoConstraints(false);
        split
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
            Retained::autorelease_return(Retained::into_super(CAMetalLayer::new()))
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

pub(super) struct BarIvars {
    id: u64,
    container_height: Cell<f64>,
    row_top: f64,
    row_height: f64,
    below_strip: bool,
    windowed: bool,
}

define_class!(
    // The title bar row while the menu bar is revealed in full screen: it holds the row's chromes and sends its pointer events to the content.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "BKBarView"]
    #[ivars = BarIvars]
    pub(super) struct BarView;

    impl BarView {
        // Over a window's own title bar the row's glass takes its press, and every other event goes to the kit's view, as it does without the bar.
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> *mut NSView {
            // SAFETY: NSView's own hit test, on a live view.
            let hit: *mut NSView = unsafe { msg_send![super(self), hitTest: point] };
            if !self.ivars().windowed || hit.is_null() {
                return hit;
            }
            if !std::ptr::eq(hit, std::ptr::from_ref::<NSView>(self).cast_mut()) {
                return hit;
            }
            match kit_view(self) {
                Some(view) => Retained::autorelease_return(Retained::into_super(view)),
                None => hit,
            }
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.send(input::mouse(MouseEventName::MouseDown, event, self.position(event)));
        }

        // An opaque title bar keeps the content view's tracking area from seeing the pointer over the row, so the row tracks it itself.
        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            // SAFETY: calls NSView's own implementation, which takes no arguments.
            let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
            for area in self.trackingAreas().iter() {
                self.removeTrackingArea(&area);
            }
            if !self.ivars().windowed {
                return;
            }
            let options = NSTrackingAreaOptions::MouseMoved
                | NSTrackingAreaOptions::ActiveAlways
                | NSTrackingAreaOptions::InVisibleRect;
            // SAFETY: a fresh NSTrackingArea, with valid options and this view as its owner.
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

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.send(input::moved(self.position(event)));
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            self.send(input::mouse(MouseEventName::MouseUp, event, self.position(event)));
        }

        // AppKit slides the accessory container right of the lights in its own steps, apart from the bar window's moves.
        #[unsafe(method(viewDidMoveToSuperview))]
        fn view_did_move_to_superview(&self) {
            let center = NSNotificationCenter::defaultCenter();
            // SAFETY: only this view's own frame observations are removed and added; AppKit drops them when the view is freed.
            unsafe {
                center.removeObserver_name_object(self, Some(NSViewFrameDidChangeNotification), None);
                if let Some(superview) = self.superview() {
                    center.addObserver_selector_name_object(
                        self,
                        sel!(superviewMoved:),
                        Some(NSViewFrameDidChangeNotification),
                        Some(&superview),
                    );
                }
            }
        }

        #[unsafe(method(superviewMoved:))]
        fn superview_moved(&self, _: &NSNotification) {
            self.align(self.ivars().container_height.get());
        }

        // AppKit sizes the accessory after it is added, so its bounds follow its height.
        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, size: NSSize) {
            // SAFETY: NSView's own setter, on a live view.
            let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
            self.align(self.ivars().container_height.get());
        }
    }
);

impl BarView {
    pub(super) fn new(
        mtm: MainThreadMarker,
        id: u64,
        width: f64,
        (row_top, row_height): (f32, f32),
        below_strip: bool,
        windowed: bool,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(BarIvars {
            id,
            container_height: Cell::new(0.0),
            row_top: f64::from(row_top),
            row_height: f64::from(row_height),
            below_strip,
            windowed,
        });
        let frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(width, f64::from(row_height)),
        );
        // SAFETY: NSView's designated initializer.
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }

    // Its bounds are the window container's coordinates, so the chromes it holds keep their frames.
    pub(super) fn align(&self, container_height: f64) {
        self.ivars().container_height.set(container_height);
        let left = self.convertRect_toView(self.bounds(), None).origin.x;
        self.setBoundsOrigin(NSPoint::new(
            left,
            container_height - self.shown_top() - self.frame().size.height,
        ));
    }

    // Below AppKit's strip the accessory is 36 high, and SwiftUI centers the 40 px toolbar row in it.
    fn shown_top(&self) -> f64 {
        let ivars = self.ivars();
        let spare = ivars.row_height - self.frame().size.height;
        ivars.row_top
            + if ivars.below_strip {
                (spare / 2.0).max(0.0)
            } else {
                0.0
            }
    }

    // Where the container's top would be on screen if it rode with this row.
    pub(super) fn top(&self) -> f64 {
        let frame = self.convertRect_toView(self.bounds(), None);
        self.window().map_or(0.0, |window| {
            let frame = window.convertRectToScreen(frame);
            frame.origin.y + frame.size.height + self.shown_top()
        })
    }

    fn position(&self, event: &NSEvent) -> CursorPoint {
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let scale = self
            .window()
            .map_or(1.0, |window| window.backingScaleFactor());
        let y = self.ivars().container_height.get() - point.y;
        CursorPoint::from((point.x * scale, y * scale))
    }

    fn send(&self, event: PlatformEvent) {
        with_runtime(|runtime| runtime.push(self.ivars().id, event));
    }
}

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
        // SAFETY: the window is owned by a Retained, so AppKit must not release it when it closes.
        unsafe { this.setReleasedWhenClosed(false) };
        this
    }
}
