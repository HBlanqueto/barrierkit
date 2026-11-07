use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use ct_design::{CAPTION, TOOLTIP};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{CursorPoint, Gaps};

use crate::menu::measure_text_weight;
use crate::platform::{PopupAnchor, PopupGravity, PopupId, PopupSpec, Popups, Timers};
use crate::style::text;
use crate::theme::{SurfaceKind, color, popover_palette};

#[derive(Default)]
struct Shared {
    pointer: Cell<CursorPoint>,
    hovers: Cell<u64>,
    shown: RefCell<Option<(Popups, PopupId)>>,
}

impl Shared {
    fn hide(&self) {
        self.hovers.set(self.hovers.get() + 1);
        if let Some((popups, id)) = self.shown.take() {
            popups.close(id);
        }
    }
}

#[derive(Clone)]
pub struct Tooltip {
    text: &'static str,
    shared: Rc<Shared>,
}

pub fn use_tooltip(text: &'static str) -> Tooltip {
    let shared = use_hook(|| Rc::new(Shared::default()));
    let on_drop = shared.clone();
    use_drop(move || on_drop.hide());
    Tooltip { text, shared }
}

impl Tooltip {
    pub fn attach(self, element: impl Into<Element>) -> Rect {
        let (enter, moved, left, pressed) = (
            self.clone(),
            self.shared.clone(),
            self.shared.clone(),
            self.shared,
        );
        rect()
            .on_pointer_enter(move |event: Event<PointerEventData>| {
                let shared = &enter.shared;
                shared.pointer.set(event.global_location());
                shared.hovers.set(shared.hovers.get() + 1);
                let (hover, tooltip) = (shared.hovers.get(), enter.clone());
                Timers::get().after(Duration::from_millis(TOOLTIP.delay_ms), move || {
                    if tooltip.shared.hovers.get() == hover {
                        tooltip.show();
                    }
                });
            })
            .on_pointer_move(move |event: Event<PointerEventData>| {
                moved.pointer.set(event.global_location())
            })
            .on_pointer_leave(move |_| left.hide())
            .on_pointer_down(move |_| pressed.hide())
            .child(element)
    }

    fn show(&self) {
        let at = self.shared.pointer.get();
        let caption_width = measure_text_weight(
            self.text,
            CAPTION.size as f32,
            FontWeight::from(CAPTION.weight as i32),
        );
        let width = caption_width + 2.0 * TOOLTIP.padding_x;
        let height = CAPTION.line_height as f32 + 2.0 * TOOLTIP.padding_y;
        let spec = PopupSpec {
            anchor_rect: (at.x.round() as i32, at.y.round() as i32, 1, 1),
            anchor: PopupAnchor::TopLeft,
            gravity: PopupGravity::BottomRight,
            offset: (0, TOOLTIP.pointer_gap.round() as i32),
            size: (width.ceil() as u32, height.ceil() as u32),
            ..PopupSpec::default()
        };
        let popups = Popups::get();
        let id = popups.open(spec, AppComponent::new(Tag(self.text)));
        self.shared.shown.replace(Some((popups, id)));
    }
}

struct Tag(&'static str);

impl App for Tag {
    fn render(&self) -> impl IntoElement {
        let menu = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
        rect()
            .expanded()
            .center()
            .padding(Gaps::new_symmetric(TOOLTIP.padding_y, TOOLTIP.padding_x))
            .corner_radius(TOOLTIP.radius)
            .background(color(menu.bg))
            .border(
                Border::new()
                    .width(ct_design::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(menu.separator_color)),
            )
            .child(text(self.0, CAPTION, menu.fg).max_lines(1))
    }
}
