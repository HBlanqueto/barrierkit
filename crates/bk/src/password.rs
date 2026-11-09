use freya_core::prelude::{Key, Modifiers, NamedKey};
use zeroize::{Zeroize, Zeroizing};

use crate::platform::inserts_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Idle,
    Verifying,
    Error(String),
}

pub struct PasswordField {
    value: Zeroizing<String>,
    state: State,
}

impl Default for PasswordField {
    fn default() -> Self {
        Self {
            value: Zeroizing::new(String::new()),
            state: State::Idle,
        }
    }
}

impl PasswordField {
    pub fn insert(&mut self, ch: char) {
        if !ch.is_control() && self.editable() {
            if self.value.len() + ch.len_utf8() > self.value.capacity() {
                // String::push would reallocate and free the old buffer without zeroing it.
                let mut grown =
                    Zeroizing::new(String::with_capacity((self.value.capacity() * 2).max(32)));
                grown.push_str(&self.value);
                self.value = grown;
            }
            self.value.push(ch);
        }
    }

    pub fn backspace(&mut self) {
        if self.editable() {
            self.value.pop();
        }
    }

    fn editable(&mut self) -> bool {
        if matches!(self.state, State::Error(_)) {
            self.state = State::Idle;
        }
        self.state != State::Verifying
    }

    pub fn handle_key(&mut self, key: &Key, modifiers: Modifiers) -> Option<Zeroizing<String>> {
        match key {
            Key::Named(NamedKey::Enter) => self.take(),
            Key::Named(NamedKey::Backspace) => {
                self.backspace();
                None
            }
            Key::Character(text) if inserts_text(modifiers) => {
                text.chars().for_each(|ch| self.insert(ch));
                None
            }
            _ => None,
        }
    }

    pub fn take(&mut self) -> Option<Zeroizing<String>> {
        if self.state == State::Verifying || self.value.is_empty() {
            return None;
        }
        self.state = State::Verifying;
        Some(std::mem::replace(
            &mut self.value,
            Zeroizing::new(String::new()),
        ))
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.value.zeroize();
        self.state = State::Error(message.into());
    }

    pub fn reset(&mut self) {
        self.value.zeroize();
        self.state = State::Idle;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub(crate) fn revealed(&self) -> String {
        self.value.to_string()
    }

    pub fn masked(&self) -> String {
        "•".repeat(self.value.chars().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(field: &mut PasswordField, text: &str, modifiers: Modifiers) {
        assert!(
            field
                .handle_key(&Key::Character(text.into()), modifiers)
                .is_none()
        );
    }

    fn enter(field: &mut PasswordField) -> Option<Zeroizing<String>> {
        field.handle_key(&Key::Named(NamedKey::Enter), Modifiers::empty())
    }

    #[test]
    fn shift_letter_is_inserted_and_enter_submits_it() {
        let mut field = PasswordField::default();
        typed(&mut field, "H", Modifiers::SHIFT);
        assert_eq!(enter(&mut field).as_deref().map(String::as_str), Some("H"));
        assert_eq!(*field.state(), State::Verifying);
    }

    #[test]
    fn typing_past_the_capacity_keeps_every_character() {
        let mut field = PasswordField::default();
        let long = "contraseña-ñ".repeat(10);
        long.chars().for_each(|ch| field.insert(ch));
        assert_eq!(
            enter(&mut field).as_deref().map(String::as_str),
            Some(long.as_str())
        );
    }

    #[test]
    fn ctrl_letter_is_not_inserted() {
        let mut field = PasswordField::default();
        typed(&mut field, "a", Modifiers::CONTROL);
        assert!(enter(&mut field).is_none());
    }

    #[test]
    fn enter_on_an_empty_field_submits_nothing() {
        assert!(enter(&mut PasswordField::default()).is_none());
    }

    #[test]
    fn keys_are_ignored_while_verifying() {
        let mut field = PasswordField::default();
        typed(&mut field, "a", Modifiers::empty());
        assert!(enter(&mut field).is_some());
        typed(&mut field, "b", Modifiers::empty());
        field.handle_key(&Key::Named(NamedKey::Backspace), Modifiers::empty());
        assert!(enter(&mut field).is_none());
        assert_eq!(field.masked(), "");
    }

    #[test]
    fn backspace_removes_the_last_character() {
        let mut field = PasswordField::default();
        typed(&mut field, "ab", Modifiers::empty());
        field.handle_key(&Key::Named(NamedKey::Backspace), Modifiers::empty());
        assert_eq!(field.masked(), "•");
    }
}
