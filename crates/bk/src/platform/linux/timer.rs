use std::{rc::Rc, time::Duration};

use crate::platform::calloop::{
    LoopHandle,
    timer::{TimeoutAction, Timer},
};
use crate::platform::runtime::Runtime;

impl Runtime {
    pub(crate) fn install_timers(&mut self, handle: &LoopHandle<Runtime>) {
        let pending: Vec<(Duration, Rc<dyn Fn()>)> = std::mem::take(&mut *self.timers.borrow_mut());
        for (delay, callback) in pending {
            let _ = handle.insert_source(
                Timer::from_duration(delay),
                move |_, _, shell: &mut Runtime| {
                    callback();
                    shell.poll_all();
                    shell.redraw_all();
                    TimeoutAction::Drop
                },
            );
        }
    }
}
