use std::cell::RefCell;
use std::rc::Rc;

use super::window_style::Knob;

pub(crate) type Rects = Vec<(i32, i32, i32, i32)>;

#[derive(Clone, Copy)]
pub(crate) struct GlassSupport(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct KnobSupport(pub(crate) bool);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlassPanels {
    pub(crate) radius: u32,
    pub(crate) rects: Rects,
    pub(crate) inspector: Option<(i32, i32, i32, i32)>,
    // The kit draws the sidebar's contents in a chrome above its glass, so the glass lies above the content.
    pub(crate) contents_above: bool,
}

// The last panels and whether the platform has yet to take them.
#[derive(Clone, Default)]
pub(crate) struct GlassOutbox(Rc<RefCell<(GlassPanels, bool)>>);

impl GlassOutbox {
    pub(crate) fn get() -> Self {
        freya_core::prelude::try_consume_root_context().unwrap_or_default()
    }

    // The sidebar and the inspector publish apart, so each keeps the other's last panel.
    pub(crate) fn put(&self, panels: GlassPanels) {
        let mut outbox = self.0.borrow_mut();
        let inspector = outbox.0.inspector;
        *outbox = (
            GlassPanels {
                inspector,
                ..panels
            },
            true,
        );
    }

    pub(crate) fn put_inspector(&self, inspector: Option<(i32, i32, i32, i32)>) {
        let mut outbox = self.0.borrow_mut();
        outbox.0.inspector = inspector;
        outbox.1 = true;
    }

    pub(crate) fn take(&self) -> Option<GlassPanels> {
        let mut outbox = self.0.borrow_mut();
        std::mem::take(&mut outbox.1).then(|| outbox.0.clone())
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

    pub(super) fn take(&self) -> Option<Knobs> {
        self.0.borrow_mut().take()
    }
}
