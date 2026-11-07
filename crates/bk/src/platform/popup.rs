use freya_core::integration::AppComponent;
use sctk::{
    delegate_xdg_popup,
    reexports::{
        calloop::ping::Ping,
        client::{Connection, Proxy, QueueHandle, backend::ObjectId},
        protocols::xdg::shell::client::xdg_positioner::{
            Anchor as SctkAnchorPos, ConstraintAdjustment as SctkConstraint, Gravity as SctkGravity,
        },
    },
    shell::xdg::{
        XdgPositioner, XdgShell, XdgSurface,
        popup::{Popup, PopupConfigure, PopupHandler},
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{app::AppInstance, runtime::Runtime, surface::SurfaceRole};
use crate::styles::{PopupRoot, current_theme};

#[cfg(test)]
use sctk::reexports::calloop;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PopupId(u64);

impl PopupId {
    fn next(counter: &Rc<Cell<u64>>) -> Self {
        let id = counter.get();
        counter.set(id + 1);
        Self(id)
    }

    #[cfg(test)]
    pub(crate) fn test(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupAnchor {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupGravity {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopupConstraintAdjustment {
    pub flip_x: bool,
    pub flip_y: bool,
    pub slide_x: bool,
    pub slide_y: bool,
    pub resize_y: bool,
}

#[derive(Clone)]
pub struct PopupSpec {
    pub anchor_rect: (i32, i32, u32, u32),
    pub anchor: PopupAnchor,
    pub gravity: PopupGravity,
    pub offset: (i32, i32),
    pub size: (u32, u32),
    pub constraint_adjustment: PopupConstraintAdjustment,
    pub grab: bool,
    pub on_dismissed: Option<Rc<dyn Fn()>>,
}

impl Default for PopupSpec {
    fn default() -> Self {
        Self {
            anchor_rect: (0, 0, 0, 0),
            anchor: PopupAnchor::default(),
            gravity: PopupGravity::default(),
            offset: (0, 0),
            size: (100, 100),
            constraint_adjustment: PopupConstraintAdjustment::default(),
            grab: false,
            on_dismissed: None,
        }
    }
}

#[derive(Clone)]
pub struct Popups {
    pub(crate) queue: Rc<RefCell<Vec<PopupCommand>>>,
    pub(crate) ping: Ping,
    pub(crate) next_id: Rc<Cell<u64>>,
    pub(crate) last_serial: Rc<Cell<u32>>,
    pub(crate) owner: ObjectId,
    pub(crate) current: Option<PopupId>,
    pub(crate) opens_left: Rc<Cell<bool>>,
    pub(crate) scroll_shift: Rc<Cell<f32>>,
}

impl Popups {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn open(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        self.push_open(spec, content, false)
    }

    pub(crate) fn open_popover(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        self.push_open(spec, content, true)
    }

    fn push_open(&self, spec: PopupSpec, content: AppComponent, popover: bool) -> PopupId {
        let id = PopupId::next(&self.next_id);
        let content = AppComponent::new(PopupRoot {
            theme: current_theme(),
            content,
        });
        self.queue.borrow_mut().push(PopupCommand::Open {
            owner: self.owner.clone(),
            parent: self.current,
            id,
            spec,
            content,
            serial: self.last_serial.get(),
            popover,
        });
        self.ping.ping();
        id
    }

    pub fn close(&self, id: PopupId) {
        self.queue.borrow_mut().push(PopupCommand::Close { id });
        self.ping.ping();
    }

    fn reposition(&self, id: PopupId, spec: PopupSpec) {
        self.queue
            .borrow_mut()
            .push(PopupCommand::Reposition { id, spec });
        self.ping.ping();
    }
}

#[cfg(test)]
impl Popups {
    pub(crate) fn for_test(
        owner: ObjectId,
        current: Option<PopupId>,
    ) -> (Self, Rc<RefCell<Vec<PopupCommand>>>) {
        let queue = Rc::new(RefCell::new(Vec::new()));
        let (ping, _) = calloop::ping::make_ping().expect("test ping");
        let popups = Popups {
            queue: queue.clone(),
            ping,
            next_id: Rc::new(Cell::new(0)),
            last_serial: Rc::new(Cell::new(0)),
            owner,
            current,
            opens_left: Rc::new(Cell::new(false)),
            scroll_shift: Rc::new(Cell::new(0.0)),
        };
        (popups, queue)
    }
}

#[cfg(test)]
impl PopupHandle {
    pub(crate) fn for_test(popups: Popups, id: PopupId) -> Self {
        Self { id, popups }
    }
}

#[derive(Clone)]
pub struct PopupHandle {
    pub(super) id: PopupId,
    pub(super) popups: Popups,
}

impl PopupHandle {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn id(&self) -> PopupId {
        self.id
    }

    pub fn opens_left(&self) -> bool {
        self.popups.opens_left.get()
    }

    pub fn scroll_shift(&self) -> f32 {
        self.popups.scroll_shift.get()
    }

    pub fn set_scroll_shift(&self, shift: f32) {
        self.popups.scroll_shift.set(shift);
    }

    pub fn close(&self) {
        self.popups.close(self.id);
    }

    pub fn reposition(&self, spec: PopupSpec) {
        self.popups.reposition(self.id, spec);
    }

    pub fn open_child(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        self.popups.open(spec, content)
    }
}
pub(crate) struct PopupInfo {
    id: PopupId,
    parent: Option<PopupId>,
    on_dismissed: Option<Rc<dyn Fn()>>,
    opens_left: Rc<Cell<bool>>,
    grabbed: bool,
    owner: ObjectId,
}

pub(crate) enum PopupCommand {
    Open {
        owner: ObjectId,
        parent: Option<PopupId>,
        id: PopupId,
        spec: PopupSpec,
        content: AppComponent,
        serial: u32,
        popover: bool,
    },
    Reposition {
        id: PopupId,
        spec: PopupSpec,
    },
    Close {
        id: PopupId,
    },
}
fn anchor_to_sctk(anchor: PopupAnchor) -> SctkAnchorPos {
    match anchor {
        PopupAnchor::TopLeft => SctkAnchorPos::TopLeft,
        PopupAnchor::Top => SctkAnchorPos::Top,
        PopupAnchor::TopRight => SctkAnchorPos::TopRight,
        PopupAnchor::Left => SctkAnchorPos::Left,
        PopupAnchor::Right => SctkAnchorPos::Right,
        PopupAnchor::BottomLeft => SctkAnchorPos::BottomLeft,
        PopupAnchor::Bottom => SctkAnchorPos::Bottom,
        PopupAnchor::BottomRight => SctkAnchorPos::BottomRight,
    }
}
fn gravity_to_sctk(gravity: PopupGravity) -> SctkGravity {
    match gravity {
        PopupGravity::TopLeft => SctkGravity::TopLeft,
        PopupGravity::Top => SctkGravity::Top,
        PopupGravity::TopRight => SctkGravity::TopRight,
        PopupGravity::Left => SctkGravity::Left,
        PopupGravity::Right => SctkGravity::Right,
        PopupGravity::BottomLeft => SctkGravity::BottomLeft,
        PopupGravity::Bottom => SctkGravity::Bottom,
        PopupGravity::BottomRight => SctkGravity::BottomRight,
    }
}
fn constraint_to_sctk(c: PopupConstraintAdjustment) -> SctkConstraint {
    [
        (c.flip_x, SctkConstraint::FlipX),
        (c.flip_y, SctkConstraint::FlipY),
        (c.slide_x, SctkConstraint::SlideX),
        (c.slide_y, SctkConstraint::SlideY),
        (c.resize_y, SctkConstraint::ResizeY),
    ]
    .into_iter()
    .filter(|(on, _)| *on)
    .fold(SctkConstraint::empty(), |all, (_, flag)| all | flag)
}

fn positioner(xdg_shell: &XdgShell, spec: &PopupSpec) -> Option<XdgPositioner> {
    let positioner = XdgPositioner::new(xdg_shell)
        .inspect_err(|error| tracing::error!(%error, "BarrierKit: could not create xdg_positioner"))
        .ok()?;
    let (ax, ay, aw, ah) = spec.anchor_rect;
    positioner.set_anchor_rect(ax, ay, aw as i32, ah as i32);
    positioner.set_anchor(anchor_to_sctk(spec.anchor));
    positioner.set_gravity(gravity_to_sctk(spec.gravity));
    positioner.set_offset(spec.offset.0, spec.offset.1);
    positioner.set_size(spec.size.0 as i32, spec.size.1 as i32);
    positioner.set_constraint_adjustment(constraint_to_sctk(spec.constraint_adjustment));
    Some(positioner)
}
impl Runtime {
    pub(crate) fn process_popup_commands(&mut self) {
        let commands = std::mem::take(&mut *self.popup_commands.borrow_mut());
        for command in commands {
            match command {
                PopupCommand::Open {
                    owner,
                    parent,
                    id,
                    spec,
                    content,
                    serial,
                    popover,
                } => {
                    self.open_popup(owner, parent, id, spec, content, serial);
                    if popover {
                        self.mark_popover(id);
                    }
                }
                PopupCommand::Reposition { id, spec } => {
                    self.reposition_popup(id, spec);
                }
                PopupCommand::Close { id } => {
                    self.close_popup(id);
                }
            }
        }
    }
    pub(crate) fn open_popup(
        &mut self,
        owner: ObjectId,
        parent: Option<PopupId>,
        id: PopupId,
        spec: PopupSpec,
        content: AppComponent,
        serial: u32,
    ) {
        let Some(owner_index) = self.surfaces.iter().position(|s| s.surface.id() == owner) else {
            return;
        };
        let Some(xdg_shell) = self.xdg_shell.as_ref() else {
            return;
        };
        let scale = self.surfaces[owner_index].scale;

        let parent_popup = parent.and_then(|parent_id| {
            self.surfaces
                .iter()
                .find_map(|s| match (&s.role, &s.popup) {
                    (SurfaceRole::Popup(popup), Some(info)) if info.id == parent_id => {
                        Some((popup.xdg_surface().clone(), info.grabbed))
                    }
                    _ => None,
                })
        });
        let parent_grabbed = parent_popup.as_ref().is_none_or(|(_, grabbed)| *grabbed);
        let parent_xdg_surface = parent_popup.map(|(surface, _)| surface).or_else(|| {
            if parent.is_none()
                && let SurfaceRole::Window(window) = &self.surfaces[owner_index].role
            {
                Some(window.xdg_surface().clone())
            } else {
                None
            }
        });

        let Some(positioner) = positioner(xdg_shell, &spec) else {
            return;
        };

        let surface = match sctk::compositor::Surface::new(&self.compositor, &self.qh) {
            Ok(surface) => surface,
            Err(error) => {
                tracing::error!(%error, "BarrierKit: could not create popup surface");
                return;
            }
        };
        let wl_surface = surface.wl_surface().clone();
        let popup = match Popup::from_surface(
            parent_xdg_surface.as_ref(),
            &positioner,
            &self.qh,
            surface,
            xdg_shell,
        ) {
            Ok(popup) => popup,
            Err(error) => {
                tracing::error!(%error, "BarrierKit: could not create xdg_popup");
                return;
            }
        };

        #[cfg(feature = "shell")]
        let attached = parent_xdg_surface.is_some()
            || parent.is_none()
                && self.surfaces[owner_index]
                    .role
                    .parent_popup(popup.xdg_popup());
        #[cfg(not(feature = "shell"))]
        let attached = parent_xdg_surface.is_some();
        // xdg_popup: a null parent must be set by another protocol before the first commit.
        if !attached {
            if let Some(dismissed) = &spec.on_dismissed {
                dismissed();
            }
            return;
        }
        if parent_xdg_surface.is_none() {
            self.surfaces[owner_index].surface.commit();
        }

        // xdg_popup.grab: a popup whose parent popup took no grab may not grab either.
        let grabbed = spec.grab && parent_grabbed && self.seat_handle.is_some();
        if grabbed && let Some(seat) = self.seat_handle.as_ref() {
            popup.xdg_popup().grab(seat, serial);
        }
        wl_surface.commit();

        let opens_left = Rc::new(Cell::new(false));
        let app = AppInstance::new(
            self,
            wl_surface.id(),
            content,
            scale,
            spec.size,
            None,
            Some((id, opens_left.clone())),
            None,
        );

        let output = self.surfaces[owner_index].output.clone();
        let mut window = self.surface_window(wl_surface, SurfaceRole::Popup(popup), app);
        window.popup = Some(PopupInfo {
            id,
            parent,
            on_dismissed: spec.on_dismissed,
            opens_left,
            grabbed,
            owner,
        });
        window.logical_size = spec.size;
        window.scale = scale;
        window.output = output;
        self.surfaces.push(window);
    }
    fn mark_popover(&self, id: PopupId) {
        let Some(style) = self
            .window_style
            .as_ref()
            .filter(|style| style.version() >= 2)
        else {
            return;
        };
        if let Some(window) = self
            .surfaces
            .iter()
            .find(|s| s.popup.as_ref().is_some_and(|info| info.id == id))
        {
            style.set_popover(&window.surface);
        }
    }
    pub(crate) fn reposition_popup(&mut self, id: PopupId, spec: PopupSpec) {
        let Some(index) = self
            .surfaces
            .iter()
            .position(|s| s.popup.as_ref().is_some_and(|info| info.id == id))
        else {
            return;
        };
        let Some(xdg_shell) = self.xdg_shell.as_ref() else {
            return;
        };
        let popup = match &self.surfaces[index].role {
            SurfaceRole::Popup(popup) => popup,
            _ => return,
        };

        let Some(positioner) = positioner(xdg_shell, &spec) else {
            return;
        };

        popup.reposition(&positioner, 0);
        self.surfaces[index].surface.commit();
    }
    pub(crate) fn close_popups_of(&mut self, owner: &ObjectId) {
        let owned: Vec<PopupId> = self
            .surfaces
            .iter()
            .filter_map(|s| s.popup.as_ref().filter(|info| info.owner == *owner))
            .map(|info| info.id)
            .collect();
        for id in owned {
            self.close_popup(id);
        }
    }

    pub(crate) fn close_popup(&mut self, id: PopupId) {
        let children: Vec<PopupId> = self
            .surfaces
            .iter()
            .filter_map(|s| {
                s.popup
                    .as_ref()
                    .filter(|info| info.parent == Some(id))
                    .map(|info| info.id)
            })
            .collect();
        for child in children {
            self.close_popup(child);
        }

        let Some(index) = self
            .surfaces
            .iter()
            .position(|s| s.popup.as_ref().is_some_and(|info| info.id == id))
        else {
            return;
        };
        if let Some(info) = self.surfaces[index].popup.take()
            && let Some(callback) = info.on_dismissed
        {
            callback();
        }
        self.surfaces.remove(index);
    }
}

impl PopupHandler for Runtime {
    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        popup: &Popup,
        config: PopupConfigure,
    ) {
        let surface = popup.wl_surface();
        let Some(index) = self.surface_for(surface) else {
            return;
        };
        let scale = self.surfaces[index].scale;
        let size = (config.width.max(0) as u32, config.height.max(0) as u32);
        if let Some(info) = self.surfaces[index].popup.as_ref() {
            info.opens_left.set(config.position.0 < 0);
        }
        self.surfaces[index].configure(size, scale);
        if self.surfaces[index].draw(&self.compositor) && self.surfaces[index].needs_redraw {
            self.surfaces[index].request_redraw(qh);
        }
    }

    fn done(&mut self, _: &Connection, _: &QueueHandle<Self>, popup: &Popup) {
        let surface = popup.wl_surface();
        if let Some(index) = self.surface_for(surface)
            && let Some(info) = self.surfaces[index].popup.as_ref()
        {
            let id = info.id;
            self.close_popup(id);
        }
    }
}
delegate_xdg_popup!(Runtime);
