use freya_core::{
    integration::{
        KeyboardEventName, MouseEventName, PlatformEvent, TouchEventName, WheelEventName,
        WheelSource,
    },
    prelude::{CursorIcon, TouchPhase},
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
        keyboard::{
            KeyEvent, KeyboardHandler, Keysym, Modifiers as SctkModifiers, RawModifiers, RepeatInfo,
        },
        pointer::{PointerEvent, PointerHandler},
        touch::TouchHandler,
    },
};
use std::time::Duration;
use torin::prelude::CursorPoint;

use super::super::{WindowEdge, runtime::Runtime, surface::physical_point, window::WindowControls};

pub(crate) struct RepeatState {
    pub(in crate::platform) event: KeyEvent,
    pub(in crate::platform) delay: Duration,
    pub(in crate::platform) rate: Duration,
    pub(in crate::platform) first: bool,
    pub(in crate::platform) pending: bool,
    pub(in crate::platform) generation: u64,
}

pub(in crate::platform) fn stop_repeat(repeat: &mut Option<RepeatState>, generation: &mut u64) {
    *repeat = None;
    *generation = generation.wrapping_add(1);
}

pub(in crate::platform) fn should_repeat_key(event: &KeyEvent) -> bool {
    !event.keysym.is_modifier_key()
}

pub(in crate::platform) fn released_repeated_key(
    repeat: &Option<RepeatState>,
    event: &KeyEvent,
) -> bool {
    repeat
        .as_ref()
        .is_some_and(|state| state.event.raw_code == event.raw_code)
}

fn edge_shape(edge: Option<WindowEdge>) -> CursorShape {
    match edge {
        None => CursorShape::Default,
        Some(WindowEdge::Top) => CursorShape::NResize,
        Some(WindowEdge::Bottom) => CursorShape::SResize,
        Some(WindowEdge::Left) => CursorShape::WResize,
        Some(WindowEdge::Right) => CursorShape::EResize,
        Some(WindowEdge::TopLeft) => CursorShape::NwResize,
        Some(WindowEdge::TopRight) => CursorShape::NeResize,
        Some(WindowEdge::BottomLeft) => CursorShape::SwResize,
        Some(WindowEdge::BottomRight) => CursorShape::SeResize,
    }
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
        _ => CursorShape::Default,
    }
}

impl SeatHandler for Runtime {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: wl_seat::WlSeat) {
        self.seat_handle = Some(seat.clone());
        self.attach_text_input(&seat);
        self.attach_clipboard(&seat);
        if let Some(hooks) = self.extension_hooks() {
            (hooks.new_seat)(self, &seat);
        }
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
                if let Some(device) = self.cursor_shape_device.take() {
                    device.destroy();
                }
                if let Some(pointer) = self.pointer.take()
                    && pointer.version() >= 3
                {
                    pointer.release();
                }
            }
            Capability::Keyboard => {
                if let Some(keyboard) = self.keyboard.take()
                    && keyboard.version() >= 3
                {
                    keyboard.release();
                }
            }
            Capability::Touch => {
                if let Some(touch) = self.touch.take()
                    && touch.version() >= 3
                {
                    touch.release();
                }
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl Runtime {
    fn resize_from_edge(&mut self, index: usize, event: &PointerEvent) -> bool {
        use sctk::seat::pointer::{BTN_LEFT, PointerEventKind};
        let edge = if self.pointer_pressed {
            None
        } else {
            self.resize_edge(index, event.position)
        };
        match event.kind {
            PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                if edge != self.resize_hover {
                    self.resize_hover = edge;
                    if let Some(device) = &self.cursor_shape_device {
                        device.set_shape(self.last_pointer_serial.get(), edge_shape(edge));
                    }
                }
                edge.is_some()
            }
            PointerEventKind::Leave { .. } => {
                self.resize_hover = None;
                false
            }
            PointerEventKind::Press {
                button: BTN_LEFT,
                serial,
                ..
            } if let Some(edge) = edge => {
                if let Some(controls) = self.record_press_serial(index, serial) {
                    controls.resize(edge);
                }
                true
            }
            PointerEventKind::Press { .. } => {
                self.pointer_pressed = true;
                false
            }
            PointerEventKind::Release { .. } => {
                self.pointer_pressed = false;
                false
            }
            _ => false,
        }
    }
}

