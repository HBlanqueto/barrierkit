use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use ctdesign::{Rgba, SUBHEADLINE, TOOLTIP};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{CursorPoint, Gaps};

use crate::menu::{measure_text_weight, measure_wrapped};
use crate::platform::{PopupAnchor, PopupGravity, PopupId, PopupSpec, Popups, Timers};
use crate::style::text;
use crate::styles::{PopupRoot, Theme, current_theme};
use crate::theme::{color, resolved};

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
pub(crate) struct Tooltip {
    text: &'static str,
    shared: Rc<Shared>,
    popups: Option<Popups>,
    theme: Theme,
}

pub(crate) fn use_tooltip(text: &'static str) -> Tooltip {
    let shared = use_hook(|| Rc::new(Shared::default()));
    let on_drop = shared.clone();
    use_drop(move || on_drop.hide());
    Tooltip {
        text,
        shared,
        popups: try_consume_root_context(),
        theme: current_theme(),
    }
}

impl Tooltip {
    pub(crate) fn attach<E: EventHandlersExt>(self, element: E) -> E {
        let (enter, moved, left, pressed) = (
            self.clone(),
            self.shared.clone(),
            self.shared.clone(),
            self.shared,
        );
        element
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
    }

    fn show(&self) {
        let at = self.shared.pointer.get();
        let (text_width, lines) = lines_of(self.text);
        let width = text_width + 2.0 * TOOLTIP.padding_x;
        let height = lines as f32 * SUBHEADLINE.line_height as f32 + 2.0 * TOOLTIP.padding_y;
        let spec = PopupSpec {
            anchor_rect: (at.x.round() as i32, at.y.round() as i32, 1, 1),
            anchor: PopupAnchor::TopLeft,
            gravity: PopupGravity::BottomRight,
            offset: (0, TOOLTIP.pointer_gap.round() as i32),
            size: (width.ceil() as u32, height.ceil() as u32),
            ..PopupSpec::default()
        };
        let Some(popups) = self.popups.clone() else {
            return;
        };
        let content = AppComponent::new(PopupRoot {
            theme: self.theme.clone(),
            content: AppComponent::new(Tag {
                text: self.text,
                lines,
            }),
        });
        let id = popups.open(spec, content);
        self.shared.shown.replace(Some((popups, id)));
    }
}

fn lines_of(text: &str) -> (f32, usize) {
    let weight = FontWeight::from(SUBHEADLINE.weight as i32);
    let size = SUBHEADLINE.size as f32;
    let single = measure_text_weight(text, size, weight);
    if single <= TOOLTIP.wrap_after {
        (single, 1)
    } else {
        measure_wrapped(text, size, weight, TOOLTIP.wrap_width)
    }
}

fn tooltip_colors() -> (Rgba, Rgba) {
    let resolved = resolved();
    match (resolved.scheme.is_dark(), resolved.contrast) {
        (false, false) => (TOOLTIP.bg_light, TOOLTIP.ink_light),
        (true, false) => (TOOLTIP.bg_dark, TOOLTIP.ink_dark),
        (false, true) => (TOOLTIP.bg_contrast_light, TOOLTIP.ink_contrast_light),
        (true, true) => (TOOLTIP.bg_contrast_dark, TOOLTIP.ink_contrast_dark),
    }
}

struct Tag {
    text: &'static str,
    lines: usize,
}

impl App for Tag {
    fn render(&self) -> impl IntoElement {
        let (bg, ink) = tooltip_colors();
        rect()
            .expanded()
            .center()
            .padding(Gaps::new_symmetric(TOOLTIP.padding_y, TOOLTIP.padding_x))
            .corner_radius(TOOLTIP.radius)
            .background(color(bg))
            .child(text(self.text, SUBHEADLINE, ink).max_lines(self.lines))
    }
}
