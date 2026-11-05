//! App-side global menu: declare a menu and export it over D-Bus.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::zvariant::{OwnedValue, Value};

use crate::menu::{CheckState, MenuItem};
use crate::platform::{AppMenuAddress, WakeHandle};

pub mod layout;
pub(crate) mod server;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppMenu {
    pub menus: Vec<Menu>,
}

impl AppMenu {
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

type AddressCallback = Box<dyn Fn(AppMenuAddress) + Send>;
type AddressCallbackCell = Arc<Mutex<Option<AddressCallback>>>;

impl std::fmt::Debug for MenuHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuHandle")
            .field("state", &self.state)
            .field("wake", &self.wake)
            .field("path", &self.path)
            .field("action_queue", &self.action_queue)
            .field("address_callback", &"...")
            .finish()
    }
}

#[derive(Clone, Debug)]
struct MenuState {
    menu: AppMenu,
    revision: u32,
}

impl MenuHandle {
    pub(crate) fn new(menu: AppMenu, wake: Option<WakeHandle>) -> Self {
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

    #[cfg(test)]
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
    pub(crate) fn replace_menu(&self, menu: AppMenu) {
        let mut guard = self.state.lock().expect("menu state poisoned");
        guard.menu = menu;
        guard.revision = guard.revision.wrapping_add(1);
    }

    pub(crate) fn set_address_callback(&self, f: impl Fn(AppMenuAddress) + Send + Sync + 'static) {
        *self
            .address_callback
            .lock()
            .expect("address callback poisoned") = Some(Box::new(f));
    }

    pub(crate) fn deliver_address(&self, address: AppMenuAddress) {
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
            if let Ok(slot) = self.path.lock()
                && let Some(path) = slot.as_ref()
            {
                server::update(path);
            }
            if let Some(wake) = &self.wake {
                wake.wake();
            }
        }
    }

    pub(crate) fn snapshot(&self) -> (AppMenu, u32) {
        let state = self.state.lock().expect("menu state poisoned");
        (state.menu.clone(), state.revision)
    }
}

fn update_item(menu: &mut AppMenu, id: &str, f: &dyn Fn(&mut MenuItem)) -> bool {
    for m in &mut menu.menus {
        if update_menu_items(&mut m.items, id, f) {
            return true;
        }
    }
    false
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

use freya_core::prelude::{Key, Modifiers};

pub use crate::menu::shortcut_matches;

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

pub fn find_menu_shortcut(menu: &AppMenu, modifiers: Modifiers, key: &Key) -> Option<String> {
    for m in &menu.menus {
        if let Some(id) = find_shortcut_in_items(&m.items, modifiers, key) {
            return Some(id);
        }
    }
    None
}

fn find_shortcut_in_items(items: &[MenuItem], modifiers: Modifiers, key: &Key) -> Option<String> {
    for item in items {
        match item {
            MenuItem::Action {
                id,
                shortcut: Some(shortcut),
                enabled: true,
                ..
            } => {
                if shortcut_matches(shortcut, modifiers, key) {
                    return Some(id.clone());
                }
            }
            MenuItem::Submenu {
                items: children, ..
            } => {
                if let Some(id) = find_shortcut_in_items(children, modifiers, key) {
                    return Some(id);
                }
            }
            _ => {}
        }
    }
    None
}

fn owned<T: Into<Value<'static>>>(value: T) -> OwnedValue {
    OwnedValue::try_from(value.into()).expect("value is ownable")
}

pub(crate) fn check_state(check: &CheckState) -> (Option<&'static str>, i32) {
    match check {
        CheckState::Checkbox(active) => (Some("checkmark"), *active as i32),
        CheckState::Radio(active) => (Some("radio"), *active as i32),
        CheckState::None => (None, 0),
    }
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
        let menu = AppMenu::new(vec![Menu::new(
            "Archivo",
            vec![MenuItem::Action {
                id: "save".into(),
                label: "Guardar".into(),
                shortcut: Some("Ctrl+S".into()),
                enabled: false,
                check: CheckState::None,
                icon: None,
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
