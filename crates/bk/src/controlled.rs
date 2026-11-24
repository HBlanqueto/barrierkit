use std::cell::RefCell;
use std::rc::Rc;

use freya_core::prelude::*;

pub(crate) fn use_controlled<T: Clone + PartialEq + 'static>(prop: &T) -> State<T> {
    use_controlled_unless(prop, false)
}

// A knob under the pointer is the source of truth: the app's echo of an older value must not pull it back.
pub(crate) fn use_controlled_unless<T: Clone + PartialEq + 'static>(
    prop: &T,
    held: bool,
) -> State<T> {
    let mut value = use_state(|| prop.clone());
    let mut seen = use_state(|| prop.clone());
    if *seen.peek() != *prop {
        seen.set(prop.clone());
        if !held {
            value.set(prop.clone());
        }
    }
    value
}

pub(crate) fn use_latest<T: Clone + 'static>(value: T) -> Rc<RefCell<T>> {
    let slot = use_hook(|| Rc::new(RefCell::new(value.clone())));
    *slot.borrow_mut() = value;
    slot
}

pub(crate) fn commit<T: Copy + PartialEq + 'static>(
    mut value: State<T>,
    on_change: &Option<Callback<T, ()>>,
    new: T,
) {
    if new != *value.peek() {
        value.set(new);
        if let Some(callback) = on_change {
            callback.call(new);
        }
    }
}
