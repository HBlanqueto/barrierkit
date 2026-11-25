use std::collections::HashMap;

use zbus::zvariant::{OwnedValue, Value};

use crate::menu::CheckState;

pub mod layout;
pub(crate) mod server;

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

pub(super) fn owned<T: Into<Value<'static>>>(value: T) -> OwnedValue {
    OwnedValue::try_from(value.into()).expect("value is ownable")
}

pub(crate) fn check_state(check: &CheckState) -> (Option<&'static str>, i32) {
    match check {
        CheckState::Checkbox(active) => (Some("checkmark"), *active as i32),
        CheckState::Radio(active) => (Some("radio"), *active as i32),
        CheckState::Mixed => (Some("checkmark"), -1),
        CheckState::None | CheckState::Minimized => (None, 0),
    }
}
