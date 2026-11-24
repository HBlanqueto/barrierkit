use freya_core::{
    integration::{KeyboardEventName, MouseEventName, PlatformEvent, WheelEventName, WheelSource},
    prelude::{Code, CursorIcon, Key, Modifiers, MouseButton, NamedKey, TouchPhase},
};
use objc2::rc::Retained;
use objc2_app_kit::{NSCursor, NSEvent, NSEventModifierFlags};
use torin::prelude::CursorPoint;

use crate::platform::app::AppInstance;

// The kit's shortcuts are written with Ctrl and Super, so Command is Ctrl and Control is Super here, as the menu draws them.
pub(super) fn modifiers(flags: NSEventModifierFlags) -> Modifiers {
    let mut out = Modifiers::empty();
    out.set(
        Modifiers::CONTROL,
        flags.contains(NSEventModifierFlags::Command),
    );
    out.set(
        Modifiers::META,
        flags.contains(NSEventModifierFlags::Control),
    );
    out.set(Modifiers::ALT, flags.contains(NSEventModifierFlags::Option));
    out.set(
        Modifiers::SHIFT,
        flags.contains(NSEventModifierFlags::Shift),
    );
    out
}

pub(super) fn mouse_button(number: isize) -> MouseButton {
    match number {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        other => MouseButton::Other(other as u16),
    }
}

pub(super) fn mouse(name: MouseEventName, event: &NSEvent, position: CursorPoint) -> PlatformEvent {
    PlatformEvent::Mouse {
        name,
        cursor: position,
        button: Some(mouse_button(event.buttonNumber())),
    }
}

pub(super) fn moved(position: CursorPoint) -> PlatformEvent {
    PlatformEvent::Mouse {
        name: MouseEventName::MouseMove,
        cursor: position,
        button: None,
    }
}

// A macOS delta is positive when the content moves down; Freya takes the Wayland sign, positive towards the end.
pub(super) fn wheel(event: &NSEvent, position: CursorPoint, started: &mut bool) -> PlatformEvent {
    let precise = event.hasPreciseScrollingDeltas();
    let (dx, dy) = (event.scrollingDeltaX(), event.scrollingDeltaY());
    let (source, phase) = if precise {
        let ended = event.phase().is_empty() && event.momentumPhase().is_empty()
            || event.phase().contains(objc2_app_kit::NSEventPhase::Ended)
                && event.momentumPhase().is_empty();
        let phase = if ended {
            *started = false;
            TouchPhase::Ended
        } else if std::mem::replace(started, true) {
            TouchPhase::Moved
        } else {
            TouchPhase::Started
        };
        (WheelSource::Pixel, phase)
    } else {
        (WheelSource::Line, TouchPhase::Moved)
    };
    let step = if precise { 1.0 } else { 10.0 };
    PlatformEvent::Wheel {
        name: WheelEventName::Wheel,
        scroll: CursorPoint::from((-dx * step, -dy * step)),
        cursor: position,
        source,
        phase,
    }
}

pub(super) fn is_named(code: u16) -> bool {
    named(code).is_some()
}

fn named(code: u16) -> Option<NamedKey> {
    Some(match code {
        36 | 76 => NamedKey::Enter,
        48 => NamedKey::Tab,
        51 => NamedKey::Backspace,
        53 => NamedKey::Escape,
        117 => NamedKey::Delete,
        115 => NamedKey::Home,
        119 => NamedKey::End,
        116 => NamedKey::PageUp,
        121 => NamedKey::PageDown,
        123 => NamedKey::ArrowLeft,
        124 => NamedKey::ArrowRight,
        125 => NamedKey::ArrowDown,
        126 => NamedKey::ArrowUp,
        122 => NamedKey::F1,
        120 => NamedKey::F2,
        99 => NamedKey::F3,
        118 => NamedKey::F4,
        96 => NamedKey::F5,
        97 => NamedKey::F6,
        98 => NamedKey::F7,
        100 => NamedKey::F8,
        101 => NamedKey::F9,
        109 => NamedKey::F10,
        103 => NamedKey::F11,
        111 => NamedKey::F12,
        _ => return None,
    })
}

