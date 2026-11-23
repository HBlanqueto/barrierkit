use std::cell::Cell;
use std::rc::Rc;

use dispatch2::DispatchQueue;
use freya_core::integration::AppComponent;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSColor, NSPopUpMenuWindowLevel, NSScreen, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindow, NSWindowOrderingMode,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CAShapeLayer;

use super::arrow;
use super::place::{Rect, place};
use super::view::{ContentView, Panel, WindowDelegate};
use super::{Runtime, Window};
use crate::platform::ObjectId;
use crate::platform::app::{AppInstance, InstanceRole};
use crate::platform::popup::{PopupCommand, PopupId, PopupSpec};
use crate::platform::window_style::PopoverShape;

pub(super) struct PopupState {
    pub(super) id: PopupId,
    parent: Option<PopupId>,
    pub(super) owner: u64,
    on_dismissed: Option<Rc<dyn Fn()>>,
    opens_left: Rc<Cell<bool>>,
    grabbed: bool,
    arrowed: Option<(Retained<NSVisualEffectView>, PopoverShape)>,
}

// The material and the content take the shape of the body and the arrow that points at the anchor.
fn shape_effect(
    effect: &NSVisualEffectView,
    content: &NSView,
    shape: PopoverShape,
    anchor: (i32, i32, u32, u32),
    placed: Rect,
) {
    let shape_of = arrow::metrics(shape);
    let anchor = Rect {
        x: anchor.0,
        y: anchor.1,
        width: anchor.2 as i32,
        height: anchor.3 as i32,
    };
    let edge = arrow::facing_edge(anchor, placed);
    let tip = arrow::tip(anchor, placed, edge, &shape_of);
    let size = (f64::from(placed.width), f64::from(placed.height));
    let outline = arrow::outline(size, edge, tip, &shape_of);
    if let Some(layer) = effect.layer() {
        layer.setCornerRadius(0.0);
        layer.setMasksToBounds(false);
    }
    effect.setMaskImage(Some(&arrow::mask_image(
        arrow::path(&outline, Some(size.1)),
        NSSize::new(size.0, size.1),
    )));
    let mask = CAShapeLayer::new();
    mask.setFrame(content.bounds());
    mask.setPath(Some(&arrow::path(&outline, None)));
    if let Some(layer) = content.layer() {
        // SAFETY: a shape layer is a layer.
        unsafe { layer.setMask(Some(&mask)) };
    }
    if let Some(window) = effect.window() {
        window.invalidateShadow();
    }
}

struct Closing(Retained<NSWindow>);

// SAFETY: built on the main thread and consumed by a block that runs on the main queue.
unsafe impl Send for Closing {}

impl Closing {
    fn close(self) {
        self.0.close();
    }
}

// AppKit's origin is the bottom left of the primary screen; the kit's popups are placed from the top left.
fn primary_height(mtm: MainThreadMarker) -> f64 {
    NSScreen::screens(mtm)
        .firstObject()
        .map_or(0.0, |screen| screen.frame().size.height)
}

fn top_left(rect: NSRect, primary: f64) -> (f64, f64) {
    (rect.origin.x, primary - (rect.origin.y + rect.size.height))
}

struct Parent {
    window: Retained<NSWindow>,
    origin: (f64, f64),
    bounds: Rect,
    // How far AppKit has carried the title bar row down with the menu bar.
    bar_drop: f64,
}

impl Runtime {
    fn parent_of(&mut self, id: u64) -> Option<Parent> {
        let primary = primary_height(self.mtm);
        let parent = self.window(id)?;
        let (window, view) = (parent.window.clone(), parent.view.clone());
        let in_window = view.convertRect_toView(view.bounds(), None);
        let on_screen = window.convertRectToScreen(in_window);
        let origin = top_left(on_screen, primary);
        let bar_drop = parent
            .bar
            .as_ref()
            .filter(|bar| bar.revealed)
            .map_or(0.0, |bar| {
                on_screen.origin.y + on_screen.size.height - bar.view.top()
            });
        let visible = window
            .screen()
            .or_else(|| NSScreen::mainScreen(self.mtm))?
            .visibleFrame();
        let (x, y) = top_left(visible, primary);
        let bounds = Rect {
            x: (x - origin.0) as i32,
            y: (y - origin.1) as i32,
            width: visible.size.width as i32,
            height: visible.size.height as i32,
        };
        Some(Parent {
            window,
            origin,
            bounds,
            bar_drop,
        })
    }

