use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle, globals::GlobalList};

use crate::platform::{PanelBehavior, runtime::Runtime};

pub(crate) use limitless_protocol::client::style::limitless_window_style_v1::limitless_window_style_v1::{
    LimitlessWindowStyleV1, Panel,
};

pub(crate) fn wire<T: TryFrom<u32, Error = ()>>(value: u32) -> T {
    T::try_from(value).expect("crate::platform::window_style declares the protocol's values")
}

impl PanelBehavior {
    pub(super) fn flags(self) -> Panel {
        [
            (self.hides_on_deactivate, Panel::HidesOnDeactivate),
            (self.floating, Panel::Floating),
            (self.key_only_if_needed, Panel::KeyOnlyIfNeeded),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .fold(Panel::empty(), |flags, (_, flag)| flags | flag)
    }
}

const VERSION: u32 = 9;

// A request newer than the bound version is a protocol error (ADR 0091): bind at what the compositor offers.
pub(crate) fn bind<State>(
    globals: &GlobalList,
    queue: &QueueHandle<State>,
) -> Option<LimitlessWindowStyleV1>
where
    State: Dispatch<LimitlessWindowStyleV1, ()> + 'static,
{
    globals.bind(queue, 1..=VERSION, ()).ok()
}

impl Dispatch<LimitlessWindowStyleV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &LimitlessWindowStyleV1,
        _: <LimitlessWindowStyleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
