use freya_core::prelude::{State, WritableUtils};
use infinity_protocol::client::infinity_window_manager_v1::infinity_window_manager_v1::{
    Layout as InfinityLayout, PairLayout as InfinityPairLayout,
};
use sctk::reexports::{
    client::{Connection, Dispatch, Proxy, QueueHandle, event_created_child},
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

use super::{app_menu_export::AppMenuAddress, foreign_menus::GtkMenusAddress, runtime::Runtime};

/// One running toplevel, as the dock lists it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToplevelInfo {
    pub title: String,
    pub app_id: String,
    pub active: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub restorable: bool,
    /// The app's dbusmenu address, when it announced one.
    pub app_menu: Option<AppMenuAddress>,
    /// The GTK 3/4 D-Bus menu address, when announced via `gtk_shell1`.
    pub gtk_menus: Option<GtkMenusAddress>,
}

/// A change that only touches one toplevel's metadata. Kept pure so the
/// interleaved-event ordering can be tested without Wayland.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    Closed,
}

pub(super) fn apply_info_event(infos: &mut Vec<ToplevelInfo>, index: usize, event: InfoEvent) {
    let Some(info) = infos.get_mut(index) else {
        return;
    };
    match event {
        InfoEvent::Title(title) => info.title = title,
        InfoEvent::AppId(app_id) => info.app_id = app_id,
        InfoEvent::State {
            active,
            maximized,
            minimized,
        } => {
            info.active = active;
            info.maximized = maximized;
            info.minimized = minimized;
        }
        InfoEvent::AppMenu(app_menu) => info.app_menu = app_menu,
        InfoEvent::GtkMenus(gtk_menus) => info.gtk_menus = gtk_menus,
        InfoEvent::Closed => {
            infos.remove(index);
        }
    }
}

#[derive(Clone)]
pub(super) struct ToplevelEntry {
    pub(super) list: Option<ExtForeignToplevelHandleV1>,
    pub(super) mgmt: Option<ZwlrForeignToplevelHandleV1>,
    pub(super) restorable: bool,
    pub(super) info: ToplevelInfo,
}

/// The running toplevels, provided as a root context. The dock reads
/// [`Toplevels::list`] and calls [`Toplevels::activate`] on a click.
type Activate = Rc<RefCell<Option<Rc<dyn Fn(usize)>>>>;
type Maximize = Rc<RefCell<Option<Rc<dyn Fn(usize, bool)>>>>;
type Arrange = Rc<RefCell<Option<Rc<dyn Fn(usize, InfinityLayout)>>>>;
type ArrangePair = Rc<RefCell<Option<Rc<dyn Fn(InfinityPairLayout)>>>>;
type ArrangeQuarters = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

#[derive(Clone)]
pub struct Toplevels {
    pub(super) list: State<Vec<ToplevelInfo>>,
    pub(super) activate: Activate,
    pub(super) close: Activate,
    pub(super) minimize: Activate,
    pub(super) maximize: Maximize,
    pub(super) unminimize: Activate,
    pub(super) arrange: Arrange,
    pub(super) arrange_pair: ArrangePair,
    pub(super) arrange_quarters: ArrangeQuarters,
    pub(super) restore: Activate,
    pub(super) set_fullscreen: Maximize,
}

impl Toplevels {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn activate(&self, index: usize) {
        if let Some(f) = self.activate.borrow().as_ref() {
            f(index);
        }
    }

    pub fn close(&self, index: usize) {
        if let Some(f) = self.close.borrow().as_ref() {
            f(index);
        }
    }

    pub fn minimize(&self, index: usize) {
        if let Some(f) = self.minimize.borrow().as_ref() {
            f(index);
        }
    }

    pub fn unset_minimized(&self, index: usize) {
        if let Some(f) = self.unminimize.borrow().as_ref() {
            f(index);
        }
    }

    pub fn set_maximized(&self, index: usize, maximized: bool) {
        if let Some(f) = self.maximize.borrow().as_ref() {
            f(index, maximized);
        }
    }

    /// Minimizes every toplevel in the list.
    pub fn minimize_all(&self) {
        let len = self.list.read().len();
        for index in 0..len {
            self.minimize(index);
        }
    }

    /// Arranges the toplevel at `index` in the requested Infinity layout.
    pub fn arrange(&self, index: usize, layout: InfinityLayout) {
        if let Some(f) = self.arrange.borrow().as_ref() {
            f(index, layout);
        }
    }

    /// Tiles the active and previous toplevels as a pair.
    pub fn arrange_pair(&self, layout: InfinityPairLayout) {
        if let Some(f) = self.arrange_pair.borrow().as_ref() {
            f(layout);
        }
    }

