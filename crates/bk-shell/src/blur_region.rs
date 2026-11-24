use sctk::compositor::Region;
use sctk::reexports::client::backend::ObjectId;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_manager_v1::{
    self, ExtBackgroundEffectManagerV1,
};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_surface_v1::{
    self, ExtBackgroundEffectSurfaceV1,
};

use bk::extension::{KitSender, Runtime};

use crate::{Shell, ShellState};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlurRegion {
    pub rects: Vec<(i32, i32, i32, i32)>,
}

pub fn set_blur_region(region: BlurRegion) {
    KitSender::get().send(region);
}

pub(crate) fn apply_blur_region(rt: &mut Runtime, surface_id: &ObjectId, region: BlurRegion) {
    let Some(index) = rt.surface_index_by_id(surface_id) else {
        return;
    };
    let surface = rt.surfaces[index].surface.clone();
    let wl_region = Region::new(&rt.compositor).ok();
    if let Some(wl_region) = &wl_region {
        for &(x, y, width, height) in &region.rects {
            wl_region.add(x, y, width, height);
        }
    }
    let qh = rt.qh.clone();
    let live: Vec<ObjectId> = rt.surfaces.iter().map(|s| s.surface.id()).collect();
    let Some(shell) = rt.extension_mut::<ShellState>() else {
        return;
    };
    let Some(manager) = shell.background_effect.as_ref() else {
        return;
    };
    shell.background_effects.retain(|id, effect| {
        let alive = live.contains(id);
        if !alive {
            effect.destroy();
        }
        alive
    });
    let effect = shell
        .background_effects
        .entry(surface_id.clone())
        .or_insert_with(|| manager.get_background_effect(&surface, &qh, Shell));
    effect.set_blur_region(
        wl_region
            .as_ref()
            .filter(|_| !region.rects.is_empty())
            .map(Region::wl_region),
    );
    surface.commit();
}

impl Dispatch<ExtBackgroundEffectManagerV1, Shell> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtBackgroundEffectManagerV1,
        _: ext_background_effect_manager_v1::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtBackgroundEffectSurfaceV1, Shell> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtBackgroundEffectSurfaceV1,
        _: ext_background_effect_surface_v1::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
