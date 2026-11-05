use freya_core::prelude::{State, WritableUtils};
use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
use sctk::reexports::{
    client::{
        Connection, Dispatch, Proxy, QueueHandle, event_created_child, protocol::wl_seat::WlSeat,
    },
    protocols::ext::foreign_toplevel_list::v1::client::{
        ext_foreign_toplevel_handle_v1::{Event as FtlHandleEvent, ExtForeignToplevelHandleV1},
        ext_foreign_toplevel_list_v1::{Event as FtlListEvent, ExtForeignToplevelListV1},
    },
    protocols_wlr::foreign_toplevel::v1::client::{
        zwlr_foreign_toplevel_handle_v1::{
            Event as ZwlrForeignToplevelHandleV1Event, State as ZwlrForeignToplevelHandleV1State,
            ZwlrForeignToplevelHandleV1,
        },
        zwlr_foreign_toplevel_manager_v1::{Event as MgmtListEvent, ZwlrForeignToplevelManagerV1},
    },
};
use std::{cell::RefCell, rc::Rc};

use crate::platform::{AppMenuAddress, runtime::Runtime};
use crate::shell::InfinityWindowManagerV1;
use crate::shell::{GtkMenusAddress, ShellState};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToplevelInfo {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub active: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub restorable: bool,
    pub app_menu: Option<AppMenuAddress>,
    pub gtk_menus: Option<GtkMenusAddress>,
}

pub(super) enum InfoEvent {
    Title(String),
    AppId(String),
    State {
        active: bool,
        maximized: bool,
        minimized: bool,
    },
    AppMenu(Option<AppMenuAddress>),
    GtkMenus(Option<GtkMenusAddress>),
}

impl ToplevelInfo {
    fn apply(&mut self, event: InfoEvent) {
        match event {
            InfoEvent::Title(title) => self.title = title,
            InfoEvent::AppId(app_id) => self.app_id = app_id,
            InfoEvent::State {
                active,
                maximized,
                minimized,
            } => {
                self.active = active;
                self.maximized = maximized;
                self.minimized = minimized;
            }
            InfoEvent::AppMenu(app_menu) => self.app_menu = app_menu,
            InfoEvent::GtkMenus(gtk_menus) => self.gtk_menus = gtk_menus,
        }
    }
}

pub(crate) struct ToplevelEntry {
    pub(crate) list: Option<ExtForeignToplevelHandleV1>,
    pub(crate) mgmt: Option<ZwlrForeignToplevelHandleV1>,
    pub(crate) info: ToplevelInfo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusBar {
    #[default]
    None,
    Hidden,
    Revealed,
}

impl FocusBar {
    pub fn over_focus(self) -> bool {
        self != FocusBar::None
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpaceInfo {
    pub index: u32,
    pub count: u32,
    pub desktop: Option<u32>,
    pub focus_bar: FocusBar,
}

#[derive(Clone)]
pub struct Toplevels {
    pub list: State<Vec<ToplevelInfo>>,
    pub space: State<SpaceInfo>,
    pub(super) entries: Rc<RefCell<Vec<ToplevelEntry>>>,
    pub(super) seat: Rc<RefCell<Option<WlSeat>>>,
    pub(super) infinity: Option<InfinityWindowManagerV1>,
}

impl Toplevels {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    fn with_handle(&self, id: u64, action: impl FnOnce(&ZwlrForeignToplevelHandleV1)) {
        if let Some(mgmt) = self
            .entries
            .borrow()
            .iter()
            .find(|entry| entry.info.id == id)
            .and_then(|entry| entry.mgmt.as_ref())
        {
            action(mgmt);
        }
    }

    fn with_infinity(
        &self,
        id: u64,
        action: impl FnOnce(&InfinityWindowManagerV1, &ZwlrForeignToplevelHandleV1),
    ) {
        if let Some(manager) = &self.infinity {
            self.with_handle(id, |mgmt| action(manager, mgmt));
        }
    }

    pub fn activate(&self, id: u64) {
        if let Some(seat) = self.seat.borrow().as_ref() {
            self.with_handle(id, |mgmt| mgmt.activate(seat));
        }
    }

    pub fn close(&self, id: u64) {
        self.with_handle(id, |mgmt| mgmt.close());
    }

    pub fn minimize(&self, id: u64) {
        self.with_handle(id, |mgmt| mgmt.set_minimized());
    }

    pub fn unset_minimized(&self, id: u64) {
        self.with_handle(id, |mgmt| mgmt.unset_minimized());
    }

    pub fn set_maximized(&self, id: u64, maximized: bool) {
        self.with_handle(id, |mgmt| {
            if maximized {
                mgmt.set_maximized();
            } else {
                mgmt.unset_maximized();
            }
        });
    }

    pub fn arrange(&self, id: u64, layout: InfinityLayout) {
        self.with_infinity(id, |manager, mgmt| manager.arrange(mgmt, layout));
    }

    pub fn arrange_pair(&self, layout: InfinityPairLayout) {
        if let Some(manager) = &self.infinity {
            manager.arrange_pair(layout);
        }
    }

    pub fn arrange_quarters(&self) {
        if let Some(manager) = &self.infinity {
            manager.arrange_quarters();
        }
    }

    pub fn restore(&self, id: u64) {
        self.with_infinity(id, |manager, mgmt| manager.restore(mgmt));
    }

    pub fn force_quit(&self, id: u64) {
        self.with_infinity(id, |manager, mgmt| {
            if manager.version() >= 4 {
                manager.force_quit(mgmt);
            }
        });
    }

    pub fn set_fullscreen(&self, id: u64, fullscreen: bool) {
        self.with_infinity(id, |manager, mgmt| {
            manager.set_fullscreen(mgmt, fullscreen as u32)
        });
    }

    pub(super) fn update(&self, toplevel: &ZwlrForeignToplevelHandleV1, event: InfoEvent) {
        if let Some(entry) = self
            .entries
            .borrow_mut()
            .iter_mut()
            .find(|entry| entry.mgmt.as_ref() == Some(toplevel))
        {
            entry.info.apply(event);
        }
    }
}

impl ShellState {
    pub(crate) fn sync_toplevels(&mut self) -> bool {
        let infos: Vec<ToplevelInfo> = self
            .toplevels
            .entries
            .borrow()
            .iter()
            .map(|entry| entry.info.clone())
            .collect();
        if *self.toplevels.list.peek() != infos {
            self.toplevels.list.set(infos);
            true
        } else {
            false
        }
    }
}

impl Runtime {
    pub(super) fn sync_toplevels(&mut self) {
        let changed = self
            .extension_mut::<ShellState>()
            .expect("sync_toplevels called without ShellState")
            .sync_toplevels();
        if changed {
            self.request_redraw_all = true;
            self.ping.ping();
        }
    }
}

impl ShellState {
    fn push_toplevel(
        &mut self,
        list: Option<ExtForeignToplevelHandleV1>,
        mgmt: Option<ZwlrForeignToplevelHandleV1>,
    ) {
        self.next_toplevel_id += 1;
        self.toplevels.entries.borrow_mut().push(ToplevelEntry {
            list,
            mgmt,
            info: ToplevelInfo {
                id: self.next_toplevel_id,
                ..ToplevelInfo::default()
            },
        });
    }
}

impl Dispatch<ExtForeignToplevelListV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        _: &ExtForeignToplevelListV1,
        event: <ExtForeignToplevelListV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let FtlListEvent::Toplevel { toplevel } = event {
            state
                .extension_mut::<ShellState>()
                .expect("ext_foreign_toplevel_list event without ShellState")
                .push_toplevel(Some(toplevel), None);
        }
        state.sync_toplevels();
    }

