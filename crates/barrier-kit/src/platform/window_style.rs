use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use super::runtime::Runtime;

pub(crate) use infinity_protocol::client::style::infinity_window_style_v1::infinity_window_style_v1::{
    Bar, InfinityWindowStyleV1, Panel,
};

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
