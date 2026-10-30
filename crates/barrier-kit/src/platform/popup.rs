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
        XdgPositioner, XdgSurface,
        popup::{Popup, PopupConfigure, PopupHandler},
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use super::{
    app::AppInstance,
    runtime::{Runtime, make_sender},
    surface::{SurfaceRole, SurfaceWindow},
    timer::{PingWake, Timers},
};

#[cfg(test)]
use sctk::reexports::calloop;

/// Identifier of an open popup within this process.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PopupId(u64);

impl PopupId {
    fn next(counter: &Rc<Cell<u64>>) -> Self {
        let id = counter.get();
        counter.set(id + 1);
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) fn test(id: u64) -> Self {
        Self(id)
    }
}

/// Which edge of the anchor rectangle a popup attaches to.
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

/// In which direction a popup grows from its anchor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopupGravity {
    /// Up and left.
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

/// How the compositor may adjust a popup that does not fit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopupConstraintAdjustment {
    pub flip_x: bool,
    pub flip_y: bool,
    pub slide_x: bool,
    pub slide_y: bool,
}

/// Options for opening a popup from the shell (menubar, submenus).
#[derive(Clone)]
pub struct PopupSpec {
    /// Rectangle on the parent surface, in logical pixels, that the popup anchors to.
    pub anchor_rect: (i32, i32, u32, u32),
    pub anchor: PopupAnchor,
    pub gravity: PopupGravity,
    /// Offset in logical pixels applied after anchoring and gravity.
    pub offset: (i32, i32),
    /// Preferred logical size of the popup.
    pub size: (u32, u32),
    pub constraint_adjustment: PopupConstraintAdjustment,
    /// Request an implicit grab with the last input serial.
    pub grab: bool,
    /// Called once when the popup is dismissed for any reason.
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

/// Opens and closes popups from inside a BarrierKit surface.
#[derive(Clone)]
pub struct Popups {
    pub(super) queue: Rc<RefCell<Vec<PopupCommand>>>,
    pub(super) ping: Ping,
    pub(super) next_id: Rc<Cell<u64>>,
    pub(super) last_serial: Rc<Cell<u32>>,
    pub(super) owner: ObjectId,
    pub(super) current: Option<PopupId>,
    pub(super) surface_count: Rc<Cell<usize>>,
    pub(super) opens_left: Rc<Cell<bool>>,
}

impl Popups {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    /// Opens a popup attached to the current surface (or to this popup, if the
    /// context belongs to a popup).
    pub fn open(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        let id = PopupId::next(&self.next_id);
        self.queue.borrow_mut().push(PopupCommand::Open {
            owner: self.owner.clone(),
            parent: self.current,
            id,
            spec,
            content,
            serial: self.last_serial.get(),
        });
        self.ping.ping();
        id
    }

    /// Closes a popup and its descendants.
    pub fn close(&self, id: PopupId) {
        self.queue.borrow_mut().push(PopupCommand::Close { id });
        self.ping.ping();
    }

    /// Repositions an already open popup with a new geometry.
    fn reposition(&self, id: PopupId, spec: PopupSpec) {
        self.queue
            .borrow_mut()
            .push(PopupCommand::Reposition { id, spec });
        self.ping.ping();
    }

