use sctk::reexports::{
    calloop::ping::Ping,
    client::{Connection, Dispatch, Proxy, QueueHandle},
    protocols::xdg::activation::v1::client::{
        xdg_activation_token_v1::{self, XdgActivationTokenV1},
        xdg_activation_v1::XdgActivationV1,
    },
};
use std::{cell::RefCell, rc::Rc};

use super::runtime::Runtime;

pub(super) type ActivationCallback = Box<dyn Fn(Option<String>)>;
pub(super) type ActivationQueue = Rc<RefCell<Vec<ActivationCallback>>>;

/// Requests an `xdg-activation-v1` token for the next launched app.
///
/// The compositor only accepts a token created by the client that currently
/// has keyboard focus and that carries the last key press serial. The runner
/// owns both, so the app asks here and receives the token (or `None` when the
/// protocol is unavailable) in a callback.
#[derive(Clone)]
pub struct Activation {
    pub(super) queue: ActivationQueue,
    pub(super) ping: Ping,
}

impl Activation {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    /// Queues one token request. The callback runs on the Wayland loop with the
    /// token string, or with `None` if the compositor does not support it.
    pub fn request(&self, callback: impl Fn(Option<String>) + 'static) {
        self.queue.borrow_mut().push(Box::new(callback));
        self.ping.ping();
    }
}

impl Runtime {
    pub(super) fn poll_activation(&mut self) {
        let requests = std::mem::take(&mut *self.activation.queue.borrow_mut());
        for callback in requests {
            let (Some(global), Some(seat), Some(window)) = (
                self.activation_global.as_ref(),
                self.seat_handle.as_ref(),
                self.surfaces.first(),
            ) else {
                callback(None);
                continue;
            };
            let serial = self.last_input_serial.get();
            if serial == 0 {
                callback(None);
                continue;
            }
            let id = self.activation_callbacks.len();
            self.activation_callbacks.push(Some(callback));
            let token = global.get_activation_token(&self.qh, id);
            token.set_serial(serial, seat);
            token.set_surface(&window.surface);
            token.commit();
        }
    }
}

impl Dispatch<XdgActivationV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &XdgActivationV1,
        _: <XdgActivationV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<XdgActivationTokenV1, usize> for Runtime {
    fn event(
        state: &mut Self,
        _: &XdgActivationTokenV1,
        event: xdg_activation_token_v1::Event,
        id: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_activation_token_v1::Event::Done { token } = event
            && let Some(callback) = state
                .activation_callbacks
                .get_mut(*id)
                .and_then(Option::take)
        {
            callback(Some(token));
        }
    }
}
