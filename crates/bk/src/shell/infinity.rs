use freya_core::prelude::WritableUtils;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use crate::platform::{MainMenuAddress, runtime::Runtime};
use crate::shell::{FocusBar, GtkMenusAddress, ShellState};

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
                Event::Restorable { handle, restorable } => shell
                    .toplevels
                    .update(&handle, |info| info.restorable = restorable != 0),
                Event::AppMenu {
                    toplevel,
                    service,
                    path,
                } => {
                    let address = if service.is_empty() || path.is_empty() {
                        None
                    } else {
                        Some(MainMenuAddress { service, path })
                    };
                    shell
                        .toplevels
                        .update(&toplevel, |info| info.main_menu = address);
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
                        .update(&toplevel, |info| info.gtk_menus = address);
                }
                Event::Space { focus_bar } => {
                    let focus_bar = match focus_bar.into_result() {
                        Ok(protocol::infinity_window_manager_v1::FocusBar::Hidden) => {
                            FocusBar::Hidden
                        }
                        Ok(protocol::infinity_window_manager_v1::FocusBar::Revealed) => {
                            FocusBar::Revealed
                        }
                        _ => FocusBar::None,
                    };
                    if *shell.toplevels.focus_bar.peek() != focus_bar {
                        shell.toplevels.focus_bar.set(focus_bar);
                        space_changed = true;
                    }
                }
                Event::Desktop { desktop } => {
                    let desktop = (desktop > 0).then_some(desktop);
                    if *shell.toplevels.desktop.peek() != desktop {
                        shell.toplevels.desktop.set(desktop);
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
