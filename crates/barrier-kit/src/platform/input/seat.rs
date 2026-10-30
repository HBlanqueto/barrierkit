use freya_core::{
    integration::{
        KeyboardEventName, MouseEventName, PlatformEvent, TouchEventName, WheelEventName,
        WheelSource,
    },
    prelude::CursorIcon,
};
use sctk::{
    delegate_keyboard, delegate_pointer, delegate_seat, delegate_touch,
    reexports::{
        calloop::{
            LoopHandle,
            timer::{TimeoutAction, Timer},
        },
        client::{
            Connection, Proxy, QueueHandle,
            backend::ObjectId,
            protocol::{wl_keyboard, wl_pointer, wl_seat, wl_surface::WlSurface, wl_touch},
        },
        protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape as CursorShape,
    },
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers as SctkModifiers, RepeatInfo},
        pointer::{PointerEvent, PointerHandler},
        touch::TouchHandler,
    },
};
use std::{rc::Rc, time::Duration};
use torin::prelude::CursorPoint;

use super::super::{runtime::Runtime, surface::physical_point};

pub(in crate::platform) struct RepeatState {
    pub(in crate::platform) event: KeyEvent,
    pub(in crate::platform) delay: Duration,
    pub(in crate::platform) rate: Duration,
    pub(in crate::platform) first: bool,
    pub(in crate::platform) pending: bool,
    pub(in crate::platform) generation: u64,
}

/// Cancels the current repeat and invalidates any timer already in flight.
pub(in crate::platform) fn stop_repeat(repeat: &mut Option<RepeatState>, generation: &mut u64) {
    *repeat = None;
    *generation = generation.wrapping_add(1);
}

/// Returns true if the pressed key should start the auto-repeat timer.
pub(in crate::platform) fn should_repeat_key(event: &KeyEvent) -> bool {
    !event.keysym.is_modifier_key()
}

/// Returns true if the released key is the one currently being repeated.
pub(in crate::platform) fn released_repeated_key(
    repeat: &Option<RepeatState>,
    event: &KeyEvent,
) -> bool {
    repeat
        .as_ref()
        .is_some_and(|state| state.event.keysym == event.keysym)
}

pub(in crate::platform) fn cursor_shape_from_icon(icon: CursorIcon) -> CursorShape {
    match icon {
        CursorIcon::Text => CursorShape::Text,
        CursorIcon::Pointer => CursorShape::Pointer,
        CursorIcon::Grab => CursorShape::Grab,
        CursorIcon::Grabbing => CursorShape::Grabbing,
        CursorIcon::NotAllowed => CursorShape::NotAllowed,
        CursorIcon::NResize => CursorShape::NResize,
        CursorIcon::EResize => CursorShape::EResize,
        CursorIcon::SResize => CursorShape::SResize,
        CursorIcon::WResize => CursorShape::WResize,
        CursorIcon::NeResize => CursorShape::NeResize,
        CursorIcon::NwResize => CursorShape::NwResize,
        CursorIcon::SeResize => CursorShape::SeResize,
        CursorIcon::SwResize => CursorShape::SwResize,
        CursorIcon::EwResize => CursorShape::EwResize,
        CursorIcon::NsResize => CursorShape::NsResize,
        CursorIcon::NeswResize => CursorShape::NeswResize,
        CursorIcon::NwseResize => CursorShape::NwseResize,
        CursorIcon::ColResize => CursorShape::ColResize,
        CursorIcon::RowResize => CursorShape::RowResize,
        CursorIcon::Move => CursorShape::Move,
        CursorIcon::Copy => CursorShape::Copy,
        CursorIcon::Alias => CursorShape::Alias,
        CursorIcon::NoDrop => CursorShape::NoDrop,
        CursorIcon::ContextMenu => CursorShape::ContextMenu,
        CursorIcon::Help => CursorShape::Help,
        CursorIcon::Progress => CursorShape::Progress,
        CursorIcon::Wait => CursorShape::Wait,
        CursorIcon::Cell => CursorShape::Cell,
        CursorIcon::Crosshair => CursorShape::Crosshair,
        CursorIcon::VerticalText => CursorShape::VerticalText,
        CursorIcon::AllScroll => CursorShape::AllScroll,
        CursorIcon::ZoomIn => CursorShape::ZoomIn,
        CursorIcon::ZoomOut => CursorShape::ZoomOut,
        CursorIcon::DndAsk => CursorShape::DndAsk,
        CursorIcon::AllResize => CursorShape::AllResize,
        CursorIcon::Default => CursorShape::Default,
        // cursor-icon is non-exhaustive; unknown icons fall back to the arrow.
        _ => CursorShape::Default,
    }
}