    event_created_child!(Runtime, ExtForeignToplevelListV1, [
        sctk::reexports::protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::EVT_TOPLEVEL_OPCODE => (ExtForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ExtForeignToplevelHandleV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        handle: &ExtForeignToplevelHandleV1,
        event: <ExtForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        {
            let shell = state
                .extension_mut::<ShellState>()
                .expect("ext_foreign_toplevel_handle event without ShellState");
            let mut entries = shell.toplevels.entries.borrow_mut();
            let Some(index) = entries
                .iter()
                .position(|entry| entry.list.as_ref() == Some(handle))
            else {
                return;
            };
            match event {
                FtlHandleEvent::Closed => {
                    entries.remove(index);
                    handle.destroy();
                }
                FtlHandleEvent::Title { title } => {
                    entries[index].info.apply(InfoEvent::Title(title))
                }
                FtlHandleEvent::AppId { app_id } => {
                    entries[index].info.apply(InfoEvent::AppId(app_id))
                }
                _ => {}
            }
        }
        state.sync_toplevels();
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: <ZwlrForeignToplevelManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let MgmtListEvent::Toplevel { toplevel } = event {
            state
                .extension_mut::<ShellState>()
                .expect("zwlr_foreign_toplevel_manager event without ShellState")
                .push_toplevel(None, Some(toplevel));
        }
        state.sync_toplevels();
    }

    event_created_child!(Runtime, ZwlrForeignToplevelManagerV1, [
        sctk::reexports::protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, (), Runtime> for Runtime {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: <ZwlrForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        {
            let toplevels = &state
                .extension_mut::<ShellState>()
                .expect("zwlr_foreign_toplevel_handle event without ShellState")
                .toplevels;
            match event {
                ZwlrForeignToplevelHandleV1Event::Closed => {
                    toplevels
                        .entries
                        .borrow_mut()
                        .retain(|entry| entry.mgmt.as_ref() != Some(handle));
                    handle.destroy();
                }
                ZwlrForeignToplevelHandleV1Event::Title { title } => {
                    toplevels.update(handle, InfoEvent::Title(title))
                }
                ZwlrForeignToplevelHandleV1Event::AppId { app_id } => {
                    toplevels.update(handle, InfoEvent::AppId(app_id))
                }
                ZwlrForeignToplevelHandleV1Event::State { state } => {
                    let has =
                        |flag: ZwlrForeignToplevelHandleV1State| state.contains(&(flag as u8));
                    toplevels.update(
                        handle,
                        InfoEvent::State {
                            active: has(ZwlrForeignToplevelHandleV1State::Activated),
                            maximized: has(ZwlrForeignToplevelHandleV1State::Maximized),
                            minimized: has(ZwlrForeignToplevelHandleV1State::Minimized),
                        },
                    )
                }
                _ => {}
            }
        }
        state.sync_toplevels();
    }
}
