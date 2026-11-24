use sctk::reexports::client::{Proxy, protocol::wl_surface::WlSurface};

use crate::platform::glass::GlassPanels;
use crate::platform::linux::window_style::{
    BlendingMode, InfinityWindowStyleV1, Material, offers_backdrop, wire,
};
use crate::platform::runtime::Runtime;
use crate::platform::surface::SurfaceWindow;

/// The protocol's rect array: each `(x, y, width, height)` as four native-endian `i32`s.
pub fn packed(rects: &[(i32, i32, i32, i32)]) -> Vec<u8> {
    rects
        .iter()
        .flat_map(|&(x, y, width, height)| [x, y, width, height])
        .flat_map(i32::to_ne_bytes)
        .collect()
}

const MINIMUM_VERSION: u32 = 6;
const KNOB_VERSION: u32 = 9;

impl Runtime {
    pub(crate) fn glass_style(&self) -> Option<InfinityWindowStyleV1> {
        self.window_style
            .clone()
            .filter(|style| style.version() >= MINIMUM_VERSION)
    }

    pub(crate) fn glass_supported(&self) -> bool {
        self.glass_style().is_some()
    }

    pub(crate) fn backdrop_supported(&self) -> bool {
        self.subcompositor.is_some() && self.window_style.as_ref().is_some_and(offers_backdrop)
    }

    pub(crate) fn knobs_supported(&self) -> bool {
        self.window_style
            .as_ref()
            .is_some_and(|style| style.version() >= KNOB_VERSION)
    }
}

// A compositor that offers `set_backdrop` draws the sidebar as the measured material behind the window.
fn send_panels(
    style: &InfinityWindowStyleV1,
    surface: &WlSurface,
    panels: &GlassPanels,
    measured: bool,
) {
    if measured {
        style.set_backdrop(
            surface,
            BlendingMode::BehindWindow,
            Material::Sidebar,
            panels.radius,
            packed(&panels.rects),
        );
    } else {
        style.set_glass(
            surface,
            panels.radius,
            packed(&panels.rects),
            panels.base,
            panels.solid,
            panels.chroma,
            0,
        );
    }
}

impl SurfaceWindow {
    // The panels go out before the buffer they describe, so both land in one commit.
    pub(crate) fn flush_glass(&mut self) {
        let (Some(panels), Some(style)) = (self.app.glass.take(), &self.glass_style) else {
            return;
        };
        send_panels(style, &self.surface, &panels, self.chrome_host.is_some());
        self.glass_holes = panels.rects;
    }

    pub(crate) fn flush_knobs(&mut self) {
        let (Some(knobs), Some(style)) = (self.app.knobs.take(), &self.glass_style) else {
            return;
        };
        style.set_knobs(
            &self.surface,
            wire(knobs.kind as u32),
            knobs.tint,
            packed(&knobs.rects),
        );
    }
}
