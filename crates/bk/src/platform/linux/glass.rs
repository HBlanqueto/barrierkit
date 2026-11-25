use sctk::reexports::client::Proxy;

use crate::platform::linux::{
    effects::{BlendingMode, Material},
    window_style::{LimitlessWindowStyleV1, wire},
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

const KNOB_VERSION: u32 = 9;

impl Runtime {
    pub(crate) fn knobs_style(&self) -> Option<LimitlessWindowStyleV1> {
        self.window_style
            .clone()
            .filter(|style| style.version() >= KNOB_VERSION)
    }

    pub(crate) fn glass_supported(&self) -> bool {
        self.effects.is_some()
    }

    pub(crate) fn backdrop_supported(&self) -> bool {
        self.subcompositor.is_some() && self.effects.is_some()
    }

    pub(crate) fn knobs_supported(&self) -> bool {
        self.knobs_style().is_some()
    }
}

impl SurfaceWindow {
    // The panels go out before the buffer they describe, so both land in one commit.
    pub(crate) fn flush_glass(&mut self) {
        let (Some(panels), Some(effects)) = (self.app.glass.take(), &self.effects) else {
            return;
        };
        let sidebar = self.sidebar.get_or_insert_with(|| {
            let sidebar = effects.visual(&self.surface);
            sidebar.set_material(Material::Sidebar);
            sidebar.set_blending_mode(BlendingMode::BehindWindow);
            sidebar
        });
        sidebar.set_shape(panels.radius, packed(&panels.rects));
        self.glass_holes = panels.rects;
    }

    pub(crate) fn flush_knobs(&mut self) {
        let (Some(knobs), Some(style)) = (self.app.knobs.take(), &self.knobs_style) else {
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
