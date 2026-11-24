use std::cell::RefCell;
use std::rc::Rc;

use freya_core::prelude::{Key, Modifiers, NamedKey, try_consume_root_context, use_drop, use_hook};

use crate::controlled::use_latest;
use crate::menu::{CheckState, MenuItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Char(char),
    Left,
    Right,
    Named(NamedKey),
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

    pub const DEFAULT_ACTION: Self =
        Self::new(Modifiers::empty(), ShortcutKey::Named(NamedKey::Enter));
    pub const CANCEL_ACTION: Self =
        Self::new(Modifiers::empty(), ShortcutKey::Named(NamedKey::Escape));

    pub const fn ctrl(key: ShortcutKey) -> Self {
        Self::new(Modifiers::CONTROL, key)
    }

    pub fn matches(&self, modifiers: Modifiers, key: &Key) -> bool {
        let same_key = match (self.key, key) {
            (ShortcutKey::Char(c), Key::Character(s)) => {
                s.chars().eq(c.to_lowercase()) || s.chars().eq(c.to_uppercase())
            }
            (ShortcutKey::Left, Key::Named(NamedKey::ArrowLeft))
            | (ShortcutKey::Right, Key::Named(NamedKey::ArrowRight)) => true,
            (ShortcutKey::Named(named), Key::Named(pressed)) => named == *pressed,
            _ => false,
        };
        let held =
            modifiers & (Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT | Modifiers::META);
        same_key && held == self.modifiers
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
            ShortcutKey::Left => "Left".into(),
            ShortcutKey::Right => "Right".into(),
            ShortcutKey::Named(named) => named.to_string(),
        });
        parts.join("+")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub id: &'static str,
    title: fn() -> &'static str,
    pub shortcut: Option<Shortcut>,
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.shortcut == other.shortcut
    }
}

impl Eq for Action {}

impl Action {
    pub const fn new(id: &'static str, title: fn() -> &'static str) -> Self {
        Self {
            id,
            title,
            shortcut: None,
        }
    }

    pub fn title(&self) -> &'static str {
        (self.title)()
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
            label: self.title().into(),
            shortcut: self.shortcut.map(|s| s.text().into()),
            enabled,
            check,
            icon: None,
            badge: None,
        }
    }
}

type MenuActionHandler = Rc<dyn Fn(&str)>;

#[derive(Clone, Default)]
pub(crate) struct MenuActions(Rc<RefCell<Vec<MenuActionHandler>>>);

impl MenuActions {
    pub(crate) fn dispatch(&self, id: &str) {
        let handlers = self.0.borrow().clone();
        for handler in handlers {
            handler(id);
        }
    }
}

pub fn use_menu_action(handler: impl Fn(&str) + 'static) {
    let actions = use_hook(try_consume_root_context::<MenuActions>);
    let registry = actions.clone();
    let latest = use_latest::<MenuActionHandler>(Rc::new(handler));
    let handler = use_hook(move || {
        let handler: MenuActionHandler = Rc::new(move |id| {
            let current = latest.borrow().clone();
            current(id)
        });
        if let Some(actions) = &registry {
            actions.0.borrow_mut().push(handler.clone());
        }
        handler
    });
    use_drop(move || {
        if let Some(actions) = actions {
            actions.0.borrow_mut().retain(|h| !Rc::ptr_eq(h, &handler));
        }
    });
}