/// Find the index of the item whose identifier matches `id`.
/// Extracted so touch routing can be unit-tested without real Wayland surfaces.
pub(in crate::platform) fn find_index_by_id<T, Id: Eq>(
    items: &[T],
    id: Id,
    get_id: impl Fn(&T) -> Id,
) -> Option<usize> {
    items.iter().position(|item| get_id(item) == id)
}

impl SeatHandler for Runtime {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: wl_seat::WlSeat) {
        self.activation_seat = Some(seat.clone());
        let entries = self.toplevel_entries.clone();
        let seat = seat.clone();
        *self.toplevels.activate.borrow_mut() = Some(Rc::new(move |index| {
            if let Some(entry) = entries.borrow().get(index)
                && let Some(mgmt) = &entry.mgmt
            {
                mgmt.activate(&seat);
            }
        }));
    }

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer => {
                if let Ok(pointer) = self.seat.get_pointer::<Self>(qh, &seat) {
                    if let Some(manager) = &self.cursor_shape {
                        self.cursor_shape_device = Some(manager.get_shape_device(&pointer, qh));
                    }
                    self.pointer = Some(pointer);
                }
            }
            Capability::Keyboard => {
                if let Ok(keyboard) = self.seat.get_keyboard::<Self, Self>(qh, &seat, None) {
                    self.keyboard = Some(keyboard);
                }
            }
            Capability::Touch => {
                if let Ok(touch) = self.seat.get_touch::<Self>(qh, &seat) {
                    self.touch = Some(touch);
                }
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer => {
                self.pointer = None;
                self.cursor_shape_device = None;
            }
            Capability::Keyboard => self.keyboard = None,
            Capability::Touch => self.touch = None,
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for Runtime {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(index) = self.surface_for(&event.surface) else {
                continue;
            };
            if let sctk::seat::pointer::PointerEventKind::Enter { serial } = event.kind {
                self.last_pointer_serial.set(serial);
                if let Some(device) = &self.cursor_shape_device {
                    device.set_shape(serial, CursorShape::Default);
                }
            }
            let position = physical_point(event.position, self.surfaces[index].scale);
            use sctk::seat::pointer::PointerEventKind;
            let platform_event = match event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    PlatformEvent::Mouse {
                        name: MouseEventName::MouseMove,
                        cursor: position,
                        button: None,
                    }
                }
                PointerEventKind::Leave { .. } => PlatformEvent::Mouse {
                    name: MouseEventName::MouseMove,
                    cursor: CursorPoint::from((-1.0, -1.0)),
                    button: None,
                },
                PointerEventKind::Press { button, serial, .. } => {
                    self.last_input_serial.set(serial);
                    let surface_id = self.surfaces[index].surface.id();
                    if let Some(context) = self
                        .window_contexts
                        .iter()
                        .find(|ctx| ctx.controls.surface_id == surface_id)
                    {
                        context.controls.pointer_serial.set(serial);
                    }
                    PlatformEvent::Mouse {
                        name: MouseEventName::MouseDown,
                        cursor: position,
                        button: Some(super::mouse_button(button)),
                    }
                }
                PointerEventKind::Release { button, .. } => PlatformEvent::Mouse {
                    name: MouseEventName::MouseUp,
                    cursor: position,
                    button: Some(super::mouse_button(button)),
                },
                PointerEventKind::Axis {
                    vertical,
                    horizontal,
                    ..
                } => PlatformEvent::Wheel {
                    name: WheelEventName::Wheel,
                    scroll: CursorPoint::from((horizontal.absolute, vertical.absolute)),
                    cursor: position,
                    source: WheelSource::Device,
                },
            };
            self.surfaces[index].app.push(platform_event);
            self.surfaces[index].request_redraw(qh);
        }
    }
}

