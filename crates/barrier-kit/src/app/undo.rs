#[derive(Clone, Debug, PartialEq)]
pub struct UndoStack<T> {
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
    pub fn record(&mut self, previous: T) {
        self.undo.push(previous);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self, current: T) -> Option<T> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        Some(previous)
    }

    pub fn redo(&mut self, current: T) -> Option<T> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_walk_the_recorded_values_and_a_new_change_drops_the_redo() {
        let mut stack = UndoStack::default();
        stack.record(0);
        stack.record(1);
        assert_eq!(stack.undo(2), Some(1));
        assert_eq!(stack.undo(1), Some(0));
        assert_eq!(stack.redo(0), Some(1));
        stack.record(1);
        assert!(!stack.can_redo());
    }
}
