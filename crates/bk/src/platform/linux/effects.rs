use sctk::reexports::client::{
    Connection, Dispatch, Proxy, QueueHandle, globals::GlobalList, protocol::wl_surface::WlSurface,
};

use crate::platform::{
    backdrop::Requests,
    linux::{glass::packed, window_style::wire},
    runtime::Runtime,
    window_style::WireMaterial,
};

pub(crate) use limitless::client::effects::limitless_effects_v1::{
    limitless_effects_v1::LimitlessEffectsV1,
    limitless_glass_container_v1::LimitlessGlassContainerV1,
    limitless_glass_effect_v1::{LimitlessGlassEffectV1, Style as GlassStyle},
    limitless_scroll_edge_effect_v1::LimitlessScrollEdgeEffectV1,
    limitless_visual_effect_v1::{BlendingMode, LimitlessVisualEffectV1, Material},
};

macro_rules! no_events {
    ($($proxy:ty),+) => {$(
        impl Dispatch<$proxy, ()> for Runtime {
            fn event(
                _: &mut Self,
                _: &$proxy,
                _: <$proxy as Proxy>::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }
    )+};
}

no_events!(
    LimitlessEffectsV1,
    LimitlessVisualEffectV1,
    LimitlessGlassEffectV1,
    LimitlessGlassContainerV1,
    LimitlessScrollEdgeEffectV1
);

#[derive(Clone)]
pub(crate) struct Effects {
    manager: LimitlessEffectsV1,
    pub(crate) queue: QueueHandle<Runtime>,
}

impl Effects {
    pub(crate) fn bind(globals: &GlobalList, queue: &QueueHandle<Runtime>) -> Option<Self> {
        let manager = globals
            .bind::<LimitlessEffectsV1, _, _>(queue, 1..=1, ())
            .ok()?;
        Some(Self {
            manager,
            queue: queue.clone(),
        })
    }

    pub(crate) fn visual(&self, surface: &WlSurface) -> LimitlessVisualEffectV1 {
        self.manager.create_visual_effect(surface, &self.queue, ())
    }
}

enum Kind {
    Visual(LimitlessVisualEffectV1),
    Glass(LimitlessGlassEffectV1, Option<LimitlessGlassContainerV1>),
}

struct Backdrop {
    key: (WireMaterial, u32),
    kind: Kind,
}

impl Backdrop {
    fn open(effects: &Effects, surface: &WlSurface, key: (WireMaterial, u32)) -> Self {
        let (material, spacing) = key;
        let glass = match material {
            WireMaterial::GlassRegular => Some(GlassStyle::Regular),
            WireMaterial::GlassClear => Some(GlassStyle::Clear),
            _ => None,
        };
        let kind = match glass {
            Some(style) => {
                let glass = effects
                    .manager
                    .create_glass_effect(surface, &effects.queue, ());
                glass.set_style(style);
                let container = (spacing > 0).then(|| {
                    let container = effects.manager.create_glass_container(&effects.queue, ());
                    container.set_spacing(spacing);
                    glass.set_container(Some(&container));
                    container
                });
                Kind::Glass(glass, container)
            }
            None => {
                let visual = effects.visual(surface);
                visual.set_material(wire(material as u32));
                visual.set_blending_mode(BlendingMode::WithinWindow);
                Kind::Visual(visual)
            }
        };
        Self { key, kind }
    }

    fn set_shape(&self, radius: u32, rects: Vec<u8>) {
        match &self.kind {
            Kind::Visual(visual) => visual.set_shape(radius, rects),
            Kind::Glass(glass, _) => glass.set_shape(radius, rects),
        }
    }

    fn destroy(self) {
        match self.kind {
            Kind::Visual(visual) => visual.destroy(),
            Kind::Glass(glass, container) => {
                glass.destroy();
                if let Some(container) = container {
                    container.destroy();
                }
            }
        }
    }
}

// The objects one chrome surface holds for what its Requests ask: one backdrop and one scroll edge.
#[derive(Default)]
pub(crate) struct Applied {
    backdrop: Option<Backdrop>,
    scroll_edge: Option<LimitlessScrollEdgeEffectV1>,
}

impl Applied {
    // The requests go out before the buffer they describe, so both land in the parent's next commit.
    pub(crate) fn send(
        &mut self,
        effects: &Effects,
        surface: &WlSurface,
        old: &Requests,
        new: &Requests,
    ) {
        if old.backdrop != new.backdrop || old.glass_spacing != new.glass_spacing {
            self.send_backdrop(effects, surface, new);
        }
        if old.scroll_edge != new.scroll_edge {
            self.send_scroll_edge(effects, surface, new);
        }
    }

    fn send_backdrop(&mut self, effects: &Effects, surface: &WlSurface, new: &Requests) {
        let Some(material) = new.backdrop.material else {
            if let Some(open) = self.backdrop.take() {
                open.destroy();
            }
            return;
        };
        let glass = matches!(
            material,
            WireMaterial::GlassRegular | WireMaterial::GlassClear
        );
        let key = (material, if glass { new.glass_spacing } else { 0 });
        if self.backdrop.as_ref().is_none_or(|open| open.key != key) {
            if let Some(open) = self.backdrop.take() {
                open.destroy();
            }
            self.backdrop = Some(Backdrop::open(effects, surface, key));
        }
        if let Some(open) = &self.backdrop {
            open.set_shape(new.backdrop.radius, packed(&new.backdrop.rects));
        }
    }

    fn send_scroll_edge(&mut self, effects: &Effects, surface: &WlSurface, new: &Requests) {
        let Some(edge) = new.scroll_edge else {
            if let Some(open) = self.scroll_edge.take() {
                open.destroy();
            }
            return;
        };
        let open = self.scroll_edge.get_or_insert_with(|| {
            effects
                .manager
                .create_scroll_edge_effect(surface, &effects.queue, ())
        });
        open.set_edge(wire(edge.edge as u32));
        open.set_style(wire(edge.style as u32));
        open.set_depth(edge.depth);
    }

    pub(crate) fn destroy(&mut self) {
        if let Some(open) = self.backdrop.take() {
            open.destroy();
        }
        if let Some(open) = self.scroll_edge.take() {
            open.destroy();
        }
    }
}