fn code(key_code: u16) -> Code {
    match key_code {
        0 => Code::KeyA,
        11 => Code::KeyB,
        8 => Code::KeyC,
        2 => Code::KeyD,
        14 => Code::KeyE,
        3 => Code::KeyF,
        5 => Code::KeyG,
        4 => Code::KeyH,
        34 => Code::KeyI,
        38 => Code::KeyJ,
        40 => Code::KeyK,
        37 => Code::KeyL,
        46 => Code::KeyM,
        45 => Code::KeyN,
        31 => Code::KeyO,
        35 => Code::KeyP,
        12 => Code::KeyQ,
        15 => Code::KeyR,
        1 => Code::KeyS,
        17 => Code::KeyT,
        32 => Code::KeyU,
        9 => Code::KeyV,
        13 => Code::KeyW,
        7 => Code::KeyX,
        16 => Code::KeyY,
        6 => Code::KeyZ,
        36 | 76 => Code::Enter,
        48 => Code::Tab,
        49 => Code::Space,
        51 => Code::Backspace,
        53 => Code::Escape,
        123 => Code::ArrowLeft,
        124 => Code::ArrowRight,
        125 => Code::ArrowDown,
        126 => Code::ArrowUp,
        _ => Code::Unidentified,
    }
}

pub(super) fn key(event: &NSEvent, name: KeyboardEventName) -> PlatformEvent {
    let key_code = event.keyCode();
    let modifiers = modifiers(event.modifierFlags());
    let characters = if modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT) {
        event.charactersIgnoringModifiers()
    } else {
        event.characters()
    };
    let characters = characters
        .map(|text| text.to_string())
        .filter(|text| !text.is_empty() && !text.starts_with(char::is_control));
    let key = match (named(key_code), characters) {
        (Some(named), _) => Key::Named(named),
        (None, Some(text)) => Key::Character(text),
        (None, None) => Key::Named(NamedKey::Unidentified),
    };
    PlatformEvent::Keyboard {
        name,
        key,
        code: code(key_code),
        modifiers,
    }
}

fn modifier_key(key_code: u16, held: Modifiers) -> Option<(KeyboardEventName, NamedKey)> {
    let (key, modifier) = match key_code {
        54 | 55 => (NamedKey::Control, Modifiers::CONTROL),
        59 | 62 => (NamedKey::Meta, Modifiers::META),
        58 | 61 => (NamedKey::Alt, Modifiers::ALT),
        56 | 60 => (NamedKey::Shift, Modifiers::SHIFT),
        _ => return None,
    };
    let name = if held.contains(modifier) {
        KeyboardEventName::KeyDown
    } else {
        KeyboardEventName::KeyUp
    };
    Some((name, key))
}

pub(super) fn modifier_changed(event: &NSEvent) -> Option<PlatformEvent> {
    let key_code = event.keyCode();
    let modifiers = modifiers(event.modifierFlags());
    let (name, key) = modifier_key(key_code, modifiers)?;
    Some(PlatformEvent::Keyboard {
        name,
        key: Key::Named(key),
        code: code(key_code),
        modifiers,
    })
}

fn as_meta(modifiers: Modifiers) -> Modifiers {
    (modifiers - Modifiers::CONTROL) | Modifiers::META
}

// Freya's text editor reads Command as META on macOS, while the kit's shortcuts read it as Ctrl.
pub(super) fn edit_chord(event: &NSEvent) -> Option<PlatformEvent> {
    if !event
        .modifierFlags()
        .contains(NSEventModifierFlags::Command)
    {
        return None;
    }
    match key(event, KeyboardEventName::KeyDown) {
        PlatformEvent::Keyboard {
            name,
            key,
            code,
            modifiers,
        } if crate::main_menu::edits_text(modifiers, &key) => Some(PlatformEvent::Keyboard {
            name,
            key,
            code,
            modifiers: as_meta(modifiers),
        }),
        _ => None,
    }
}

