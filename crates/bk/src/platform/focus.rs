use freya_core::prelude::WritableUtils;
use sctk::reexports::client::backend::ObjectId;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use super::runtime::Runtime;

pub(crate) use infinity_protocol::client::focus::infinity_focus_v1::infinity_focus_manager_v1::InfinityFocusManagerV1;
use infinity_protocol::client::focus::infinity_focus_v1::infinity_focus_surface_v1;
pub(crate) use infinity_protocol::client::focus::infinity_focus_v1::infinity_focus_surface_v1::InfinityFocusSurfaceV1;

impl Dispatch<InfinityFocusManagerV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &InfinityFocusManagerV1,
        _: <InfinityFocusManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<InfinityFocusSurfaceV1, ObjectId> for Runtime {
    fn event(
        state: &mut Self,
        _: &InfinityFocusSurfaceV1,
        event: infinity_focus_surface_v1::Event,
        surface_id: &ObjectId,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(index) = state.surface_index_by_id(surface_id) else {
            return;
        };
        if let infinity_focus_surface_v1::Event::Wallpaper { rgb } = event {
            let [_, r, g, b] = rgb.to_be_bytes();
            let global = *state.resolved.peek();
            if let Some(context) = state.surfaces[index].context.as_mut() {
                context.fade_wallpaper(ctdesign::Rgba::rgb8(r, g, b), global);
                state.request_redraw_all = true;
                state.ping.ping();
            }
            return;
        }
        let Some(context) = state.surfaces[index].context.as_ref() else {
            return;
        };
        let revealed = match event {
            infinity_focus_surface_v1::Event::Bar { revealed } => revealed,
            infinity_focus_surface_v1::Event::Desktop { desktop } => {
                let mut shown = context.controls.desktop;
                let number = (desktop > 0).then_some(desktop);
                if *shown.peek() != number {
                    shown.set(number);
                    state.ping.ping();
                }
                return;
            }
            infinity_focus_surface_v1::Event::BarHeight { height } => {
                let mut bar_height = context.controls.focus_bar_height;
                if *bar_height.peek() != height as f32 {
                    bar_height.set(height as f32);
                    state.request_redraw_all = true;
                    state.ping.ping();
                }
                return;
            }
            _ => return,
        };
        tracing::debug!(target: "bk::focus", revealed, "focus bar");
        let mut focus_bar = context.controls.focus_bar;
        if *focus_bar.peek() != (revealed != 0) {
            focus_bar.set(revealed != 0);
            state.request_redraw_all = true;
            state.ping.ping();
        }
    }
}
