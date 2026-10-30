use super::*;

pub(super) type TimerQueue = Rc<RefCell<Vec<(Duration, Rc<dyn Fn()>)>>>;
pub(super) type WakeQueue = Rc<RefCell<Vec<Rc<dyn Fn()>>>>;

/// Wakes the render loop from another thread. `wake` only pings the loop; the
/// callback registered by [`Timers::wakeup`] runs on the render thread.
#[derive(Clone, Debug)]
pub struct WakeHandle {
    ping: Ping,
}

impl WakeHandle {
    pub fn wake(&self) {
        self.ping.ping();
    }

    pub(crate) fn from_ping(ping: Ping) -> Self {
        Self { ping }
    }
}

#[derive(Clone)]
pub(super) struct Wake {
    pub(super) ping: Ping,
    pub(super) callbacks: WakeQueue,
}

/// Schedules one-shot callbacks on the runner's calloop loop. The app uses it
/// for the menubar clock; there is no periodic polling.
#[derive(Clone)]
pub struct Timers {
    pub(super) queue: TimerQueue,
    pub(super) ping: Ping,
    pub(super) wake: Wake,
}

impl Timers {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn after(&self, delay: Duration, callback: impl Fn() + 'static) {
        self.queue.borrow_mut().push((delay, Rc::new(callback)));
        self.ping.ping();
    }

    /// Registers a callback that runs on the render thread whenever the
    /// returned handle is woken from any thread.
    pub fn wakeup(&self, callback: impl Fn() + 'static) -> WakeHandle {
        self.wake.callbacks.borrow_mut().push(Rc::new(callback));
        WakeHandle {
            ping: self.wake.ping.clone(),
        }
    }
}

/// A calloop waker for Freya's futures and user events.
pub(super) struct PingWake(pub(super) Ping);

impl ArcWake for PingWake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.ping();
    }
}

impl Runtime {
    pub(super) fn install_timers(&mut self, handle: &LoopHandle<Runtime>) {
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
