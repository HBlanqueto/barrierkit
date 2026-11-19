use std::cell::RefCell;
use std::rc::Rc;

use sctk::reexports::client::Proxy;

use super::runtime::Runtime;
use super::surface::SurfaceWindow;
use super::window_style::{InfinityWindowStyleV1, Knob};

const MINIMUM_VERSION: u32 = 6;
const KNOB_VERSION: u32 = 9;

pub(crate) type Rects = Vec<(i32, i32, i32, i32)>;

pub(crate) fn packed(rects: &[(i32, i32, i32, i32)]) -> Vec<u8> {
    rects
        .iter()
        .flat_map(|&(x, y, width, height)| [x, y, width, height])
        .flat_map(i32::to_ne_bytes)
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) struct GlassSupport(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct KnobSupport(pub(crate) bool);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlassPanels {
    pub(crate) radius: u32,
    pub(crate) rects: Rects,
    pub(crate) base: u32,
    pub(crate) solid: u32,
    pub(crate) chroma: u32,
}

#[derive(Clone, Default)]
pub(crate) struct GlassOutbox(Rc<RefCell<Option<GlassPanels>>>);

impl GlassOutbox {
    pub(crate) fn get() -> Self {
        freya_core::prelude::try_consume_root_context().unwrap_or_default()
    }

    pub(crate) fn put(&self, panels: GlassPanels) {
        *self.0.borrow_mut() = Some(panels);
    }

    fn take(&self) -> Option<GlassPanels> {
        self.0.borrow_mut().take()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Knobs {
    pub(crate) kind: Knob,
    pub(crate) tint: u32,
    pub(crate) rects: Rects,
}

impl Default for Knobs {
    fn default() -> Self {
        Self {
            kind: Knob::Slider,
            tint: 0,
            rects: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct KnobOutbox(Rc<RefCell<Option<Knobs>>>);

impl KnobOutbox {
    pub(crate) fn get() -> Self {
        freya_core::prelude::try_consume_root_context().unwrap_or_default()
    }

    pub(crate) fn put(&self, knobs: Knobs) {
        *self.0.borrow_mut() = Some(knobs);
    }

    fn take(&self) -> Option<Knobs> {
        self.0.borrow_mut().take()
    }
}

impl Runtime {
    pub(crate) fn glass_style(&self) -> Option<InfinityWindowStyleV1> {
        self.window_style
            .clone()
            .filter(|style| style.version() >= MINIMUM_VERSION)
    }

    pub(crate) fn glass_supported(&self) -> bool {
        self.glass_style().is_some()
    }

    pub(crate) fn knobs_supported(&self) -> bool {
        self.window_style
            .as_ref()
            .is_some_and(|style| style.version() >= KNOB_VERSION)
    }
}

impl SurfaceWindow {
    // The panels go out before the buffer they describe, so both land in one commit.
    pub(crate) fn flush_glass(&mut self) {
        let (Some(panels), Some(style)) = (self.app.glass.take(), &self.glass_style) else {
            return;
        };
        style.set_glass(
            &self.surface,
            panels.radius,
            packed(&panels.rects),
            panels.base,
            panels.solid,
            panels.chroma,
            0,
        );
        self.glass_holes = panels.rects;
    }

    pub(crate) fn flush_knobs(&mut self) {
        let (Some(knobs), Some(style)) = (self.app.knobs.take(), &self.glass_style) else {
            return;
        };
        style.set_knobs(&self.surface, knobs.kind, knobs.tint, packed(&knobs.rects));
    }
}
