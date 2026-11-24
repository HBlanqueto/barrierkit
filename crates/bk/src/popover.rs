use std::cell::Cell;
use std::rc::Rc;

use ctdesign::POPOVER;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps};

use crate::platform::{
    PopoverShape, PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId,
    PopupSpec, Popups,
};
use crate::theme::color;

pub struct Popover {
    size: (f32, f32),
    content: Rc<dyn Fn() -> Element>,
    on_dismissed: Option<Rc<dyn Fn()>>,
}

impl Popover {
    pub fn new(size: (f32, f32), content: impl Fn() -> Element + 'static) -> Self {
        Self {
            size,
            content: Rc::new(content),
            on_dismissed: None,
        }
    }

    pub fn on_dismissed(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_dismissed = Some(Rc::new(callback));
        self
    }

    pub fn show(self, anchor: Area) -> PopupId {
        let size = self.size;
        let native = try_consume_root_context::<crate::platform::backdrop::NativePopovers>()
            .is_some_and(|popovers| popovers.0);
        let arrow = if native { 0.0 } else { POPOVER.arrow_height };
        let spec = PopupSpec {
            anchor_rect: crate::platform::anchor_rect(anchor),
            anchor: PopupAnchor::Bottom,
            gravity: PopupGravity::Bottom,
            size: (size.0.ceil() as u32, (size.1 + arrow).ceil() as u32),
            constraint_adjustment: PopupConstraintAdjustment {
                slide_x: true,
                ..PopupConstraintAdjustment::default()
            },
            grab: true,
            on_dismissed: self.on_dismissed,
            ..PopupSpec::default()
        };
        Popups::get().open_popover(
            spec,
            AppComponent::new(PopoverBody(self.content, native)),
            PopoverShape::Popover,
        )
    }
}

pub(crate) fn use_popover<E: EventHandlersExt + LayoutExt>(
    element: E,
    is_presented: State<bool>,
    size: (f32, f32),
    content: impl Fn() -> Element + 'static,
) -> E {
    let mut anchor = use_state(Area::default);
    let area = *anchor.read();
    let presented = *is_presented.read();
    let shown = use_hook(|| Rc::new(Cell::new(None::<PopupId>)));
    let popups: Option<Popups> = try_consume_root_context();
    let (owned_popups, owned_shown) = (popups.clone(), shown.clone());
    use_drop(move || {
        if let (Some(popups), Some(id)) = (owned_popups, owned_shown.get()) {
            popups.close(id);
        }
    });
    if let Some(popups) = popups {
        match (presented && area.size.width > 0.0, shown.get()) {
            (true, None) => {
                let opened = Rc::new(Cell::new(None));
                let dismissed = {
                    let (shown, opened) = (shown.clone(), opened.clone());
                    move || {
                        if shown.get() == opened.get() {
                            shown.set(None);
                            let mut presented = is_presented;
                            presented.set(false);
                        }
                    }
                };
                let id = Popover::new(size, content)
                    .on_dismissed(dismissed)
                    .show(area);
                opened.set(Some(id));
                shown.set(Some(id));
            }
            (false, Some(id)) => {
                shown.set(None);
                popups.close(id);
            }
            _ => {}
        }
    }
    element.on_sized(move |event: Event<SizedEventData>| anchor.set_if_modified(event.area))
}

struct PopoverBody(Rc<dyn Fn() -> Element>, bool);

impl App for PopoverBody {
    fn render(&self) -> impl IntoElement {
        let PopoverBody(content, native) = self;
        let handle: Option<PopupHandle> = try_consume_root_context();
        rect()
            .expanded()
            .maybe(!native, |body| {
                body.background(color(crate::styles::popover_style().bg))
                    .padding(Gaps::new(POPOVER.arrow_height, 0.0, 0.0, 0.0))
            })
            .maybe(*native, |body| {
                body.on_global_key_down(move |event: Event<KeyboardEventData>| {
                    if event.key == Key::Named(NamedKey::Escape)
                        && let Some(handle) = &handle
                    {
                        handle.close();
                    }
                })
            })
            .child(content())
    }
}
