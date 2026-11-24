use std::cell::RefCell;
use std::rc::Rc;

use freya_core::prelude::*;

// Binds a control to a value its parent derives: the State follows `current`, and a change made through it goes to `changed`.
pub(crate) fn use_mirror<T: Clone + PartialEq + 'static>(
    current: T,
    changed: impl FnMut(T) + 'static,
) -> State<T> {
    let mut value = use_state(|| current.clone());
    if *value.peek() != current {
        value.set(current.clone());
    }
    let latest = use_latest((current, Rc::new(RefCell::new(changed))));
    use_side_effect(move || {
        let next = value.read().clone();
        let (current, changed) = latest.borrow().clone();
        if next != current {
            (changed.borrow_mut())(next);
        }
    });
    value
}

pub(crate) fn use_latest<T: Clone + 'static>(value: T) -> Rc<RefCell<T>> {
    let slot = use_hook(|| Rc::new(RefCell::new(value.clone())));
    *slot.borrow_mut() = value;
    slot
}

pub(crate) fn commit<T: PartialEq + 'static>(mut value: State<T>, new: T) {
    if new != *value.peek() {
        value.set(new);
    }
}
