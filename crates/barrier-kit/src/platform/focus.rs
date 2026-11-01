//! MalaysiaDreams focus-mode notices (`infinity_focus_v1`, ADR 0051).
//!
//! In focus mode a window is fullscreen in a space of its own and hides its
//! traffic lights. When the pointer reaches the top edge, the compositor
//! brings the bar down, moves a client-decorated window down under it and
//! sends `bar(1)`: the window shows its lights until `bar(0)`.

use freya_core::prelude::WritableUtils;
use sctk::reexports::client::backend::ObjectId;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use super::runtime::Runtime;

pub(crate) use infinity_protocol::client::focus::infinity_focus_v1::infinity_focus_manager_v1::InfinityFocusManagerV1;
use infinity_protocol::client::focus::infinity_focus_v1::infinity_focus_surface_v1::{
    self, InfinityFocusSurfaceV1,
};

impl Dispatch<InfinityFocusManagerV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &InfinityFocusManagerV1,
        _: <InfinityFocusManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // The manager has no events.
    }
}

/// The user data is the id of the toplevel surface the notices are about.
impl Dispatch<InfinityFocusSurfaceV1, ObjectId> for Runtime {
    fn event(
        state: &mut Self,
        _: &InfinityFocusSurfaceV1,
        event: infinity_focus_surface_v1::Event,
        surface_id: &ObjectId,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let infinity_focus_surface_v1::Event::Bar { revealed } = event else {
            return;
        };
        let Some(context) = state
            .window_contexts
            .iter()
            .find(|context| context.controls.surface_id == *surface_id)
        else {
            return;
        };
        let mut focus_bar = context.controls.focus_bar;
        if *focus_bar.peek() != (revealed != 0) {
            focus_bar.set(revealed != 0);
            state.request_redraw_all = true;
            state.ping.ping();
        }
    }
}
