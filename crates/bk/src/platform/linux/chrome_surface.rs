use std::rc::Rc;

use freya_core::node_id::NodeId;
use sctk::{
    reexports::{
        client::{
            Proxy, QueueHandle,
            protocol::{wl_subsurface::WlSubsurface, wl_surface::WlSurface},
        },
        protocols::wp::viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
    },
    subcompositor::SubcompositorState,
};

use super::{
    runtime::Runtime,
    surface::{SurfaceRole, SurfaceWindow, buffer_size},
    vulkan::{Graphics, GraphicsShared},
};
use crate::platform::{
    RunError,
    backdrop::{Chrome, Placement, Requests},
    glass::packed,
    linux::window_style::{BlendingMode, InfinityWindowStyleV1},
};

#[derive(Clone)]
pub(crate) struct ChromeHost {
    pub(crate) subcompositor: Rc<SubcompositorState>,
    pub(crate) style: InfinityWindowStyleV1,
    pub(crate) viewporter: Option<WpViewporter>,
    pub(crate) queue: QueueHandle<Runtime>,
}

// Synchronized: its commit waits for the parent's, so the material and the content below it change in one frame (ADR 0092).
pub(crate) struct ChromeSurface {
    root: NodeId,
    pub(crate) nodes: Rc<[NodeId]>,
    surface: WlSurface,
    subsurface: WlSubsurface,
    viewport: Option<WpViewport>,
    pub(crate) graphics: Option<Graphics>,
    pub(crate) placement: Placement,
    configured: Option<((u32, u32), f64)>,
    sent: Requests,
}

impl ChromeSurface {
    fn open(
        host: &ChromeHost,
        parent: &WlSurface,
        chrome: &Chrome,
        scaled_by_viewport: bool,
    ) -> Self {
        let (subsurface, surface) = host
            .subcompositor
            .create_subsurface(parent.clone(), &host.queue);
        let viewport = host
            .viewporter
            .as_ref()
            .filter(|_| scaled_by_viewport)
            .map(|viewporter| viewporter.get_viewport(&surface, &host.queue, surface.id()));
        Self {
            root: chrome.root,
            nodes: Rc::default(),
            surface,
            subsurface,
            viewport,
            graphics: None,
            placement: Placement {
                origin: (0, 0),
                size: (0, 0),
            },
            configured: None,
            sent: Requests::default(),
        }
    }

    fn configure(
        &mut self,
        shared: &Rc<GraphicsShared>,
        size: (u32, u32),
        scale: f64,
    ) -> Result<(), RunError> {
        let graphics = self
            .graphics
            .get_or_insert_with(|| Graphics::new(shared.clone()));
        graphics.configure(&self.surface, buffer_size(size, scale))?;
        match &self.viewport {
            Some(viewport) => {
                self.surface.set_buffer_scale(1);
                viewport.set_destination(size.0 as i32, size.1 as i32);
            }
            None => self.surface.set_buffer_scale(scale.round().max(1.0) as i32),
        }
        self.configured = Some((size, scale));
        Ok(())
    }

    fn update(
        &mut self,
        host: &ChromeHost,
        shared: &Rc<GraphicsShared>,
        chrome: &Chrome,
        scale: f64,
    ) -> bool {
        let placement = chrome.placement(scale as f32);
        self.nodes.clone_from(&chrome.nodes);
        if placement.origin != self.placement.origin {
            self.subsurface
                .set_position(placement.origin.0, placement.origin.1);
        }
        self.placement = placement;
        if self.configured != Some((placement.size, scale))
            && let Err(error) = self.configure(shared, placement.size, scale)
        {
            tracing::error!("BarrierKit: the chrome surface: {error}");
            return false;
        }
        let requests = Requests::of(chrome, scale as f32);
        send_requests(&host.style, &self.surface, &self.sent, &requests);
        self.sent = requests;
        true
    }
}

// The requests go out before the buffer they describe, so both land in the parent's next commit.
fn send_requests(
    style: &InfinityWindowStyleV1,
    surface: &WlSurface,
    old: &Requests,
    new: &Requests,
) {
    if old.backdrop != new.backdrop
        && let Some(material) = new.backdrop.material.or(old.backdrop.material)
    {
        style.set_backdrop(
            surface,
            BlendingMode::WithinWindow,
            material,
            new.backdrop.radius,
            packed(&new.backdrop.rects),
        );
    }
    if old.scroll_edge != new.scroll_edge
        && let Some(edge) = new.scroll_edge.or(old.scroll_edge)
    {
        let depth = new.scroll_edge.map_or(0, |edge| edge.depth);
        style.set_scroll_edge(surface, edge.edge, edge.style, depth);
    }
}

