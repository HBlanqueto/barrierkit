#[cfg(target_os = "linux")]
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle, globals::GlobalList};

#[cfg(target_os = "linux")]
use super::runtime::Runtime;

pub use infinity_protocol::client::style::infinity_window_style_v1::infinity_window_style_v1::{
    Bar, PopoverShape,
};
pub(crate) use infinity_protocol::client::style::infinity_window_style_v1::infinity_window_style_v1::{
    Edge, Knob, Material as WireMaterial, Panel, ScrollEdgeEffectStyle,
};
#[cfg(target_os = "linux")]
pub(crate) use infinity_protocol::client::style::infinity_window_style_v1::infinity_window_style_v1::{
    BlendingMode, InfinityWindowStyleV1,
};

pub(crate) const BACKDROP_VERSION: u32 = 10;

// A request newer than the bound version is a protocol error (ADR 0091): bind at what the compositor offers.
#[cfg(target_os = "linux")]
pub(crate) fn bind<State>(
    globals: &GlobalList,
    queue: &QueueHandle<State>,
) -> Option<InfinityWindowStyleV1>
where
    State: Dispatch<InfinityWindowStyleV1, ()> + 'static,
{
    globals.bind(queue, 1..=BACKDROP_VERSION, ()).ok()
}

#[cfg(target_os = "linux")]
pub(crate) fn offers_backdrop(style: &InfinityWindowStyleV1) -> bool {
    style.version() >= BACKDROP_VERSION
}

#[cfg(target_os = "linux")]
impl Dispatch<InfinityWindowStyleV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &InfinityWindowStyleV1,
        _: <InfinityWindowStyleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
