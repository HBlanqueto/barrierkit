//! Password field for the lock screen.

use zeroize::{Zeroize, Zeroizing};

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
        if self.state == State::Verifying || ch.is_control() {
            return;
        }
        if matches!(self.state, State::Error(_)) {
            self.state = State::Idle;
        }
        self.value.push(ch);
    }

    pub fn backspace(&mut self) {
        if self.state == State::Verifying {
            return;
        }
        if matches!(self.state, State::Error(_)) {
            self.state = State::Idle;
        }
        self.value.pop();
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
        self.value.clear();
        self.state = State::Error(message.into());
    }

    pub fn reset(&mut self) {
        self.value.zeroize();
        self.value.clear();
        self.state = State::Idle;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn masked(&self) -> String {
        "•".repeat(self.value.chars().count())
    }
}
