use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::zvariant::{OwnedValue, Value};

use crate::menu::{CheckState, MenuItem};
use crate::platform::{MainMenuAddress, WakeHandle};

pub mod layout;
pub(crate) mod server;

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
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(title: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

#[derive(Clone)]
pub struct MenuHandle {
    state: Arc<Mutex<MenuState>>,
    wake: Option<WakeHandle>,
    path: Arc<Mutex<Option<String>>>,
    action_queue: Arc<Mutex<Vec<String>>>,
    address_callback: AddressCallbackCell,
}

type AddressCallback = Box<dyn Fn(MainMenuAddress) + Send>;
type AddressCallbackCell = Arc<Mutex<Option<AddressCallback>>>;

#[derive(Clone, Debug)]
struct MenuState {
    menu: MainMenu,
    revision: u32,
}

impl MenuHandle {
    pub(crate) fn new(menu: MainMenu, wake: Option<WakeHandle>) -> Self {
        Self {
            state: Arc::new(Mutex::new(MenuState { menu, revision: 1 })),
            wake,
            path: Arc::new(Mutex::new(None)),
            action_queue: Arc::new(Mutex::new(Vec::new())),
            address_callback: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn set_path(&self, path: impl Into<String>) {
        if let Ok(mut slot) = self.path.lock() {
            *slot = Some(path.into());
        }
    }

    pub(crate) fn path(&self) -> Option<String> {
        self.path.lock().ok().and_then(|g| g.clone())
    }

    pub(crate) fn push_action(&self, action_id: String) -> bool {
        let Some(wake) = &self.wake else {
            return false;
        };
        self.action_queue
            .lock()
            .expect("action queue poisoned")
            .push(action_id);
        wake.wake();
        true
    }

    pub(crate) fn drain_actions(&self) -> Vec<String> {
        let mut guard = self.action_queue.lock().expect("action queue poisoned");
        std::mem::take(&mut *guard)
    }

    #[cfg(test)]
    pub(crate) fn replace_menu(&self, menu: MainMenu) {
        let mut guard = self.state.lock().expect("menu state poisoned");
        guard.menu = menu;
        guard.revision = guard.revision.wrapping_add(1);
    }

    pub(crate) fn set_address_callback(&self, f: impl Fn(MainMenuAddress) + Send + Sync + 'static) {
        *self
            .address_callback
            .lock()
            .expect("address callback poisoned") = Some(Box::new(f));
    }

    pub(crate) fn deliver_address(&self, address: MainMenuAddress) {
        if let Ok(guard) = self.address_callback.lock()
            && let Some(cb) = guard.as_ref()
        {
            cb(address);
        }
    }

    pub fn update(&self, id: impl AsRef<str>, f: impl Fn(&mut MenuItem)) {
        let mut state = self.state.lock().expect("menu state poisoned");
        if update_item(&mut state.menu, id.as_ref(), &f) {
            state.revision = state.revision.wrapping_add(1);
            drop(state);
            self.publish();
        }
    }

    pub fn set_menu_items(&self, title: &str, items: Vec<MenuItem>) {
        let mut state = self.state.lock().expect("menu state poisoned");
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
        if let Ok(slot) = self.path.lock()
            && let Some(path) = slot.as_ref()
        {
            server::update(path);
        }
        if let Some(wake) = &self.wake {
            wake.wake();
        }
    }

    pub(crate) fn snapshot(&self) -> (MainMenu, u32) {
        let state = self.state.lock().expect("menu state poisoned");
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
        } = item
            && update_menu_items(children, id, f)
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, Default)]
pub(crate) struct IdMap {
    ids: HashMap<String, i32>,
    next: i32,
}

impl IdMap {
    pub(crate) fn get(&mut self, key: impl Into<String>) -> i32 {
        let key = key.into();
        if let Some(id) = self.ids.get(&key) {
            return *id;
        }
        self.next += 1;
        self.ids.insert(key, self.next);
        self.next
    }
}

pub(crate) fn dbusmenu_label(label: &str) -> String {
    label.replace('_', "__")
}

use freya_core::prelude::{
    Key, Modifiers, WritableUtils, try_consume_root_context, use_hook, use_state,
};

use crate::menu::shortcut_matches;

pub(crate) fn edits_text(modifiers: Modifiers, key: &Key) -> bool {
    let Key::Character(text) = key else {
        return false;
    };
    let command = modifiers.contains(Modifiers::CONTROL)
        && !modifiers.intersects(Modifiers::ALT | Modifiers::META);
    command
        && matches!(
            text.to_lowercase().as_str(),
            "a" | "c" | "v" | "x" | "y" | "z"
        )
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
        } => find_shortcut_in_items(children, modifiers, key),
        _ => None,
    })
}