impl Drop for ChromeSurface {
    fn drop(&mut self) {
        // The swapchain must go before the wl_surface (vkCreateWaylandSurfaceKHR).
        self.graphics = None;
        if let Some(viewport) = self.viewport.take() {
            viewport.destroy();
        }
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

impl SurfaceWindow {
    pub(crate) fn sync_chromes(&mut self) {
        let Some(host) = self.chrome_host.clone() else {
            return;
        };
        let scale = self.scale;
        let scaled_by_viewport = self.fractional.is_some() && self.viewport.is_some();
        let declared = &self.app.chromes;
        self.chromes
            .retain(|open| declared.iter().any(|chrome| chrome.root == open.root));
        for chrome in declared {
            let index = match self
                .chromes
                .iter()
                .position(|open| open.root == chrome.root)
            {
                Some(index) => index,
                None => {
                    tracing::debug!(
                        target: "bk::chrome",
                        placement = ?chrome.placement(scale as f32),
                        effects = chrome.effects.len(),
                        "chrome surface opened"
                    );
                    self.chromes.push(ChromeSurface::open(
                        &host,
                        &self.surface,
                        chrome,
                        scaled_by_viewport,
                    ));
                    self.chromes.len() - 1
                }
            };
            if !self.chromes[index].update(&host, &self.shared, chrome, scale) {
                self.chromes.remove(index);
            }
        }
    }
}

impl Runtime {
    pub(crate) fn chrome_host(&self, role: &SurfaceRole) -> Option<ChromeHost> {
        matches!(role, SurfaceRole::Window(_)).then_some(())?;
        Some(ChromeHost {
            subcompositor: self.subcompositor.clone()?,
            style: self
                .window_style
                .clone()
                .filter(crate::platform::linux::window_style::offers_backdrop)?,
            viewporter: self.viewporter.clone(),
            queue: self.qh.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use sctk::reexports::client::Proxy;

    use super::super::fake_server::{Probe, Seen};
    use super::*;
    use crate::platform::backdrop::{Backdrop, ScrollEdge};
    use crate::platform::window_style::{Edge, ScrollEdgeEffectStyle, WireMaterial};

    fn menu() -> Requests {
        Requests {
            backdrop: Backdrop {
                material: Some(WireMaterial::Menu),
                radius: 6,
                rects: vec![(1, 2, 30, 40)],
            },
            scroll_edge: None,
            glass_spacing: 0,
        }
    }

    fn top() -> Requests {
        Requests {
            backdrop: Backdrop::default(),
            scroll_edge: Some(ScrollEdge {
                edge: Edge::Top,
                style: ScrollEdgeEffectStyle::Soft,
                depth: 96,
            }),
            glass_spacing: 0,
        }
    }

    fn sent(old: &Requests, new: &Requests) -> Vec<Seen> {
        let (mut probe, mut queue, server) = Probe::connect(Some(10));
        let root = probe.surface();
        let (_subsurface, chrome) = probe
            .subcompositor
            .create_subsurface(root.clone(), &probe.queue);
        let style = probe.style.clone().unwrap();
        send_requests(&style, &chrome, old, new);
        queue.roundtrip(&mut probe).unwrap();
        let (root, chrome) = (root.id().protocol_id(), chrome.id().protocol_id());
        let mut seen = server.seen();
        assert_eq!(
            seen.remove(0),
            Seen::Subsurface {
                child: chrome,
                parent: root
            }
        );
        seen
    }

    #[test]
    fn a_material_goes_to_the_sub_surface_within_the_window() {
        let seen = sent(&Requests::default(), &menu());
        assert!(matches!(
            seen.as_slice(),
            [Seen::Backdrop { blending: 1, material: 2, radius: 6, rects, .. }] if rects == &[1, 2, 30, 40]
        ));
    }

    #[test]
    fn a_scroll_edge_goes_to_the_sub_surface_with_its_depth() {
        let seen = sent(&Requests::default(), &top());
        assert!(matches!(
            seen.as_slice(),
            [Seen::ScrollEdge {
                edge: 0,
                style: 1,
                depth: 96,
                ..
            }]
        ));
    }

    #[test]
    fn an_unchanged_declaration_sends_nothing() {
        assert!(sent(&menu(), &menu()).is_empty());
        assert!(sent(&top(), &top()).is_empty());
    }

    #[test]
    fn only_the_part_that_changed_is_sent() {
        let both = Requests {
            scroll_edge: top().scroll_edge,
            ..menu()
        };
        let seen = sent(&both, &menu());
        assert!(matches!(
            seen.as_slice(),
            [Seen::ScrollEdge {
                edge: 0,
                style: 1,
                depth: 0,
                ..
            }]
        ));
    }

    #[test]
    fn a_declaration_that_goes_away_is_removed_with_no_rectangles_and_no_depth() {
        let seen = sent(&menu(), &Requests::default());
        assert!(matches!(
            seen.as_slice(),
            [Seen::Backdrop { material: 2, radius: 0, rects, .. }] if rects.is_empty()
        ));
        assert!(matches!(
            sent(&top(), &Requests::default()).as_slice(),
            [Seen::ScrollEdge { depth: 0, .. }]
        ));
    }
}