impl PointerHandler for Runtime {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
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
            use sctk::seat::pointer::PointerEventKind;
            if self.resize_from_edge(index, event) {
                continue;
            }
            let position = physical_point(event.position, self.surfaces[index].scale);
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
                    self.record_press_serial(index, serial);
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
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: KeyEvent,
    ) {
        self.last_input_serial.set(serial);
        self.last_input_surface = self.keyboard_focus.clone();
        self.key_event(event.clone(), KeyboardEventName::KeyDown);
        if should_repeat_key(&event)
            && let Some((delay, rate)) = self.repeat_timing()
        {
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

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: KeyEvent,
    ) {
        self.last_input_serial.set(serial);
        self.last_input_surface = self.keyboard_focus.clone();
        self.key_event(event, KeyboardEventName::KeyDown);
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.key_event(event.clone(), KeyboardEventName::KeyUp);
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
        _: RawModifiers,
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
    pub(super) fn key_event(&mut self, event: KeyEvent, name: KeyboardEventName) {
        let key = super::map_key(event.keysym, event.utf8.as_deref());
        let code = super::code(event.keysym);
        let modifiers = super::modifiers(self.modifiers);

        let index = self
            .keyboard_focus
            .as_ref()
            .and_then(|surface| self.surface_for(surface))
            .or_else(|| (!self.surfaces.is_empty()).then_some(0));

        let shortcut_action = if name == KeyboardEventName::KeyDown {
            index.and_then(|idx| {
                let surface = &self.surfaces[idx];
                if surface.app.text_field_focused() && crate::app_menu::edits_text(modifiers, &key)
                {
                    return None;
                }
                let menu = surface.menu_handle.as_ref()?.snapshot().0;
                crate::app_menu::find_menu_shortcut(&menu, modifiers, &key)
                    .map(|action_id| (idx, action_id))
            })
        } else {
            None
        };

        if let Some((idx, ref action_id)) = shortcut_action {
            self.surfaces[idx].app.menu_actions.dispatch(action_id);
        }

        let platform_event = PlatformEvent::Keyboard {
            name,
            key,
            code,
            modifiers,
        };
        if let Some(idx) = index {
            if shortcut_action.is_some() {
                self.poll_all();
                self.redraw_all();
            } else {
                self.surfaces[idx].app.push(platform_event);
            }
        }
    }

    fn repeat_timing(&self) -> Option<(Duration, Duration)> {
        match self.repeat_info? {
            RepeatInfo::Repeat { rate, delay } => Some((
                Duration::from_millis(u64::from(delay)),
                Duration::from_micros(1_000_000 / u64::from(rate.get())),
            )),
            RepeatInfo::Disable => None,
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
                        shell.key_event(event, KeyboardEventName::KeyDown);
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

impl TouchHandler for Runtime {
    fn down(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        serial: u32,
        _: u32,
        surface: WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        let surface_id = surface.id();
        if let Some(index) = self.surface_index_by_id(&surface_id) {
            self.record_press_serial(index, serial);
        }
        self.touch_surfaces
            .insert(id, (surface_id.clone(), position));
        self.touch_event(surface_id, id, position, TouchPhase::Started);
    }

    fn up(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: u32,
        _: u32,
        id: i32,
    ) {
        let Some((surface_id, position)) = self.touch_surfaces.remove(&id) else {
            return;
        };
        self.touch_event(surface_id, id, position, TouchPhase::Ended);
    }

    fn motion(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_touch::WlTouch,
        _: u32,
        id: i32,
        position: (f64, f64),
    ) {
        let Some((surface_id, last_position)) = self.touch_surfaces.get_mut(&id) else {
            return;
        };
        *last_position = position;
        let surface_id = surface_id.clone();
        self.touch_event(surface_id, id, position, TouchPhase::Moved);
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

    fn cancel(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_touch::WlTouch) {
        for (id, (surface_id, position)) in std::mem::take(&mut self.touch_surfaces) {
            self.touch_event(surface_id, id, position, TouchPhase::Cancelled);
        }
    }
}

impl Runtime {
    fn record_press_serial(&mut self, index: usize, serial: u32) -> Option<&WindowControls> {
        self.last_input_serial.set(serial);
        self.last_input_surface = Some(self.surfaces[index].surface.clone());
        let controls = &self.surfaces[index].context.as_ref()?.controls;
        controls.pointer_serial.set(serial);
        Some(controls)
    }

    fn touch_event(
        &mut self,
        surface_id: ObjectId,
        id: i32,
        position: (f64, f64),
        phase: TouchPhase,
    ) {
        let Some(index) = self.surface_index_by_id(&surface_id) else {
            return;
        };
        let platform_event = PlatformEvent::Touch {
            name: match phase {
                TouchPhase::Started => TouchEventName::TouchStart,
                TouchPhase::Moved => TouchEventName::TouchMove,
                TouchPhase::Ended => TouchEventName::TouchEnd,
                TouchPhase::Cancelled => TouchEventName::TouchCancel,
            },
            location: physical_point(position, self.surfaces[index].scale),
            finger_id: id as u64,
            phase,
            force: None,
        };
        self.surfaces[index].app.push(platform_event);
    }
}

delegate_seat!(Runtime);
delegate_pointer!(Runtime);
delegate_keyboard!(Runtime);
delegate_touch!(Runtime);