fn owned<T: Into<Value<'static>>>(value: T) -> OwnedValue {
    OwnedValue::try_from(value.into()).expect("value is ownable")
}

pub(crate) fn check_state(check: &CheckState) -> (Option<&'static str>, i32) {
    match check {
        CheckState::Checkbox(active) => (Some("checkmark"), *active as i32),
        CheckState::Radio(active) => (Some("radio"), *active as i32),
        CheckState::None | CheckState::Minimized => (None, 0),
    }
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

pub fn use_menu_label(id: &'static str, label: &'static str) {
    use_menu_update(id, label, |item, label| {
        if let MenuItem::Action { label: shown, .. } = item {
            *shown = label.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use freya_core::prelude::NamedKey;

    #[test]
    fn shortcuts_export_as_keysyms_in_modifier_order() {
        use crate::menu::shortcut_keys;
        assert_eq!(shortcut_keys("Ctrl+Shift+S"), vec!["Control", "Shift", "S"]);
        assert_eq!(
            shortcut_keys("Super+Alt+Left"),
            vec!["Alt", "Super", "Left"]
        );
        assert_eq!(shortcut_keys("Ctrl+,"), vec!["Control", "comma"]);
        assert_eq!(shortcut_keys("Ctrl+Num+Delete"), vec!["Control", "Delete"]);
    }

    #[test]
    fn locks_and_named_keys_still_match() {
        let locks = Modifiers::CAPS_LOCK | Modifiers::NUM_LOCK;
        assert!(shortcut_matches(
            "Ctrl+S",
            Modifiers::CONTROL | locks,
            &Key::Character("s".into())
        ));
        assert!(shortcut_matches("F11", locks, &Key::Named(NamedKey::F11)));
        assert!(shortcut_matches(
            "Ctrl+space",
            Modifiers::CONTROL,
            &Key::Character(" ".into())
        ));
        assert!(shortcut_matches(
            "Ctrl+comma",
            Modifiers::CONTROL,
            &Key::Character(",".into())
        ));
        assert!(shortcut_matches(
            "Page_Down",
            Modifiers::empty(),
            &Key::Named(NamedKey::PageDown)
        ));
    }

    #[test]
    fn labels_escape_underscores() {
        assert_eq!(dbusmenu_label("a_b"), "a__b");
        assert_eq!(dbusmenu_label("Archivo"), "Archivo");
    }

    #[test]
    fn a_focused_text_field_keeps_select_all_copy_paste_cut_undo_and_redo() {
        let ctrl = Modifiers::CONTROL;
        for c in ["a", "c", "v", "x", "y", "z", "Z"] {
            assert!(edits_text(ctrl, &Key::Character(c.into())), "{c}");
        }
        assert!(edits_text(
            ctrl | Modifiers::SHIFT,
            &Key::Character("z".into())
        ));
        assert!(!edits_text(ctrl, &Key::Character("p".into())));
        assert!(!edits_text(Modifiers::empty(), &Key::Character("z".into())));
        assert!(!edits_text(
            ctrl | Modifiers::ALT,
            &Key::Character("z".into())
        ));
        assert!(!edits_text(ctrl, &Key::Named(NamedKey::ArrowLeft)));
    }

    #[test]
    fn ctrl_shift_s_matches() {
        assert!(shortcut_matches(
            "Ctrl+Shift+S",
            Modifiers::CONTROL | Modifiers::SHIFT,
            &Key::Character("s".into())
        ));
    }

    #[test]
    fn ctrl_shift_z_uppercase_still_matches() {
        assert!(shortcut_matches(
            "Ctrl+Shift+Z",
            Modifiers::CONTROL | Modifiers::SHIFT,
            &Key::Character("Z".into())
        ));
    }

    #[test]
    fn disabled_action_does_not_fire() {
        let menu = MainMenu::new(vec![Menu::new(
            "Archivo",
            vec![MenuItem::Action {
                id: "save".into(),
                label: "Guardar".into(),
                shortcut: Some("Ctrl+S".into()),
                enabled: false,
                check: CheckState::None,
                icon: None,
                badge: None,
            }],
        )]);
        assert!(
            find_menu_shortcut(
                &menu,
                Modifiers::CONTROL | Modifiers::SHIFT,
                &Key::Character("s".into())
            )
            .is_none()
        );
    }

    #[test]
    fn plain_typing_does_not_trigger_shortcut() {
        assert!(!shortcut_matches(
            "Ctrl+R",
            Modifiers::empty(),
            &Key::Character("r".into())
        ));
    }
}