    /// The number of popup surfaces currently tracked by the runner. Useful for
    /// leak checks in tests and self-tests.
    pub fn surface_count(&self) -> usize {
        self.surface_count.get()
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
            surface_count: Rc::new(Cell::new(0)),
            opens_left: Rc::new(Cell::new(false)),
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

/// Handle available inside the content of a popup surface.
#[derive(Clone)]
pub struct PopupHandle {
    id: PopupId,
    popups: Popups,
}

impl PopupHandle {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn id(&self) -> PopupId {
        self.id
    }

    /// True when the compositor configured this popup to the left of its
    /// parent (its configure `x` is negative). Descendant submenus use it to
    /// keep opening on the same side.
    pub fn opens_left(&self) -> bool {
        self.popups.opens_left.get()
    }

    pub fn close(&self) {
        self.popups.close(self.id);
    }

    /// Repositions this popup with a new size and anchor rectangle.
    ///
    /// The caller is responsible for requesting a redraw so the surface is
    /// resized before the compositor commits it.
    pub fn reposition(&self, spec: PopupSpec) {
        self.popups.reposition(self.id, spec);
    }

    /// Opens a submenu attached to this popup.
    pub fn open_child(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        self.popups.open(spec, content)
    }
}
/// Popup bookkeeping attached to a `SurfaceWindow` whose role is `Popup`.
pub(super) struct PopupInfo {
    id: PopupId,
    parent: Option<PopupId>,
    on_dismissed: Option<Rc<dyn Fn()>>,
    opens_left: Rc<Cell<bool>>,
}

pub(crate) enum PopupCommand {
    Open {
        owner: ObjectId,
        parent: Option<PopupId>,
        id: PopupId,
        spec: PopupSpec,
        content: AppComponent,
        serial: u32,
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
    let mut out = SctkConstraint::empty();
    if c.flip_x {
        out |= SctkConstraint::FlipX;
    }
    if c.flip_y {
        out |= SctkConstraint::FlipY;
    }
    if c.slide_x {
        out |= SctkConstraint::SlideX;
    }
    if c.slide_y {
        out |= SctkConstraint::SlideY;
    }
    out
}
impl Runtime {
    pub(super) fn process_popup_commands(&mut self) {
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
                } => {
                    self.open_popup(owner, parent, id, spec, content, serial);
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
    pub(super) fn open_popup(
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

        let parent_xdg_surface = parent
            .and_then(|parent_id| {
                self.surfaces.iter().find_map(|s| {
                    if let SurfaceRole::Popup(popup) = &s.role
                        && s.popup.as_ref().is_some_and(|info| info.id == parent_id)
                    {
                        Some(popup.xdg_surface().clone())
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                // A toplevel parent must be passed as the xdg_surface parent,
                // not left `None`: xdg_popup requires an xdg_surface parent.
                // Layer-shell parents are set later with `get_popup`.
                if parent.is_none()
                    && let SurfaceRole::Window(window) = &self.surfaces[owner_index].role
                {
                    Some(window.xdg_surface().clone())
                } else {
                    None
                }
            });

        let positioner = match XdgPositioner::new(xdg_shell) {
            Ok(positioner) => positioner,
            Err(error) => {
                tracing::error!(%error, "BarrierKit: could not create xdg_positioner");
                return;
            }
        };
        let (ax, ay, aw, ah) = spec.anchor_rect;
        positioner.set_anchor_rect(ax, ay, aw as i32, ah as i32);
        positioner.set_anchor(anchor_to_sctk(spec.anchor));
        positioner.set_gravity(gravity_to_sctk(spec.gravity));
        let (ox, oy) = spec.offset;
        positioner.set_offset(ox, oy);
        positioner.set_size(spec.size.0 as i32, spec.size.1 as i32);
        positioner.set_constraint_adjustment(constraint_to_sctk(spec.constraint_adjustment));

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

        // If the parent is a layer surface, use get_popup to set it as the
        // implicit parent (an xdg_popup cannot have a layer surface as its
        // direct xdg parent).
        if parent.is_none()
            && let SurfaceRole::Layer(layer_surface) = &self.surfaces[owner_index].role
        {
            layer_surface.get_popup(popup.xdg_popup());
            // `get_popup` takes effect when the parent layer surface commits.
            self.surfaces[owner_index].surface.commit();
        }

        if spec.grab
            && let Some(seat) = self.activation_seat.as_ref()
        {
            popup.xdg_popup().grab(seat, serial);
        }
        wl_surface.commit();

        let waker = futures_util::task::waker(Arc::new(PingWake(self.ping.clone())));
        let sender = make_sender(self.queue.clone(), self.ping.clone(), wl_surface.id());
        let opens_left = Rc::new(Cell::new(false));
        let popups = Popups {
            queue: self.popup_commands.clone(),
            ping: self.ping.clone(),
            next_id: self.popup_next_id.clone(),
            last_serial: self.last_input_serial.clone(),
            owner: wl_surface.id(),
            current: Some(id),
            surface_count: self.surface_count.clone(),
            opens_left: opens_left.clone(),
        };
        let handle = PopupHandle {
            id,
            popups: popups.clone(),
        };
        let app = AppInstance::new(
            content,
            scale,
            spec.size,
            sender,
            waker,
            Some(self.toplevels.clone()),
            None,
            self.activation.clone(),
            None,
            Timers {
                queue: self.timers.clone(),
                ping: self.ping.clone(),
                wake: self.wake.clone(),
            },
            popups,
            Some(handle),
            self.graphics.font_collection.clone(),
            self.graphics.font_manager.clone(),
            None,
            None,
            self.resolved,
        );

        let (fractional, viewport) = self.create_fractional(&wl_surface);
        self.surfaces.push(SurfaceWindow {
            surface: wl_surface,
            role: SurfaceRole::Popup(popup),
            popup: Some(PopupInfo {
                id,
                parent,
                on_dismissed: spec.on_dismissed,
                opens_left,
            }),
            shared: self.graphics.clone(),
            graphics: None,
            app,
            logical_size: spec.size,
            buffer_size: (0, 0),
            scale,
            output: self.surfaces[owner_index].output.clone(),
            needs_redraw: true,
            frame_pending: false,
            draw_count: 0,
            configure_count: 0,
            pending_configure: None,
            appmenu: None,
            appmenu_address: None,
            menu_handle: None,
            fractional,
            viewport,
        });
        self.surface_count.set(self.surfaces.len());
    }
    pub(super) fn reposition_popup(&mut self, id: PopupId, spec: PopupSpec) {
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

        let positioner = match XdgPositioner::new(xdg_shell) {
            Ok(positioner) => positioner,
            Err(error) => {
                tracing::error!(%error, "BarrierKit: could not create xdg_positioner");
                return;
            }
        };
        let (ax, ay, aw, ah) = spec.anchor_rect;
        positioner.set_anchor_rect(ax, ay, aw as i32, ah as i32);
        positioner.set_anchor(anchor_to_sctk(spec.anchor));
        positioner.set_gravity(gravity_to_sctk(spec.gravity));
        let (ox, oy) = spec.offset;
        positioner.set_offset(ox, oy);
        positioner.set_size(spec.size.0 as i32, spec.size.1 as i32);
        positioner.set_constraint_adjustment(constraint_to_sctk(spec.constraint_adjustment));

        popup.reposition(&positioner, 0);
        self.surfaces[index].surface.commit();
    }
    pub(super) fn close_popup(&mut self, id: PopupId) {
        // Close descendants first so Wayland objects are destroyed leaf-to-root.
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
        // Drop the xdg_popup before its wl_surface: the other order destroys
        // the surface first, so the compositor never sees `popup_destroyed`
        // and keeps the old popup in its scene.
        let mut window = self.surfaces.remove(index);
        drop(std::mem::replace(&mut window.role, SurfaceRole::None));
        drop(window);
        self.surface_count.set(self.surfaces.len());
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
        // Like a toplevel, a popup must attach its first buffer before it can
        // map; there is no frame callback to wait for yet.
        if self.surfaces[index].draw() && self.surfaces[index].needs_redraw {
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
