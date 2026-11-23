use super::calloop::ping::Ping;
use futures_util::task::ArcWake;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(crate) type TimerQueue = Rc<RefCell<Vec<(Duration, Rc<dyn Fn()>)>>>;
pub(crate) type WakeQueue = Rc<RefCell<Vec<Rc<dyn Fn()>>>>;

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

pub(crate) struct WakeRegistration {
    callbacks: WakeQueue,
    callback: Rc<dyn Fn()>,
}

impl Drop for WakeRegistration {
    fn drop(&mut self) {
        self.callbacks
            .borrow_mut()
            .retain(|callback| !Rc::ptr_eq(callback, &self.callback));
    }
}

#[derive(Clone)]
pub(crate) struct Wake {
    pub(crate) ping: Ping,
    pub(crate) callbacks: WakeQueue,
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

    pub fn every(&self, interval: Duration, callback: impl Fn() + 'static) {
        self.repeat(Rc::new(move || interval), Rc::new(callback));
    }

    pub fn every_minute(&self, callback: impl Fn() + 'static) {
        self.repeat(Rc::new(until_next_minute), Rc::new(callback));
    }

    fn repeat(&self, delay: Rc<dyn Fn() -> Duration>, callback: Rc<dyn Fn()>) {
        let timers = self.clone();
        let (next_delay, next_callback) = (delay.clone(), callback);
        self.after(delay(), move || {
            next_callback();
            timers.repeat(next_delay.clone(), next_callback.clone());
        });
    }

    pub fn wakeup(&self, callback: impl Fn() + 'static) -> WakeHandle {
        self.wake.callbacks.borrow_mut().push(Rc::new(callback));
        WakeHandle {
            ping: self.wake.ping.clone(),
        }
    }

    pub(crate) fn wakeup_until_dropped(
        &self,
        callback: impl Fn() + 'static,
    ) -> (WakeHandle, Rc<WakeRegistration>) {
        let callback: Rc<dyn Fn()> = Rc::new(callback);
        self.wake.callbacks.borrow_mut().push(callback.clone());
        let registration = WakeRegistration {
            callbacks: self.wake.callbacks.clone(),
            callback,
        };
        (
            WakeHandle::from_ping(self.wake.ping.clone()),
            Rc::new(registration),
        )
    }
}

fn seconds_to_next_minute(unix_seconds: u64) -> u64 {
    60 - unix_seconds % 60
}

fn until_next_minute() -> Duration {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    Duration::from_secs(seconds_to_next_minute(now))
}

pub(crate) struct PingWake(pub(crate) Ping);

impl ArcWake for PingWake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.ping();
    }
}

#[cfg(test)]
mod tests {
    use super::{Timers, Wake, seconds_to_next_minute};
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn the_minute_that_just_started_waits_a_whole_minute() {
        assert_eq!(seconds_to_next_minute(0), 60);
        assert_eq!(seconds_to_next_minute(120), 60);
    }

    #[test]
    fn the_last_second_of_a_minute_waits_one_second() {
        assert_eq!(seconds_to_next_minute(59), 1);
        assert_eq!(seconds_to_next_minute(179), 1);
    }

    #[test]
    fn a_dropped_wakeup_registration_stops_its_callback_and_frees_its_slot() {
        let (ping, _source) = crate::platform::calloop::ping::make_ping().unwrap();
        let timers = Timers {
            queue: Rc::default(),
            ping: ping.clone(),
            wake: Wake {
                ping,
                callbacks: Rc::default(),
            },
        };
        let calls = Rc::new(Cell::new(0));
        let counter = calls.clone();
        let (_, kept) = timers.wakeup_until_dropped(move || counter.set(counter.get() + 1));
        for _ in 0..100 {
            drop(timers.wakeup_until_dropped(|| {}));
        }
        let callbacks = timers.wake.callbacks.borrow().clone();
        assert_eq!(callbacks.len(), 1);
        callbacks.iter().for_each(|callback| callback());
        drop(kept);
        assert!(timers.wake.callbacks.borrow().is_empty());
        assert_eq!(calls.get(), 1);
    }
}
