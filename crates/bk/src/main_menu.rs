use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use freya_core::prelude::{
    Key, Modifiers, WritableUtils, try_consume_root_context, use_hook, use_state,
};

use crate::app::{Action, COPY, CUT, PASTE, REDO, SELECT_ALL, UNDO};
use crate::lock;
use crate::menu::{CheckState, MenuItem, shortcut_matches};
use crate::platform::WakeHandle;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MainMenu {
    pub menus: Vec<Menu>,
}

impl MainMenu {
    pub fn new(menus: Vec<Menu>) -> Self {
        Self { menus }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub title: Cow<'static, str>,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(title: impl Into<Cow<'static, str>>, items: Vec<MenuItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

#[derive(Clone)]
pub struct MenuHandle {
    pub(crate) state: Arc<Mutex<MenuState>>,
    wake: Option<WakeHandle>,
    path: Arc<Mutex<Option<String>>>,
    action_queue: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct MenuState {
    menu: MainMenu,
    pub(crate) revision: u32,
}

impl MenuHandle {
    pub(crate) fn new(menu: MainMenu, wake: Option<WakeHandle>) -> Self {
        Self {
            state: Arc::new(Mutex::new(MenuState { menu, revision: 1 })),
            wake,
            path: Arc::new(Mutex::new(None)),
            action_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn set_path(&self, path: impl Into<String>) {
        *lock(&self.path) = Some(path.into());
    }

    pub(crate) fn path(&self) -> Option<String> {
        lock(&self.path).clone()
    }

    pub(crate) fn push_action(&self, action_id: String) -> bool {
        let Some(wake) = &self.wake else {
            return false;
        };
        lock(&self.action_queue).push(action_id);
        wake.wake();
        true
    }

    pub(crate) fn drain_actions(&self) -> Vec<String> {
        let mut guard = lock(&self.action_queue);
        std::mem::take(&mut *guard)
    }

    pub fn update(&self, id: impl AsRef<str>, f: impl Fn(&mut MenuItem)) {
        let mut state = lock(&self.state);
        if update_item(&mut state.menu, id.as_ref(), &f) {
            state.revision = state.revision.wrapping_add(1);
            drop(state);
            self.publish();
        }
    }

    pub fn set_menu_items(&self, title: &str, items: Vec<MenuItem>) {
        let mut state = lock(&self.state);
        let Some(menu) = state.menu.menus.iter_mut().find(|menu| menu.title == title) else {
            return;
        };
        if menu.items == items {
            return;
        }
        menu.items = items;
        state.revision = state.revision.wrapping_add(1);
        drop(state);
        self.publish();
    }

    fn publish(&self) {
        if let Some(path) = lock(&self.path).as_ref() {
            crate::platform::menu_changed(path);
        }
        if let Some(wake) = &self.wake {
            wake.wake();
        }
    }

    pub(crate) fn snapshot(&self) -> (MainMenu, u32) {
        let state = lock(&self.state);
        (state.menu.clone(), state.revision)
    }
}

fn update_item(menu: &mut MainMenu, id: &str, f: &dyn Fn(&mut MenuItem)) -> bool {
    menu.menus
        .iter_mut()
        .any(|m| update_menu_items(&mut m.items, id, f))
}

fn update_menu_items(items: &mut [MenuItem], id: &str, f: &dyn Fn(&mut MenuItem)) -> bool {
    for item in items {
        let matched = match item {
            MenuItem::Action { id: item_id, .. } => item_id == id,
            MenuItem::Submenu { id: item_id, .. } => item_id.as_deref() == Some(id),
            _ => false,
        };
        if matched {
            f(item);
            return true;
        }
        if let MenuItem::Submenu {
            items: children, ..
        }
        | MenuItem::Palette { items: children } = item
            && update_menu_items(children, id, f)
        {
            return true;
        }
    }
    false
}

// A focused text field serves these Edit items before the app, as AppKit's responder chain does (docs/barrierkit/13-app.md#standard-menus).
const FIELD_EDITS: [(Action, &str); 6] = [
    (UNDO, "z"),
    (REDO, "y"),
    (CUT, "x"),
    (COPY, "c"),
    (PASTE, "v"),
    (SELECT_ALL, "a"),
];

pub(crate) fn field_edit_key(id: &str) -> Option<&'static str> {
    FIELD_EDITS
        .into_iter()
        .find_map(|(action, key)| (action.id == id).then_some(key))
}

pub(crate) fn edits_text(modifiers: Modifiers, key: &Key) -> bool {
    let Key::Character(text) = key else {
        return false;
    };
    let command = modifiers.contains(Modifiers::CONTROL)
        && !modifiers.intersects(Modifiers::ALT | Modifiers::META);
    command
        && FIELD_EDITS
            .iter()
            .any(|(_, letter)| text.eq_ignore_ascii_case(letter))
}

pub fn find_menu_shortcut(menu: &MainMenu, modifiers: Modifiers, key: &Key) -> Option<String> {
    menu.menus
        .iter()
        .find_map(|m| find_shortcut_in_items(&m.items, modifiers, key))
}

fn find_shortcut_in_items(items: &[MenuItem], modifiers: Modifiers, key: &Key) -> Option<String> {
    items.iter().find_map(|item| match item {
        MenuItem::Action {
            id,
            shortcut: Some(shortcut),
            enabled: true,
            ..
        } if shortcut_matches(shortcut, modifiers, key) => Some(id.clone()),
        MenuItem::Submenu {
            items: children, ..
        }
        | MenuItem::Palette { items: children } => find_shortcut_in_items(children, modifiers, key),
        _ => None,
    })
}

pub fn use_menu_update<T: PartialEq + Clone + 'static>(
    id: &'static str,
    value: T,
    apply: fn(&mut MenuItem, &T),
) {
    let handle = use_hook(try_consume_root_context::<MenuHandle>);
    let mut published = use_state(|| None::<T>);
    if published.peek().as_ref() == Some(&value) {
        return;
    }
    published.set(Some(value.clone()));
    if let Some(handle) = handle {
        handle.update(id, |item| apply(item, &value));
    }
}

pub fn use_menu_enabled(id: &'static str, enabled: bool) {
    use_menu_update(id, enabled, |item, enabled| {
        if let MenuItem::Action { enabled: shown, .. } | MenuItem::Submenu { enabled: shown, .. } =
            item
        {
            *shown = *enabled;
        }
    });
}

pub fn use_menu_label(id: &'static str, label: impl Into<Cow<'static, str>>) {
    use_menu_update(id, label.into(), |item, label| {
        if let MenuItem::Action { label: shown, .. } = item {
            *shown = label.clone();
        }
    });
}

pub fn use_menu_checked(id: &'static str, checked: bool) {
    use_menu_update(id, checked, |item, checked| {
        if let MenuItem::Action { check, .. } = item {
            *check = CheckState::Checkbox(*checked);
        }
    });
}

pub fn use_submenu_items(id: &'static str, items: Vec<MenuItem>) {
    use_menu_update(id, items, |item, items| {
        if let MenuItem::Submenu { items: shown, .. } = item {
            *shown = items.clone();
        }
    });
}
