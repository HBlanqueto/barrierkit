use freya_core::prelude::{Key, Modifiers};

type Names = &'static [&'static str];

const MODIFIERS: [(Modifiers, &str, &str, Names); 4] = [
    (
        Modifiers::CONTROL,
        "Control",
        "⌃",
        &["ctrl", "control", "primary"],
    ),
    (
        Modifiers::ALT,
        "Alt",
        "⌥",
        &["alt", "option", "opt", "mod1"],
    ),
    (Modifiers::SHIFT, "Shift", "⇧", &["shift"]),
    (
        Modifiers::META,
        "Super",
        "❖",
        &["super", "meta", "cmd", "command"],
    ),
];

const KEYS: [(&str, &str, Names); 26] = [
    ("Escape", "⎋", &["esc"]),
    ("Return", "↩", &["enter"]),
    ("Tab", "⇥", &[]),
    ("space", "␣", &[" "]),
    ("BackSpace", "⌫", &[]),
    ("Delete", "⌦", &[]),
    ("Left", "←", &["arrowleft"]),
    ("Right", "→", &["arrowright"]),
    ("Up", "↑", &["arrowup"]),
    ("Down", "↓", &["arrowdown"]),
    ("Home", "↖", &[]),
    ("End", "↘", &[]),
    ("Page_Up", "⇞", &["pageup"]),
    ("Page_Down", "⇟", &["pagedown"]),
    ("comma", ",", &[]),
    ("period", ".", &[]),
    ("minus", "-", &[]),
    ("plus", "+", &[]),
    ("equal", "=", &[]),
    ("slash", "/", &[]),
    ("question", "?", &[]),
    ("backslash", "\\", &[]),
    ("semicolon", ";", &[]),
    ("apostrophe", "'", &[]),
    ("bracketleft", "[", &[]),
    ("bracketright", "]", &[]),
];

struct Shortcut {
    modifiers: Modifiers,
    key: Option<(String, String)>,
}

fn names(names: Names, token: &str) -> bool {
    names.iter().any(|name| name.eq_ignore_ascii_case(token))
}

fn key_token(token: &str) -> (String, String) {
    let named = KEYS.iter().find(|(keysym, glyph, aliases)| {
        keysym.eq_ignore_ascii_case(token) || *glyph == token || names(aliases, token)
    });
    match named {
        Some((keysym, glyph, _)) => (keysym.to_string(), glyph.to_string()),
        None if token.chars().count() == 1 => (token.to_string(), token.to_uppercase()),
        None => (token.to_string(), token.to_string()),
    }
}

fn parse_shortcut(text: &str) -> Shortcut {
    let mut shortcut = Shortcut {
        modifiers: Modifiers::empty(),
        key: None,
    };
    for token in text.split('+').map(str::trim).filter(|t| !t.is_empty()) {
        match MODIFIERS.iter().find(|(.., aliases)| names(aliases, token)) {
            Some((modifier, ..)) => shortcut.modifiers |= *modifier,
            None if token.eq_ignore_ascii_case("num") => {}
            None => shortcut.key = Some(key_token(token)),
        }
    }
    shortcut
}

fn held_modifiers(shortcut: &Shortcut) -> impl Iterator<Item = &(Modifiers, &str, &str, Names)> {
    MODIFIERS
        .iter()
        .filter(|(modifier, ..)| shortcut.modifiers.contains(*modifier))
}

pub fn format_shortcut(text: &str) -> String {
    let shortcut = parse_shortcut(text);
    held_modifiers(&shortcut)
        .map(|(_, _, glyph, _)| *glyph)
        .chain(shortcut.key.as_ref().map(|(_, glyph)| glyph.as_str()))
        .collect()
}

pub(crate) fn shortcut_keys(text: &str) -> Vec<String> {
    let shortcut = parse_shortcut(text);
    held_modifiers(&shortcut)
        .map(|(_, keysym, ..)| keysym.to_string())
        .chain(shortcut.key.clone().map(|(keysym, _)| keysym))
        .collect()
}

pub fn shortcut_matches(text: &str, modifiers: Modifiers, key: &Key) -> bool {
    let shortcut = parse_shortcut(text);
    let held =
        modifiers & (Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT | Modifiers::META);
    shortcut.modifiers == held
        && shortcut
            .key
            .is_some_and(|(_, glyph)| glyph == key_token(&key.to_string()).1)
}