impl KeyboardHandler for Runtime {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        self.keyboard_focus = Some(surface.clone());
        if let Some(index) = self.surface_for(surface) {
            self.surfaces[index].app.set_focused(true);
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &WlSurface,
        _: u32,
    ) {
        if self.keyboard_focus.as_ref() == Some(surface) {
            self.keyboard_focus = None;
            if let Some(index) = self.surface_for(surface) {
                self.surfaces[index].app.set_focused(false);
            }
        }
        stop_repeat(&mut self.repeat, &mut self.repeat_generation);
    }

    fn press_key(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: KeyEvent,
    ) {
        self.last_input_serial.set(serial);
        self.key_event(qh, event.clone(), KeyboardEventName::KeyDown);
        // Modifier keys and keys that xkb marks non-repeating should not
        // produce repeated KeyDown events. SCTK 0.19.2 can repeat keys itself
        // via SeatState::get_keyboard_with_repeat, but that requires a loop
        // handle at keyboard creation time and a callback that knows the
        // focused surface; keeping our own timer is the smaller change.
        // shortcut: keys other than modifiers that xkb marks non-repeating
        // are still repeated because KeyboardHandler does not expose that bit.
        if should_repeat_key(&event) {
            let (delay, rate) = self.repeat_timing();
            self.repeat_generation = self.repeat_generation.wrapping_add(1);
            self.repeat = Some(RepeatState {
                event,
                delay,
                rate,
                first: true,
                pending: false,
                generation: self.repeat_generation,
            });
        }
        self.ping.ping();
    }

    fn release_key(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.key_event(qh, event.clone(), KeyboardEventName::KeyUp);
        // Only stop repeating if the released key is the one currently being
        // repeated. This keeps B repeating after A is released when A and B
        // were both pressed.
        if released_repeated_key(&self.repeat, &event) {
            stop_repeat(&mut self.repeat, &mut self.repeat_generation);
        }
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        modifiers: SctkModifiers,
        _: u32,
    ) {
        self.modifiers = modifiers;
    }

    fn update_repeat_info(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        info: RepeatInfo,
    ) {
        self.repeat_info = Some(info);
    }
}

impl Runtime {
    pub(super) fn key_event(
        &mut self,
        qh: &QueueHandle<Self>,
        event: KeyEvent,
        name: KeyboardEventName,
    ) {
        let (key, code) = super::key(event.keysym, event.utf8.clone());
        let modifiers = super::modifiers(self.modifiers);

        let index = self
            .keyboard_focus
            .as_ref()
            .and_then(|surface| self.surface_for(surface))
            .or_else(|| (!self.surfaces.is_empty()).then_some(0));

        let shortcut_action = if name == KeyboardEventName::KeyDown {
            index.and_then(|idx| {
                let surface = &self.surfaces[idx];
                let menu = surface
                    .menu_handle
                    .as_ref()
                    .map(|h| h.snapshot().0)
                    .or_else(|| surface.app.menu.clone())?;
                crate::app_menu::find_menu_shortcut(&menu, modifiers, &key)
                    .map(|action_id| (idx, action_id))
            })
        } else {
            None
        };

        if let Some((idx, ref action_id)) = shortcut_action
            && let Some(on_action) = self.surfaces[idx].app.on_menu_action.as_ref()
        {
            on_action(action_id);
        }

        let platform_event = PlatformEvent::Keyboard {
            name,
            key,
            code,
            modifiers,
        };
        if let Some(idx) = index {
            self.surfaces[idx].app.push(platform_event);
            self.surfaces[idx].request_redraw(qh);
            if shortcut_action.is_some() {
                self.poll_all();
                self.redraw_all();
            }
        }
    }

    fn repeat_timing(&self) -> (Duration, Duration) {
        match self.repeat_info {
            Some(RepeatInfo::Repeat { rate, delay }) => (
                Duration::from_millis(u64::from(delay)),
                Duration::from_micros(1_000_000 / u64::from(rate.get())),
            ),
            _ => (Duration::from_millis(200), Duration::from_millis(40)),
        }
    }

