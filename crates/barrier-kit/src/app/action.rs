use std::cell::RefCell;
use std::rc::Rc;

use freya_core::prelude::{try_consume_root_context, use_drop, use_hook};
use keyboard_types::{Key, Modifiers, NamedKey};

use crate::menu::{CheckState, MenuItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutKey {
    Char(char),
    Left,
    Right,
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
            (ShortcutKey::Left, Key::Named(NamedKey::ArrowLeft))
            | (ShortcutKey::Right, Key::Named(NamedKey::ArrowRight)) => true,
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
            ShortcutKey::Left => "Left".into(),
            ShortcutKey::Right => "Right".into(),
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
    let handler = use_hook(move || {
        let handler: MenuActionHandler = Rc::new(handler);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_menu::shortcut_matches;

    #[test]
    fn a_menu_action_reaches_its_listener_until_the_listener_unmounts() {
        use freya_core::integration::{App, AppComponent};
        use freya_core::prelude::*;
        use freya_testing::TestingRunner;
        use torin::prelude::Size2D;

        struct Listener {
            heard: State<Vec<String>>,
        }
        impl Component for Listener {
            fn render(&self) -> impl IntoElement {
                let heard = self.heard;
                use_menu_action(move |id| {
                    let mut heard = heard;
                    heard.write().push(id.to_string());
                });
                rect()
            }
        }
        impl PartialEq for Listener {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }
        struct Host {
            heard: State<Vec<String>>,
            mounted: State<bool>,
        }
        impl App for Host {
            fn render(&self) -> impl IntoElement {
                rect().maybe_child(
                    (*self.mounted.read()).then(|| Listener { heard: self.heard }.into_element()),
                )
            }
        }

        let heard = State::create_global(Vec::new());
        let mut mounted = State::create_global(true);
        let app = AppComponent::new(Host { heard, mounted });
        let (mut test, actions) = TestingRunner::new(
            app,
            Size2D::new(10.0, 10.0),
            |runner| runner.provide_root_context(MenuActions::default),
            1.0,
        );
        test.sync_and_update();
        actions.dispatch("next");
        mounted.set(false);
        test.sync_and_update();
        actions.dispatch("previous");
        assert_eq!(*heard.peek(), vec!["next".to_string()]);
    }

    #[test]
    fn a_typed_shortcut_and_its_menu_text_match_the_same_keys() {
        let shortcuts = [
            Shortcut::ctrl(ShortcutKey::Char('n')),
            Shortcut::ctrl(ShortcutKey::Left),
            Shortcut::new(
                Modifiers::CONTROL | Modifiers::SHIFT,
                ShortcutKey::Char('z'),
            ),
        ];
        let keys = [
            Key::Character("n".into()),
            Key::Named(NamedKey::ArrowLeft),
            Key::Character("z".into()),
        ];
        for (shortcut, key) in shortcuts.iter().zip(&keys) {
            assert!(shortcut.matches(shortcut.modifiers, key));
            assert!(shortcut_matches(&shortcut.text(), shortcut.modifiers, key));
        }
    }
}
