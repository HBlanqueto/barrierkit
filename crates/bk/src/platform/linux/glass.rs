use sctk::reexports::client::{Proxy, protocol::wl_surface::WlSurface};

use crate::platform::glass::{GlassPanels, packed};
use crate::platform::linux::window_style::{BlendingMode, InfinityWindowStyleV1, offers_backdrop};
use crate::platform::runtime::Runtime;
use crate::platform::surface::SurfaceWindow;
use crate::platform::window_style::WireMaterial;

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
            WireMaterial::Sidebar,
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
        style.set_knobs(&self.surface, knobs.kind, knobs.tint, packed(&knobs.rects));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::linux::fake_server::{Probe, Seen};

    fn panels() -> GlassPanels {
        GlassPanels {
            radius: 16,
            rects: vec![(0, 0, 200, 600)],
            base: 1,
            solid: 2,
            chroma: 3,
        }
    }

    fn sent(version: u32, measured: bool) -> Vec<Seen> {
        let (mut probe, mut queue, server) = Probe::connect(Some(version));
        let surface = probe.surface();
        send_panels(probe.style.as_ref().unwrap(), &surface, &panels(), measured);
        queue.roundtrip(&mut probe).unwrap();
        server.seen()
    }

    #[test]
    fn a_compositor_with_version_ten_gets_the_sidebar_as_the_measured_material_behind_the_window() {
        assert!(matches!(
            sent(10, true).as_slice(),
            [Seen::Backdrop { blending: 0, material: 4, radius: 16, rects, .. }] if rects == &[0, 0, 200, 600]
        ));
    }

    #[test]
    fn an_older_compositor_still_gets_the_glass_panels() {
        assert!(matches!(
            sent(9, false).as_slice(),
            [Seen::Glass { radius: 16, rects, base: 1, solid: 2, chroma: 3, .. }] if rects == &[0, 0, 200, 600]
        ));
    }

    #[test]
    fn an_empty_sidebar_removes_the_material() {
        let (mut probe, mut queue, server) = Probe::connect(Some(10));
        let surface = probe.surface();
        send_panels(
            probe.style.as_ref().unwrap(),
            &surface,
            &GlassPanels::default(),
            true,
        );
        queue.roundtrip(&mut probe).unwrap();
        assert!(matches!(
            server.seen().as_slice(),
            [Seen::Backdrop { rects, .. }] if rects.is_empty()
        ));
    }
}