    pub(super) fn process_popup_commands(&mut self) {
        let commands = std::mem::take(&mut *self.popup_commands.borrow_mut());
        for command in commands {
            match command {
                PopupCommand::Open {
                    owner,
                    parent,
                    id,
                    spec,
                    content,
                    popover,
                    ..
                } => self.open_popup(owner, parent, id, spec, content, popover),
                PopupCommand::Reposition { id, spec } => self.reposition_popup(id, &spec),
                PopupCommand::Close { id } => self.close_popup(id),
            }
        }
    }

    fn frame_of(parent: &Parent, spec: &PopupSpec, primary: f64) -> (NSRect, bool, Rect) {
        let placed = place(spec, parent.bounds);
        let (_, anchor_top, _, anchor_height) = spec.anchor_rect;
        let drop = if crate::titlebar::in_bar_row(anchor_top as f32, anchor_height as f32) {
            parent.bar_drop
        } else {
            0.0
        };
        let top = parent.origin.1 + drop + f64::from(placed.y);
        let frame = NSRect::new(
            NSPoint::new(
                parent.origin.0 + f64::from(placed.x),
                primary - (top + f64::from(placed.height)),
            ),
            NSSize::new(f64::from(placed.width), f64::from(placed.height)),
        );
        (frame, placed.x < 0, placed)
    }

    fn open_popup(
        &mut self,
        owner: ObjectId,
        parent: Option<PopupId>,
        id: PopupId,
        spec: PopupSpec,
        content: AppComponent,
        popover: Option<PopoverShape>,
    ) {
        let Some(parent_window) = self.parent_of(owner.raw()) else {
            return;
        };
        let primary = primary_height(self.mtm);
        let (frame, opens_left, placed) = Self::frame_of(&parent_window, &spec, primary);
        let window_id = self.next_window;
        self.next_window += 1;

        let panel = Panel::new(self.mtm, frame);
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setHasShadow(true);
        panel.setLevel(NSPopUpMenuWindowLevel);
        let (material, corner) = match popover {
            Some(PopoverShape::Popover) => (
                NSVisualEffectMaterial::Popover,
                ctdesign::CORNER_POPOVER.radius,
            ),
            Some(PopoverShape::Label) => (
                NSVisualEffectMaterial::ToolTip,
                ctdesign::CORNER_TOOLTIP.radius,
            ),
            Some(_) => (NSVisualEffectMaterial::Menu, ctdesign::CORNER_MENU.radius),
            None if spec.grab => (NSVisualEffectMaterial::Menu, ctdesign::CORNER_MENU.radius),
            None => (
                NSVisualEffectMaterial::ToolTip,
                ctdesign::CORNER_TOOLTIP.radius,
            ),
        };
        let bounds = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
        let effect = NSVisualEffectView::new(self.mtm);
        effect.setFrame(bounds);
        effect.setMaterial(material);
        effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect.setState(NSVisualEffectState::Active);
        effect.setWantsLayer(true);
        if let Some(layer) = effect.layer() {
            layer.setCornerRadius(f64::from(corner));
            layer.setMasksToBounds(true);
        }
        let view = ContentView::new(self.mtm, window_id, bounds);
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        let layer = self.configure_layer(&view);
        layer.setOpaque(false);
        effect.addSubview(&view);
        panel.setContentView(Some(&effect));
        self.follow_scheme(&panel);
        let delegate = WindowDelegate::object(self.mtm, window_id);
        panel.setDelegate(Some(&delegate));

        let scale = parent_window.window.backingScaleFactor();
        let physical = (
            (frame.size.width * scale) as u32,
            (frame.size.height * scale) as u32,
        );
        let opens_left_flag = Rc::new(Cell::new(opens_left));
        let app = AppInstance::new(
            &self.host(),
            ObjectId::from_raw(window_id),
            content,
            scale,
            physical,
            InstanceRole::Popup {
                id,
                opens_left: opens_left_flag.clone(),
            },
            |_| {},
        );
        let grabbed = spec.grab;
        let arrowed = popover.map(|shape| {
            shape_effect(&effect, &view, shape, spec.anchor_rect, placed);
            (effect.clone(), shape)
        });
        let native: Retained<NSWindow> = Retained::into_super(Retained::into_super(panel));
        self.windows.push(Window {
            id: window_id,
            window: native.clone(),
            link: super::frame_link(&native, &view),
            view,
            layer,
            _delegate: delegate,
            app,
            controls: None,
            container: None,
            behind: None,
            chromes: Vec::new(),
            bar: None,
            ime_area: None,
            popup: Some(PopupState {
                id,
                parent,
                owner: owner.raw(),
                on_dismissed: spec.on_dismissed,
                opens_left: opens_left_flag,
                grabbed,
                arrowed,
            }),
            main: false,
            needs_redraw: true,
        });
        // SAFETY: both are live windows the runner owns; the child is removed before it closes.
        unsafe {
            parent_window
                .window
                .addChildWindow_ordered(&native, NSWindowOrderingMode::Above)
        };
        if grabbed {
            native.makeKeyAndOrderFront(None);
        } else {
            native.orderFront(None);
        }
        self.resized(window_id);
    }