pub(super) fn cursor(icon: CursorIcon) -> Retained<NSCursor> {
    match icon {
        CursorIcon::Text => NSCursor::IBeamCursor(),
        CursorIcon::VerticalText => NSCursor::IBeamCursorForVerticalLayout(),
        CursorIcon::Pointer => NSCursor::pointingHandCursor(),
        CursorIcon::Grab => NSCursor::openHandCursor(),
        CursorIcon::Grabbing => NSCursor::closedHandCursor(),
        CursorIcon::NotAllowed | CursorIcon::NoDrop => NSCursor::operationNotAllowedCursor(),
        CursorIcon::Crosshair | CursorIcon::Cell => NSCursor::crosshairCursor(),
        CursorIcon::ContextMenu => NSCursor::contextualMenuCursor(),
        CursorIcon::Copy => NSCursor::dragCopyCursor(),
        CursorIcon::Alias => NSCursor::dragLinkCursor(),
        CursorIcon::EResize
        | CursorIcon::WResize
        | CursorIcon::EwResize
        | CursorIcon::ColResize => NSCursor::columnResizeCursor(),
        CursorIcon::NResize
        | CursorIcon::SResize
        | CursorIcon::NsResize
        | CursorIcon::RowResize => NSCursor::rowResizeCursor(),
        _ => NSCursor::arrowCursor(),
    }
}

// A trackpad pinch or rotation: its phase and what changed since the last event.
pub(super) fn trackpad(
    event: &NSEvent,
    kind: fn(f32) -> crate::gesture::TrackpadKind,
) -> crate::gesture::Trackpad {
    use crate::gesture::{Trackpad, TrackpadPhase};
    use objc2_app_kit::NSEventPhase;
    let phase = event.phase();
    let phase = if phase.contains(NSEventPhase::Began) {
        TrackpadPhase::Began
    } else if phase.contains(NSEventPhase::Ended) || phase.contains(NSEventPhase::Cancelled) {
        TrackpadPhase::Ended
    } else {
        TrackpadPhase::Changed
    };
    Trackpad {
        phase,
        kind: kind(match event.r#type() {
            objc2_app_kit::NSEventType::Rotate => event.rotation(),
            _ => event.magnification() as f32,
        }),
    }
}

impl AppInstance {
    pub(crate) fn trackpad(&mut self, event: crate::gesture::Trackpad) {
        self.runner.run_in(|| crate::gesture::trackpad(event));
        self.waker.wake_by_ref();
    }
}

#[cfg(test)]
mod tests {
    use objc2_app_kit::NSEventType;
    use objc2_foundation::{NSPoint, NSString};

    use super::*;

    #[test]
    fn command_key_goes_down_as_control_and_up_when_released() {
        assert_eq!(
            modifier_key(55, Modifiers::CONTROL),
            Some((KeyboardEventName::KeyDown, NamedKey::Control))
        );
        assert_eq!(
            modifier_key(55, Modifiers::empty()),
            Some((KeyboardEventName::KeyUp, NamedKey::Control))
        );
        assert_eq!(
            modifier_key(59, Modifiers::META),
            Some((KeyboardEventName::KeyDown, NamedKey::Meta))
        );
        assert_eq!(modifier_key(57, Modifiers::empty()), None);
    }

    #[test]
    fn command_option_s_reads_s_not_the_composed_character() {
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            NSEventType::KeyDown,
            NSPoint::ZERO,
            NSEventModifierFlags::Command | NSEventModifierFlags::Option,
            0.0,
            0,
            None,
            &NSString::from_str("ß"),
            &NSString::from_str("s"),
            false,
            1,
        )
        .unwrap();
        assert!(matches!(
            key(&event, KeyboardEventName::KeyDown),
            PlatformEvent::Keyboard { key: Key::Character(text), .. } if text == "s"
        ));
    }
}
