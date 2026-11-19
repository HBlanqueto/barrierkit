use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use freya_core::prelude::*;

use crate::platform::Timers;

pub fn use_timeline(active: bool, every: Duration) {
    let ticks = use_state(|| 0u64);
    let scheduled = use_hook(|| Rc::new(Cell::new(false)));
    let _ = *ticks.read();
    if active && !scheduled.replace(true) {
        Timers::get().after(every, move || {
            scheduled.set(false);
            let mut ticks = ticks;
            *ticks.write() += 1;
        });
    }
}
