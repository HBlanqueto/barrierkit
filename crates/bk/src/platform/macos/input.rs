use freya_core::{
    integration::{KeyboardEventName, MouseEventName, PlatformEvent, WheelEventName, WheelSource},
    prelude::{Code, Key, Modifiers, MouseButton, NamedKey, TouchPhase},
};
use objc2_app_kit::{NSEvent, NSEventModifierFlags};
use torin::prelude::CursorPoint;

// The kit's shortcuts are written with Ctrl, so Command is Ctrl here (docs/barrierkit/13-app.md).
pub(super) fn modifiers(flags: NSEventModifierFlags) -> Modifiers {
    let mut out = Modifiers::empty();
    out.set(
        Modifiers::CONTROL,
        flags.contains(NSEventModifierFlags::Command)
            || flags.contains(NSEventModifierFlags::Control),
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
    let characters = event
        .characters()
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
