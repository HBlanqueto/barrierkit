use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};
use sctk::reexports::protocols::xdg::activation::v1::client::xdg_activation_token_v1::{
    self, XdgActivationTokenV1,
};
use sctk::reexports::protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

use super::runtime::Runtime;

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

pub(crate) struct ActivateSurface(WlSurface);

impl Dispatch<XdgActivationTokenV1, ActivateSurface> for Runtime {
    fn event(
        state: &mut Self,
        token: &XdgActivationTokenV1,
        event: xdg_activation_token_v1::Event,
        request: &ActivateSurface,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_activation_token_v1::Event::Done { token: value } = event {
            if let Some(activation) = &state.activation {
                activation.activate(value, &request.0);
            }
            token.destroy();
        }
    }
}

impl Runtime {
    // xdg-activation-v1: a token without a serial lets the compositor decide between focus and attention.
    pub(crate) fn request_activation(&self, surface: &WlSurface) {
        let Some(activation) = &self.activation else {
            return;
        };
        let token = activation.get_activation_token(&self.qh, ActivateSurface(surface.clone()));
        token.set_surface(surface);
        token.commit();
    }

    // A key-only-if-needed panel never gets the keyboard from a press; it asks with the press's
    // token, which Malaysia accepts while the keyboard stays on the same client.
    pub(crate) fn request_keyboard(&self, index: usize, serial: u32) {
        let surface = &self.surfaces[index].surface;
        let asks = self.surfaces[index]
            .context
            .as_ref()
            .is_some_and(|context| context.key_only_if_needed);
        if !asks || self.keyboard_focus.as_ref() == Some(surface) {
            return;
        }
        let (Some(activation), Some(seat)) = (&self.activation, &self.seat_handle) else {
            return;
        };
        let token = activation.get_activation_token(&self.qh, ActivateSurface(surface.clone()));
        token.set_serial(serial, seat);
        token.set_surface(surface);
        token.commit();
    }
}
