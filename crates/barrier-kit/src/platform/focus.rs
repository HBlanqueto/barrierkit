//! MalaysiaDreams focus-mode notices (`infinity_focus_v1`, ADR 0051).
//!
//! In focus mode a window is fullscreen in a space of its own and hides its
//! traffic lights. When the pointer reaches the top edge, the compositor
//! brings the bar down and sends `bar(1)`: the window shows its lights until
//! `bar(0)`. With version 2 the kit calls `set_overlay`: the window is not
//! moved, the titlebar leaves the layout in fullscreen and is drawn over the
//! content right below the bar (`bar_height`) while it is down (ADR 0054).
//! With version 1 the compositor moves the window down under the bar.

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
        let Some(context) = state
            .window_contexts
            .iter()
            .find(|context| context.controls.surface_id == *surface_id)
        else {
            return;
        };
        let revealed = match event {
            infinity_focus_surface_v1::Event::Bar { revealed } => revealed,
            infinity_focus_surface_v1::Event::BarHeight { height } => {
                // Comes right before bar(1), which redraws.
                let mut bar_height = context.controls.focus_bar_height;
                if *bar_height.peek() != height as f32 {
                    bar_height.set(height as f32);
                }
                return;
            }
            _ => return,
        };
        let mut focus_bar = context.controls.focus_bar;
        if *focus_bar.peek() != (revealed != 0) {
            focus_bar.set(revealed != 0);
            state.request_redraw_all = true;
            state.ping.ping();
        }
    }
}
