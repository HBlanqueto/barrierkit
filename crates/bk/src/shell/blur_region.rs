use sctk::compositor::Region;
use sctk::reexports::client::backend::ObjectId;
use sctk::reexports::client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_manager_v1::{
    self, ExtBackgroundEffectManagerV1,
};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_surface_v1::{
    self, ExtBackgroundEffectSurfaceV1,
};

use crate::platform::runtime::Runtime;
use crate::shell::ShellState;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlurRegion {
    pub rects: Vec<(i32, i32, i32, i32)>,
}

pub fn set_blur_region(region: BlurRegion) {
    crate::platform::KitSender::get().send(region);
}

impl Runtime {
    pub(crate) fn apply_blur_region(&mut self, surface_id: &ObjectId, region: BlurRegion) {
        let Some(index) = self.surface_index_by_id(surface_id) else {
            return;
        };
        let surface = self.surfaces[index].surface.clone();
        let wl_region = Region::new(&self.compositor).ok();
        if let Some(wl_region) = &wl_region {
            for &(x, y, width, height) in &region.rects {
                wl_region.add(x, y, width, height);
            }
        }
        let qh = self.qh.clone();
        let Some(shell) = self.extension_mut::<ShellState>() else {
            return;
        };
        let Some(manager) = shell.background_effect.as_ref() else {
            return;
        };
        let effect = shell
            .background_effects
            .entry(surface_id.clone())
            .or_insert_with(|| manager.get_background_effect(&surface, &qh, ()));
        effect.set_blur_region(
            wl_region
                .as_ref()
                .filter(|_| !region.rects.is_empty())
                .map(Region::wl_region),
        );
        surface.commit();
    }
}

impl Dispatch<ExtBackgroundEffectManagerV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtBackgroundEffectManagerV1,
        _: ext_background_effect_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtBackgroundEffectSurfaceV1, ()> for Runtime {
    fn event(
        _: &mut Self,
        _: &ExtBackgroundEffectSurfaceV1,
        _: ext_background_effect_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