    /// Tiles the four most recent toplevels in quarters.
    pub fn arrange_quarters(&self) {
        if let Some(f) = self.arrange_quarters.borrow().as_ref() {
            f();
        }
    }

    /// Restores the toplevel at `index` to its pre-arrange rectangle.
    pub fn restore(&self, index: usize) {
        if let Some(f) = self.restore.borrow().as_ref() {
            f(index);
        }
    }

    /// Sets or unsets fullscreen on the toplevel at `index`.
    pub fn set_fullscreen(&self, index: usize, fullscreen: bool) {
        if let Some(f) = self.set_fullscreen.borrow().as_ref() {
            f(index, fullscreen);
        }
    }
}

impl Runtime {
    pub(super) fn sync_toplevels(&mut self) {
        let infos: Vec<ToplevelInfo> = self
            .toplevel_entries
            .borrow()
            .iter()
            .map(|entry| {
                let mut info = entry.info.clone();
                info.restorable = entry.restorable;
                info
            })
            .collect();
        let mut list = self.toplevels.list;
        if *list.peek() != infos {
            list.set(infos);
            self.request_redraw_all = true;
            self.ping.ping();
        }
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
        if let FtlListEvent::Toplevel { toplevel } = event
            && state._foreign_mgmt.is_none()
        {
            state.toplevel_entries.borrow_mut().push(ToplevelEntry {
                list: Some(toplevel),
                mgmt: None,
                restorable: false,
                info: ToplevelInfo::default(),
            });
            state.sync_toplevels();
        }
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
        if state._foreign_mgmt.is_some() {
            return;
        }
        if matches!(event, FtlHandleEvent::Closed) {
            state.toplevel_entries.borrow_mut().retain(|entry| {
                entry
                    .list
                    .as_ref()
                    .is_none_or(|list| list.id() != handle.id())
            });
            state.sync_toplevels();
            return;
        }
        let mut entries = state.toplevel_entries.borrow_mut();
        let index = entries
            .iter()
            .position(|e| e.list.as_ref().is_some_and(|list| list.id() == handle.id()));
        if let Some(index) = index {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            let info_event = match event {
                FtlHandleEvent::Title { title } => Some(InfoEvent::Title(title)),
                FtlHandleEvent::AppId { app_id } => Some(InfoEvent::AppId(app_id)),
                _ => None,
            };
            if let Some(info_event) = info_event {
                apply_info_event(&mut infos, index, info_event);
                entries[index].info = infos[index].clone();
            }
        }
        drop(entries);
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
            state.toplevel_entries.borrow_mut().push(ToplevelEntry {
                list: None,
                mgmt: Some(toplevel),
                restorable: false,
                info: ToplevelInfo::default(),
            });
            state.sync_toplevels();
        }
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
        let mut entries = state.toplevel_entries.borrow_mut();
        let index = entries
            .iter()
            .position(|e| e.mgmt.as_ref().is_some_and(|mgmt| mgmt.id() == handle.id()));
        let Some(index) = index else {
            return;
        };
        if matches!(event, ZwlrForeignToplevelHandleV1Event::Closed) {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            apply_info_event(&mut infos, index, InfoEvent::Closed);
            entries.remove(index);
            drop(entries);
            state.sync_toplevels();
            return;
        }
        let info_event = match event {
            ZwlrForeignToplevelHandleV1Event::Title { title } => Some(InfoEvent::Title(title)),
            ZwlrForeignToplevelHandleV1Event::AppId { app_id } => Some(InfoEvent::AppId(app_id)),
            ZwlrForeignToplevelHandleV1Event::State { state } => Some(InfoEvent::State {
                active: state.contains(&(ZwlrForeignToplevelHandleV1State::Activated as u8)),
                maximized: state.contains(&(ZwlrForeignToplevelHandleV1State::Maximized as u8)),
                minimized: state.contains(&(ZwlrForeignToplevelHandleV1State::Minimized as u8)),
            }),
            _ => None,
        };
        if let Some(info_event) = info_event {
            let mut infos: Vec<ToplevelInfo> = entries.iter().map(|e| e.info.clone()).collect();
            apply_info_event(&mut infos, index, info_event);
            entries[index].info = infos[index].clone();
        }
        drop(entries);
        state.sync_toplevels();
    }
}

pub(super) fn update_toplevel_info(
    entries: &mut [ToplevelEntry],
    toplevel: &ZwlrForeignToplevelHandleV1,
    event: InfoEvent,
) {
    if let Some(index) = entries.iter().position(|entry| {
        entry
            .mgmt
            .as_ref()
            .is_some_and(|mgmt| mgmt.id() == toplevel.id())
    }) {
        let mut infos: Vec<ToplevelInfo> = entries.iter().map(|entry| entry.info.clone()).collect();
        apply_info_event(&mut infos, index, event);
        entries[index].info = infos[index].clone();
    }
}
