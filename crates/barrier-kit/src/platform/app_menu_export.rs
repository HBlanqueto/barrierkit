use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle, backend::ObjectId};
use wayland_protocols_plasma::appmenu::client::{
    org_kde_kwin_appmenu::{self, OrgKdeKwinAppmenu},
    org_kde_kwin_appmenu_manager::{self, OrgKdeKwinAppmenuManager},
};

use super::runtime::Runtime;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AppMenuAddress {
    pub service: String,
    pub path: String,
}

impl Runtime {
    pub(super) fn process_pending_menu_addresses(&mut self) {
        let pending = {
            let mut guard = self
                .pending_menu_addresses
                .lock()
                .expect("pending menu addresses poisoned");
            std::mem::take(&mut *guard)
        };
        for (surface_id, address) in pending {
            let Some(index) = self
                .surfaces
                .iter()
                .position(|s| s.surface.id() == surface_id)
            else {
                continue;
            };
            if self.surfaces[index].appmenu.is_none()
                && let Some(manager) = &self.appmenu_manager
            {
                self.surfaces[index].appmenu = Some(manager.create(
                    &self.surfaces[index].surface,
                    &self.qh,
                    self.surfaces[index].surface.id(),
                ));
            }
            if let Some(appmenu) = &self.surfaces[index].appmenu {
                appmenu.set_address(address.service.clone(), address.path.clone());
            }
            self.surfaces[index].appmenu_address = Some(address);
        }
    }

    pub(super) fn process_menu_actions(&mut self) {
        for surface in &mut self.surfaces {
            let action_ids = surface
                .menu_handle
                .as_ref()
                .map(|handle| handle.drain_actions())
                .unwrap_or_default();
            if let Some(on_action) = surface.app.on_menu_action.as_ref() {
                for id in action_ids {
                    on_action(&id);
                }
            }
        }
    }
}

impl Dispatch<OrgKdeKwinAppmenuManager, (), Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &OrgKdeKwinAppmenuManager,
        _: org_kde_kwin_appmenu_manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}

impl Dispatch<OrgKdeKwinAppmenu, ObjectId, Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &OrgKdeKwinAppmenu,
        _: org_kde_kwin_appmenu::Event,
        _: &ObjectId,
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}
