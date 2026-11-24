use freya_core::prelude::*;

use crate::app::{Action, Shortcut, ShortcutKey, use_menu_action};
use crate::i18n::text;
use crate::main_menu::use_menu_enabled;

pub const UNDO: Action =
    Action::new("undo", || text!("undo")).shortcut(Shortcut::ctrl(ShortcutKey::Char('z')));
pub const REDO: Action = Action::new("redo", || text!("redo")).shortcut(Shortcut::new(
    Modifiers::CONTROL.union(Modifiers::SHIFT),
    ShortcutKey::Char('z'),
));

struct UndoStack<T> {
    undo: Vec<T>,
    redo: Vec<T>,
}

impl<T> Default for UndoStack<T> {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }
}

impl<T> UndoStack<T> {
    fn record(&mut self, previous: T) {
        self.undo.push(previous);
        self.redo.clear();
    }

    fn undo(&mut self, current: T) -> Option<T> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        Some(previous)
    }

    fn redo(&mut self, current: T) -> Option<T> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }
}

pub struct UndoManager<T> {
    value: State<T>,
    stack: State<UndoStack<T>>,
}

impl<T> Clone for UndoManager<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UndoManager<T> {}

impl<T: Clone + 'static> UndoManager<T> {
    pub fn set(mut self, next: T) {
        let previous = self.value.read().clone();
        self.stack.write().record(previous);
        self.value.set(next);
    }

    pub fn undo(mut self) {
        let current = self.value.read().clone();
        let previous = self.stack.write().undo(current);
        if let Some(previous) = previous {
            self.value.set(previous);
        }
    }

    pub fn redo(mut self) {
        let current = self.value.read().clone();
        let next = self.stack.write().redo(current);
        if let Some(next) = next {
            self.value.set(next);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.stack.read().undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.stack.read().redo.is_empty()
    }
}

pub fn use_undo<T: Clone + 'static>(value: State<T>) -> UndoManager<T> {
    let manager = UndoManager {
        value,
        stack: use_state(UndoStack::default),
    };
    use_menu_action(move |id| {
        if id == UNDO.id {
            manager.undo();
        } else if id == REDO.id {
            manager.redo();
        }
    });
    use_menu_enabled(UNDO.id, manager.can_undo());
    use_menu_enabled(REDO.id, manager.can_redo());
    manager
}
