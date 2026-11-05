use freya_core::prelude::WritableUtils;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use crate::platform::{AppMenuAddress, runtime::Runtime};
use crate::shell::{FocusBar, GtkMenusAddress, ShellState, SpaceInfo, toplevels::InfoEvent};

pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    InfinityWindowManagerV1, Layout, PairLayout,
};

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
        let mut space_changed = false;
        {
            let shell = state
                .extension_mut::<ShellState>()
                .expect("infinity_window_manager event without ShellState");
            match event {
                Event::Restorable { handle, restorable } => {
                    if let Some(entry) = shell
                        .toplevels
                        .entries
                        .borrow_mut()
                        .iter_mut()
                        .find(|entry| entry.mgmt.as_ref() == Some(&handle))
                    {
                        entry.info.restorable = restorable != 0;
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
                    shell
                        .toplevels
                        .update(&toplevel, InfoEvent::AppMenu(address));
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
                    shell
                        .toplevels
                        .update(&toplevel, InfoEvent::GtkMenus(address));
                }
                Event::Space {
                    index,
                    count,
                    desktop,
                    focus_bar,
                } => {
                    let focus_bar = match focus_bar.into_result() {
                        Ok(protocol::infinity_window_manager_v1::FocusBar::Hidden) => {
                            FocusBar::Hidden
                        }
                        Ok(protocol::infinity_window_manager_v1::FocusBar::Revealed) => {
                            FocusBar::Revealed
                        }
                        _ => FocusBar::None,
                    };
                    let info = SpaceInfo {
                        index,
                        count,
                        desktop: (desktop > 0).then_some(desktop),
                        focus_bar,
                    };
                    if *shell.toplevels.space.peek() != info {
                        shell.toplevels.space.set(info);
                        space_changed = true;
                    }
                }
                _ => {}
            }
        }
        if space_changed {
            state.request_redraw_all = true;
            state.ping.ping();
        }
        state.sync_toplevels();
    }
}
