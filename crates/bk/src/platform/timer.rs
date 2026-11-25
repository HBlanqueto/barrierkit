use super::calloop::ping::Ping;
use futures_util::task::ArcWake;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub(crate) type TimerQueue = Rc<RefCell<Vec<(Duration, Rc<dyn Fn()>)>>>;
pub(crate) type WakeQueue = Rc<RefCell<Vec<(Arc<AtomicBool>, Rc<dyn Fn()>)>>>;

#[derive(Clone, Debug)]
pub struct WakeHandle {
    ping: Ping,
    invalidated: Arc<AtomicBool>,
}

impl WakeHandle {
    pub fn wake(&self) {
        self.ping.ping();
    }

    /// Unregisters the `wakeup` callback this handle came from; the next wake drops it.
    pub fn invalidate(&self) {
        self.invalidated.store(true, Ordering::Relaxed);
        self.ping.ping();
    }

    pub(crate) fn from_ping(ping: Ping) -> Self {
        Self {
            ping,
            invalidated: Arc::default(),
        }
    }
}

pub(crate) struct WakeRegistration(WakeHandle);

impl Drop for WakeRegistration {
    fn drop(&mut self) {
        self.0.invalidated.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone, Default)]
pub struct Timer(Rc<Cell<bool>>);

impl Timer {
    pub fn invalidate(&self) {
        self.0.set(true);
    }
}

#[derive(Clone)]
pub(crate) struct Wake {
    pub(crate) ping: Ping,
    pub(crate) callbacks: WakeQueue,
}

impl Wake {
    pub(crate) fn live_callbacks(&self) -> Vec<Rc<dyn Fn()>> {
        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.retain(|(invalidated, _)| !invalidated.load(Ordering::Relaxed));
        callbacks
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect()
    }
}

#[derive(Clone)]
pub struct Timers {
    pub(crate) queue: TimerQueue,
    pub(crate) ping: Ping,
    pub(crate) wake: Wake,
}

impl Timers {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn after(&self, delay: Duration, callback: impl Fn() + 'static) {
        self.queue.borrow_mut().push((delay, Rc::new(callback)));
        self.ping.ping();
    }

    pub fn every(&self, interval: Duration, callback: impl Fn() + 'static) -> Timer {
        let timer = Timer::default();
        self.repeat(Rc::new(move || interval), Rc::new(callback), timer.clone());
        timer
    }

    fn repeat(&self, delay: Rc<dyn Fn() -> Duration>, callback: Rc<dyn Fn()>, timer: Timer) {
        let timers = self.clone();
        let next_delay = delay.clone();
        self.after(delay(), move || {
            if timer.0.get() {
                return;
            }
            callback();
            timers.repeat(next_delay.clone(), callback.clone(), timer.clone());
        });
    }

    pub fn wakeup(&self, callback: impl Fn() + 'static) -> WakeHandle {
        let handle = WakeHandle::from_ping(self.wake.ping.clone());
        self.wake
            .callbacks
            .borrow_mut()
            .push((handle.invalidated.clone(), Rc::new(callback)));
        handle
    }

    pub(crate) fn wakeup_until_dropped(
        &self,
        callback: impl Fn() + 'static,
    ) -> (WakeHandle, Rc<WakeRegistration>) {
        let handle = self.wakeup(callback);
        (handle.clone(), Rc::new(WakeRegistration(handle)))
    }
}

pub(crate) struct PingWake(pub(crate) Ping);

impl ArcWake for PingWake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.ping();
    }
}
