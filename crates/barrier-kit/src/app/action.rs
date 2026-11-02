use keyboard_types::{Key, Modifiers, NamedKey};

use crate::menu::{CheckState, MenuItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Return,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: Modifiers,
    key: ShortcutKey,
}

impl Shortcut {
    pub const fn new(modifiers: Modifiers, key: ShortcutKey) -> Self {
        Self { modifiers, key }
    }

    pub const fn ctrl(key: ShortcutKey) -> Self {
        Self::new(Modifiers::CONTROL, key)
    }

    pub fn matches(&self, modifiers: Modifiers, key: &Key) -> bool {
        let same_key = match (self.key, key) {
            (ShortcutKey::Char(c), Key::Character(s)) => {
                s.chars().eq(c.to_lowercase()) || s.chars().eq(c.to_uppercase())
            }
            (ShortcutKey::Up, Key::Named(NamedKey::ArrowUp))
            | (ShortcutKey::Down, Key::Named(NamedKey::ArrowDown))
            | (ShortcutKey::Left, Key::Named(NamedKey::ArrowLeft))
            | (ShortcutKey::Right, Key::Named(NamedKey::ArrowRight))
            | (ShortcutKey::Return, Key::Named(NamedKey::Enter))
            | (ShortcutKey::Escape, Key::Named(NamedKey::Escape)) => true,
            _ => false,
        };
        same_key && modifiers == self.modifiers
    }

    pub fn text(&self) -> String {
        let mut parts = Vec::new();
        for (flag, name) in [
            (Modifiers::CONTROL, "Ctrl"),
            (Modifiers::ALT, "Alt"),
            (Modifiers::SHIFT, "Shift"),
            (Modifiers::META, "Super"),
        ] {
            if self.modifiers.contains(flag) {
                parts.push(name.to_string());
            }
        }
        parts.push(match self.key {
            ShortcutKey::Char(c) => c.to_uppercase().to_string(),
            ShortcutKey::Up => "Up".into(),
            ShortcutKey::Down => "Down".into(),
            ShortcutKey::Left => "Left".into(),
            ShortcutKey::Right => "Right".into(),
            ShortcutKey::Return => "Return".into(),
            ShortcutKey::Escape => "Escape".into(),
        });
        parts.join("+")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action {
    pub id: &'static str,
    pub title: &'static str,
    pub shortcut: Option<Shortcut>,
}

impl Action {
    pub const fn new(id: &'static str, title: &'static str) -> Self {
        Self {
            id,
            title,
            shortcut: None,
        }
    }

    pub const fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn item(&self, enabled: bool) -> MenuItem {
        self.item_with(enabled, CheckState::None)
    }

    pub fn item_with(&self, enabled: bool, check: CheckState) -> MenuItem {
        MenuItem::Action {
            id: self.id.to_string(),
            label: self.title.to_string(),
            shortcut: self.shortcut.map(|s| s.text()),
            enabled,
            check,
            icon: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_menu::shortcut_matches;

    #[test]
    fn a_typed_shortcut_and_its_menu_text_match_the_same_keys() {
        let shortcuts = [
            Shortcut::ctrl(ShortcutKey::Char('n')),
            Shortcut::ctrl(ShortcutKey::Up),
            Shortcut::new(
                Modifiers::CONTROL | Modifiers::SHIFT,
                ShortcutKey::Char('z'),
            ),
        ];
        let keys = [
            Key::Character("n".into()),
            Key::Named(NamedKey::ArrowUp),
            Key::Character("z".into()),
        ];
        for (shortcut, key) in shortcuts.iter().zip(&keys) {
            assert!(shortcut.matches(shortcut.modifiers, key));
            assert!(shortcut_matches(&shortcut.text(), shortcut.modifiers, key));
        }
    }
}
