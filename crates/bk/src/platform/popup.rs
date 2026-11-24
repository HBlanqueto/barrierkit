use freya_core::integration::AppComponent;
use freya_core::prelude::State;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{ObjectId, calloop::ping::Ping, window_style::PopoverShape};
use crate::styles::{PopupRoot, current_theme};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PopupId(u64);

impl PopupId {
    fn next(counter: &Rc<Cell<u64>>) -> Self {
        let id = counter.get();
        counter.set(id + 1);
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
    pub(crate) glass_luma: State<Option<f32>>,
}

impl Popups {
    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub fn open(&self, spec: PopupSpec, content: AppComponent) -> PopupId {
        self.push_open(spec, content, None)
    }

    /// The compositor draws the arrow in a band of `spec.size` along the edge that faces the
    /// anchor rectangle; `content` leaves that band empty.
    pub fn open_popover(
        &self,
        spec: PopupSpec,
        content: AppComponent,
        shape: PopoverShape,
    ) -> PopupId {
        self.push_open(spec, content, Some(shape))
    }

    fn push_open(
        &self,
        spec: PopupSpec,
        content: AppComponent,
        popover: Option<PopoverShape>,
    ) -> PopupId {
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

    pub fn reposition(&self, id: PopupId, spec: PopupSpec) {
        self.queue
            .borrow_mut()
            .push(PopupCommand::Reposition { id, spec });
        self.ping.ping();
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

pub(crate) enum PopupCommand {
    Open {
        owner: ObjectId,
        parent: Option<PopupId>,
        id: PopupId,
        spec: PopupSpec,
        content: AppComponent,
        serial: u32,
        popover: Option<PopoverShape>,
    },
    Reposition {
        id: PopupId,
        spec: PopupSpec,
    },
    Close {
        id: PopupId,
    },
}

pub(crate) fn anchor_rect(area: torin::prelude::Area) -> (i32, i32, u32, u32) {
    (
        area.origin.x.round() as i32,
        area.origin.y.round() as i32,
        area.size.width.round() as u32,
        area.size.height.round() as u32,
    )
}
