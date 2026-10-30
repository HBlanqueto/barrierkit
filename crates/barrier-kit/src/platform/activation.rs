use sctk::reexports::{
    calloop::ping::Ping,
    client::{
        Connection, Dispatch, Proxy, QueueHandle,
        protocol::{wl_seat, wl_surface},
    },
    protocols::xdg::activation::v1::client::{
        xdg_activation_token_v1::{self, XdgActivationTokenV1},
        xdg_activation_v1::XdgActivationV1,
    },
};
use std::{cell::RefCell, rc::Rc};

use super::{runtime::Runtime, shell_state::ShellState};

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
impl ShellState {
    /// Drains pending activation token requests and asks the compositor for a
    /// token for the first surface.
    pub(crate) fn poll_activation(
        &mut self,
        seat: Option<&wl_seat::WlSeat>,
        window: Option<&wl_surface::WlSurface>,
        serial: u32,
        qh: &QueueHandle<Runtime>,
    ) {
        let requests = std::mem::take(&mut *self.activation.queue.borrow_mut());
        for callback in requests {
            let (Some(global), Some(seat), Some(window)) =
                (self.activation_global.as_ref(), seat, window)
            else {
                callback(None);
                continue;
            };
            if serial == 0 {
                callback(None);
                continue;
            }
            let id = self.activation_callbacks.len();
            self.activation_callbacks.push(Some(callback));
            let token = global.get_activation_token(qh, id);
            token.set_serial(serial, seat);
            token.set_surface(window);
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
            && let Some(shell) = state.extension_mut::<ShellState>()
            && let Some(callback) = shell
                .activation_callbacks
                .get_mut(*id)
                .and_then(Option::take)
        {
            callback(Some(token));
        }
    }
}