    fn reposition_popup(&mut self, id: PopupId, spec: &PopupSpec) {
        let Some((window_id, owner)) = self.windows.iter().find_map(|window| {
            let popup = window.popup.as_ref().filter(|popup| popup.id == id)?;
            Some((window.id, popup.owner))
        }) else {
            return;
        };
        let Some(parent) = self.parent_of(owner) else {
            return;
        };
        let primary = primary_height(self.mtm);
        let (frame, opens_left, placed) = Self::frame_of(&parent, spec, primary);
        if let Some(window) = self.window(window_id) {
            if let Some(popup) = &window.popup {
                popup.opens_left.set(opens_left);
            }
            window.window.setFrame_display(frame, true);
            if let Some((effect, shape)) = window
                .popup
                .as_ref()
                .and_then(|popup| popup.arrowed.as_ref())
            {
                shape_effect(effect, &window.view, *shape, spec.anchor_rect, placed);
            }
        }
    }

    pub(super) fn close_popup(&mut self, id: PopupId) {
        let children: Vec<PopupId> = self
            .windows
            .iter()
            .filter_map(|window| window.popup.as_ref())
            .filter(|popup| popup.parent == Some(id))
            .map(|popup| popup.id)
            .collect();
        for child in children {
            self.close_popup(child);
        }
        let Some(index) = self
            .windows
            .iter()
            .position(|window| window.popup.as_ref().is_some_and(|popup| popup.id == id))
        else {
            return;
        };
        let mut window = self.windows.remove(index);
        let popup = window.popup.take();
        if let Some(popup) = popup {
            if let Some(callback) = &popup.on_dismissed {
                callback();
            }
            let parent_popup = popup.parent.and_then(|parent| {
                self.windows.iter().find(|w| {
                    w.popup
                        .as_ref()
                        .is_some_and(|p| p.id == parent && p.grabbed)
                })
            });
            if popup.grabbed
                && let Some(parent_popup) = parent_popup
            {
                parent_popup.window.makeKeyWindow();
            }
        }
        if let Some(parent) = window.window.parentWindow() {
            parent.removeChildWindow(&window.window);
        }
        let closing = Closing(window.window.clone());
        DispatchQueue::main().exec_async(move || closing.close());
    }

    // A grabbed popup lives while the keyboard is in one of the popups; a click elsewhere takes it away.
    pub(super) fn dismiss_grab_without_keyboard(&mut self) {
        let key = NSApplication::sharedApplication(self.mtm).keyWindow();
        let in_popups = key.is_some_and(|key| {
            self.windows.iter().any(|window| {
                window.popup.is_some() && Retained::as_ptr(&window.window) == Retained::as_ptr(&key)
            })
        });
        if in_popups {
            return;
        }
        let roots: Vec<PopupId> = self
            .windows
            .iter()
            .filter_map(|window| window.popup.as_ref())
            .filter(|popup| popup.grabbed && popup.parent.is_none())
            .map(|popup| popup.id)
            .collect();
        for root in roots {
            self.close_popup(root);
        }
        self.redraw_all();
    }
}
