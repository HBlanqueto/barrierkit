use std::sync::{Arc, Mutex, PoisonError};

use freya_core::prelude::*;

use crate::platform::{Timers, WakeHandle};

pub struct Publisher<T> {
    latest: Arc<Mutex<T>>,
    wake: WakeHandle,
}

impl<T> Publisher<T> {
    pub fn send(&self, value: T) {
        *self.latest.lock().unwrap_or_else(PoisonError::into_inner) = value;
        self.wake.wake();
    }
}

pub fn use_published<T: Clone + PartialEq + Send + 'static>(
    initial: T,
    start: impl FnOnce(Publisher<T>) + 'static,
) -> State<T> {
    let state = use_state(|| initial.clone());
    use_hook(move || {
        let latest = Arc::new(Mutex::new(initial));
        let shared = latest.clone();
        let wake = Timers::get().wakeup(move || {
            let mut state = state;
            state.set_if_modified(
                shared
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone(),
            );
        });
        start(Publisher { latest, wake });
    });
    state
}

pub fn use_wake(start: impl FnOnce(WakeHandle) + 'static) -> u64 {
    let ticks = use_state(|| 0u64);
    use_hook(move || {
        start(Timers::get().wakeup(move || {
            let mut ticks = ticks;
            *ticks.write() += 1;
        }));
    });
    *ticks.read()
}
