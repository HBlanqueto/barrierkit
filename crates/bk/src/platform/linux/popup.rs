use freya_core::integration::AppComponent;
use sctk::{
    delegate_xdg_popup,
    reexports::{
        client::{Connection, Proxy, QueueHandle},
        protocols::xdg::shell::client::xdg_positioner::{
            Anchor as SctkAnchorPos, ConstraintAdjustment as SctkConstraint, Gravity as SctkGravity,
        },
    },
    shell::xdg::{
        XdgPositioner, XdgShell, XdgSurface,
        popup::{Popup, PopupConfigure, PopupHandler},
    },
};
use std::{cell::Cell, rc::Rc};

use crate::platform::{
    ObjectId,
    app::{AppInstance, InstanceRole},
    linux::window_style::wire,
    popup::{
        PopupAnchor, PopupCommand, PopupConstraintAdjustment, PopupGravity, PopupId, PopupSpec,
    },
    runtime::Runtime,
    surface::{SurfaceRole, buffer_size},
    window_style::PopoverShape,
};

pub(crate) struct PopupInfo {
    id: PopupId,
    parent: Option<PopupId>,
    on_dismissed: Option<Rc<dyn Fn()>>,
    opens_left: Rc<Cell<bool>>,
    grabbed: bool,
    owner: ObjectId,
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
    // xdg_positioner.invalid_input: a size or an anchor rectangle below 1 is a fatal protocol error.
    let side = |length: u32| length.clamp(1, i32::MAX as u32) as i32;
    let (ax, ay, aw, ah) = spec.anchor_rect;
    positioner.set_anchor_rect(ax, ay, side(aw), side(ah));
    positioner.set_anchor(anchor_to_sctk(spec.anchor));
    positioner.set_gravity(gravity_to_sctk(spec.gravity));
    positioner.set_offset(spec.offset.0, spec.offset.1);
    positioner.set_size(side(spec.size.0), side(spec.size.1));
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
                    if let Some(shape) = popover {
                        self.mark_popover(id, shape);
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
            &self.host(),
            wl_surface.id(),
            content,
            scale,
            buffer_size(spec.size, scale),
            InstanceRole::Popup {
                id,
                opens_left: opens_left.clone(),
            },
            |runner| self.provide_extension_contexts(runner),
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
        window.glass_notice = self
            .focus_manager
            .as_ref()
            .filter(|manager| manager.version() >= 5)
            .map(|manager| {
                manager.get_focus_surface(&window.surface, &self.qh, window.surface.id())
            });
        self.surfaces.push(window);
    }
    fn mark_popover(&self, id: PopupId, shape: PopoverShape) {
        let Some(style) = self.window_style.as_ref() else {
            return;
        };
        let Some(window) = self
            .surfaces
            .iter()
            .find(|s| s.popup.as_ref().is_some_and(|info| info.id == id))
        else {
            return;
        };
        if style.version() >= 7 {
            style.set_popover_shape(&window.surface, wire(shape as u32));
        } else if style.version() >= 2 && shape == PopoverShape::Popover {
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
        self.surfaces[index].draw_after_configure(&self.compositor, qh);
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