    pub(in crate::platform) fn install_repeat(&mut self, handle: &LoopHandle<Runtime>) {
        let Some(state) = &mut self.repeat else {
            return;
        };
        if state.pending || self.repeat_info.is_none() {
            return;
        }
        state.pending = true;
        let delay = if state.first { state.delay } else { state.rate };
        state.first = false;
        let generation = state.generation;
        let _ = handle.insert_source(
            Timer::from_duration(delay),
            move |_, _, shell: &mut Runtime| {
                if shell
                    .repeat
                    .as_ref()
                    .is_some_and(|state| state.generation == generation)
                {
                    let event = shell.repeat.as_ref().map(|state| state.event.clone());
                    if let Some(event) = event {
                        let qh = shell.qh.clone();
                        shell.key_event(&qh, event, KeyboardEventName::KeyDown);
                    }
                    if let Some(state) = &mut shell.repeat {
                        state.pending = false;
                    }
                    shell.ping.ping();
                }
                TimeoutAction::Drop
            },
        );
    }
}

/// Drain every active touch contact so `TouchHandler::cancel` can dispatch a
/// `TouchPhase::Cancelled` event to each surface. Kept as a pure helper so the
/// routing can be unit-tested without a live Wayland connection.
pub(in crate::platform) fn dispatch_touch_cancels(
    touch_surfaces: &mut std::collections::HashMap<i32, ObjectId>,
    mut dispatch: impl FnMut(i32, ObjectId),
) {
    for (id, surface_id) in touch_surfaces.drain() {
        dispatch(id, surface_id);
    }
}

impl TouchHandler for Runtime {
    fn down(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: u32,
        _: u32,
        surface: WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        let surface_id = surface.id();
        self.touch_surfaces.insert(id, surface_id.clone());
        self.touch_event(
            qh,
            surface_id,
            id,
            position,
            freya_core::prelude::TouchPhase::Started,
        );
    }

    fn up(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: u32,
        _: u32,
        id: i32,
    ) {
        let Some(surface_id) = self.touch_surfaces.remove(&id) else {
            return;
        };
        self.touch_event(
            qh,
            surface_id,
            id,
            (0.0, 0.0),
            freya_core::prelude::TouchPhase::Ended,
        );
    }

    fn motion(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: u32,
        id: i32,
        position: (f64, f64),
    ) {
        let Some(surface_id) = self.touch_surfaces.get(&id).cloned() else {
            return;
        };
        self.touch_event(
            qh,
            surface_id,
            id,
            position,
            freya_core::prelude::TouchPhase::Moved,
        );
    }

    fn shape(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: i32,
        _: f64,
        _: f64,
    ) {
    }

    fn orientation(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: i32,
        _: f64,
    ) {
    }

    fn cancel(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_touch::WlTouch) {
        // Move the active contacts out so the dispatch closure can borrow `self`
        // to push the event.
        let mut cancelled = std::mem::take(&mut self.touch_surfaces);
        dispatch_touch_cancels(&mut cancelled, |id, surface_id| {
            self.touch_event(
                qh,
                surface_id,
                id,
                (0.0, 0.0),
                freya_core::prelude::TouchPhase::Cancelled,
            );
        });
    }
}

impl Runtime {
    fn touch_event(
        &mut self,
        qh: &QueueHandle<Self>,
        surface_id: ObjectId,
        id: i32,
        position: (f64, f64),
        phase: freya_core::prelude::TouchPhase,
    ) {
        let Some(index) = find_index_by_id(&self.surfaces, surface_id, |s| s.surface.id()) else {
            return;
        };
        let platform_event = PlatformEvent::Touch {
            name: match phase {
                freya_core::prelude::TouchPhase::Started => TouchEventName::TouchStart,
                freya_core::prelude::TouchPhase::Moved => TouchEventName::TouchMove,
                freya_core::prelude::TouchPhase::Ended => TouchEventName::TouchEnd,
                freya_core::prelude::TouchPhase::Cancelled => TouchEventName::TouchCancel,
            },
            location: physical_point(position, self.surfaces[index].scale),
            finger_id: id as u64,
            phase,
            force: None,
        };
        self.surfaces[index].app.push(platform_event);
        self.surfaces[index].request_redraw(qh);
    }
}

delegate_seat!(Runtime);
delegate_pointer!(Runtime);
delegate_keyboard!(Runtime);
delegate_touch!(Runtime);
