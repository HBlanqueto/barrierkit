//! Back/forward navigation history and its toolbar buttons.

use freya_core::prelude::*;
use torin::prelude::{Direction, Size};

use crate::compact::IconButton;
use crate::symbol::Symbol;

#[derive(Clone, Debug, PartialEq)]
pub struct History<T> {
    entries: Vec<T>,
    position: usize,
}

impl<T: Clone + PartialEq> History<T> {
    pub fn new(root: T) -> Self {
        Self {
            entries: vec![root],
            position: 0,
        }
    }

    pub fn current(&self) -> &T {
        &self.entries[self.position]
    }

    pub fn push(&mut self, entry: T) {
        if *self.current() == entry {
            return;
        }
        self.entries.truncate(self.position + 1);
        self.entries.push(entry);
        self.position += 1;
    }

    pub fn can_go_back(&self) -> bool {
        self.position > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.position + 1 < self.entries.len()
    }

    pub fn back(&mut self) {
        if self.can_go_back() {
            self.position -= 1;
        }
    }

    pub fn forward(&mut self) {
        if self.can_go_forward() {
            self.position += 1;
        }
    }
}

pub fn navigation_buttons<T: Clone + PartialEq + 'static>(history: State<History<T>>) -> Rect {
    let mut back = history;
    let mut forward = history;
    rect()
        .direction(Direction::Horizontal)
        .height(Size::auto())
        .child(
            IconButton::new(Symbol::ChevronLeft, "Atrás")
                .disabled(!history.read().can_go_back())
                .on_press(move |_| back.write().back()),
        )
        .child(
            IconButton::new(Symbol::ChevronRight, "Adelante")
                .disabled(!history.read().can_go_forward())
                .on_press(move |_| forward.write().forward()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushing_after_going_back_drops_the_forward_entries() {
        let mut history = History::new("general");
        history.push("apariencia");
        history.push("wifi");
        history.back();
        history.push("bluetooth");
        assert_eq!(*history.current(), "bluetooth");
        assert!(!history.can_go_forward());
        history.back();
        assert_eq!(*history.current(), "apariencia");
    }

    #[test]
    fn pushing_the_current_entry_does_not_grow_the_history() {
        let mut history = History::new("general");
        history.push("general");
        assert!(!history.can_go_back());
    }
}
