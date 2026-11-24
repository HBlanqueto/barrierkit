use freya_core::prelude::WritableUtils;
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle};

use bk::{extension::Runtime, platform::MainMenuAddress};

use crate::{FocusBar, GtkMenusAddress, Shell, ShellState, toplevels::sync_toplevels};

use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    Event, FocusBar as WireFocusBar,
};
pub use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    InfinityWindowManagerV1, Layout, PairLayout,
};

impl Dispatch<InfinityWindowManagerV1, Shell> for Runtime {
    fn event(
        state: &mut Self,
        _: &InfinityWindowManagerV1,
        event: <InfinityWindowManagerV1 as Proxy>::Event,
        _: &Shell,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
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
                    let address = (!unique_bus_name.is_empty()
                        && !(menubar_path.is_empty() && app_menu_path.is_empty()))
                    .then_some(GtkMenusAddress {
                        unique_bus_name,
                        application_id,
                        menubar_path,
                        app_menu_path,
                        application_object_path,
                        window_object_path,
                    });
                    shell
                        .toplevels
                        .update(&toplevel, |info| info.gtk_menus = address);
                }
                Event::Space { focus_bar } => {
                    let focus_bar = match focus_bar.into_result() {
                        Ok(WireFocusBar::Hidden) => FocusBar::Hidden,
                        Ok(WireFocusBar::Revealed) => FocusBar::Revealed,
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
        sync_toplevels(state);
    }
}
