use freya_core::prelude::{Key, Modifiers, NamedKey};
use std::borrow::Cow;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordState {
    Idle,
    Verifying,
    Error(Cow<'static, str>),
}

pub struct PasswordField {
    value: Zeroizing<String>,
    state: PasswordState,
}

impl Default for PasswordField {
    fn default() -> Self {
        Self {
            value: Zeroizing::new(String::new()),
            state: PasswordState::Idle,
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
        if matches!(self.state, PasswordState::Error(_)) {
            self.state = PasswordState::Idle;
        }
        self.state != PasswordState::Verifying
    }

    pub fn handle_key(&mut self, key: &Key, modifiers: Modifiers) -> Option<Zeroizing<String>> {
        match key {
            Key::Named(NamedKey::Enter) => self.take(),
            Key::Named(NamedKey::Backspace) => {
                self.backspace();
                None
            }
            Key::Character(text) if !modifiers.intersects(Modifiers::CONTROL | Modifiers::META) => {
                text.chars().for_each(|ch| self.insert(ch));
                None
            }
            _ => None,
        }
    }

    pub fn take(&mut self) -> Option<Zeroizing<String>> {
        if self.state == PasswordState::Verifying || self.value.is_empty() {
            return None;
        }
        self.state = PasswordState::Verifying;
        Some(std::mem::replace(
            &mut self.value,
            Zeroizing::new(String::new()),
        ))
    }

    pub fn set_error(&mut self, message: impl Into<Cow<'static, str>>) {
        self.value.zeroize();
        self.state = PasswordState::Error(message.into());
    }

    pub fn reset(&mut self) {
        self.value.zeroize();
        self.state = PasswordState::Idle;
    }

    pub fn state(&self) -> &PasswordState {
        &self.state
    }

    pub(crate) fn revealed(&self) -> String {
        self.value.to_string()
    }

    pub fn masked(&self) -> String {
        "•".repeat(self.value.chars().count())
    }
}
