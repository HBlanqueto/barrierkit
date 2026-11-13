use sctk::reexports::client::Proxy;
use sctk::reexports::client::backend::ObjectId;

use super::KitSender;
use super::runtime::Runtime;

const MINIMUM_VERSION: u32 = 6;

pub(crate) type Rects = Vec<(i32, i32, i32, i32)>;

#[derive(Clone, Copy)]
pub(crate) struct GlassSupport(pub(crate) bool);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlassPanels {
    pub(crate) radius: u32,
    pub(crate) rects: Rects,
    pub(crate) base: u32,
    pub(crate) solid: u32,
    pub(crate) chroma: u32,
    pub(crate) rim: u32,
}

pub(crate) fn set_glass_panels(panels: GlassPanels) {
    KitSender::get().send(panels);
}

impl Runtime {
    pub(crate) fn glass_supported(&self) -> bool {
        self.window_style
            .as_ref()
            .is_some_and(|style| style.version() >= MINIMUM_VERSION)
    }

    pub(crate) fn apply_glass_panels(&mut self, surface_id: &ObjectId, panels: GlassPanels) {
        let Some(index) = self.surface_index_by_id(surface_id) else {
            return;
        };
        let Some(style) = self
            .window_style
            .as_ref()
            .filter(|style| style.version() >= MINIMUM_VERSION)
        else {
            return;
        };
        let rects = panels
            .rects
            .iter()
            .flat_map(|&(x, y, width, height)| [x, y, width, height])
            .flat_map(i32::to_ne_bytes)
            .collect();
        style.set_glass(
            &self.surfaces[index].surface,
            panels.radius,
            rects,
            panels.base,
            panels.solid,
            panels.chroma,
            panels.rim,
        );
        self.surfaces[index].glass_holes = panels.rects;
        self.surfaces[index].needs_redraw = true;
    }
}
