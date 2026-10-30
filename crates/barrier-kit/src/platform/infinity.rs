//! Client-side Infinity protocol bindings (ADR 0039).
//!
//! The shell uses this private protocol to ask the compositor for window
//! geometry: halves, quarters, pairs, fill, center, restore and fullscreen.

use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use super::{
    app_menu_export::AppMenuAddress,
    foreign_menus::GtkMenusAddress,
    runtime::Runtime,
    shell_state::ShellState,
    toplevels::{InfoEvent, update_toplevel_info},
};

pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    InfinityWindowManagerV1, Layout, PairLayout,
};

/// Re-export of the generated protocol module, matching the old layout.
pub mod protocol {
    pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1;
}

impl Dispatch<InfinityWindowManagerV1, ()> for Runtime {
    fn event(
        state: &mut Self,
        _: &InfinityWindowManagerV1,
        event: <InfinityWindowManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use self::protocol::infinity_window_manager_v1::Event;
        {
            let shell = state
                .extension_mut::<ShellState>()
                .expect("infinity_window_manager event without ShellState");
            match event {
                Event::Restorable { handle, restorable } => {
                    let restorable = restorable != 0;
                    for entry in shell.toplevel_entries.borrow_mut().iter_mut() {
                        if entry
                            .mgmt
                            .as_ref()
                            .is_some_and(|mgmt| mgmt.id() == handle.id())
                        {
                            entry.restorable = restorable;
                            break;
                        }
                    }
                }
                Event::AppMenu {
                    toplevel,
                    service,
                    path,
                } => {
                    let address = if service.is_empty() || path.is_empty() {
                        None
                    } else {
                        Some(AppMenuAddress { service, path })
                    };
                    update_toplevel_info(
                        &mut shell.toplevel_entries.borrow_mut(),
                        &toplevel,
                        InfoEvent::AppMenu(address),
                    );
                }
                Event::GtkMenus {
                    toplevel,
                    unique_bus_name,
                    application_id,
                    menubar_path,
                    app_menu_path,
                    application_object_path,
                    window_object_path,
                } => {
                    let address = GtkMenusAddress::new(
                        unique_bus_name,
                        application_id,
                        menubar_path,
                        app_menu_path,
                        application_object_path,
                        window_object_path,
                    );
                    update_toplevel_info(
                        &mut shell.toplevel_entries.borrow_mut(),
                        &toplevel,
                        InfoEvent::GtkMenus(address),
                    );
                }
                // shortcut: future Infinity events are ignored; add handlers when
                // the compositor starts sending them.
                _ => {}
            }
        }
        state.sync_toplevels();
    }
}
