use sctk::reexports::client::{Connection, Dispatch, QueueHandle, backend::ObjectId};
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
        let pending = std::mem::take(
            &mut *self
                .pending_menu_addresses
                .lock()
                .expect("pending menu addresses poisoned"),
        );
        for (surface_id, address) in pending {
            let Some(index) = self.surface_index_by_id(&surface_id) else {
                continue;
            };
            let window = &mut self.surfaces[index];
            if window.appmenu.is_none()
                && let Some(manager) = &self.appmenu_manager
            {
                window.appmenu = Some(manager.create(&window.surface, &self.qh, surface_id));
            }
            if let Some(appmenu) = &window.appmenu {
                appmenu.set_address(address.service.clone(), address.path.clone());
            }
            window.appmenu_address = Some(address);
        }
    }

    pub(super) fn process_menu_actions(&mut self) {
        for surface in &mut self.surfaces {
            let action_ids: Vec<String> = surface
                .menu_handle
                .iter()
                .chain(&surface.status_handle)
                .flat_map(|handle| handle.drain_actions())
                .collect();
            for id in action_ids {
                surface.app.menu_actions.dispatch(&id);
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
