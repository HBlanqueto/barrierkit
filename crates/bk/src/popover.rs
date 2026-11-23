use std::cell::Cell;
use std::rc::Rc;

use ctdesign::POPOVER;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps};

use crate::platform::{
    PopoverShape, PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupId, PopupSpec, Popups,
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
        let spec = PopupSpec {
            anchor_rect: crate::platform::anchor_rect(anchor),
            anchor: PopupAnchor::Bottom,
            gravity: PopupGravity::Bottom,
            size: (
                size.0.ceil() as u32,
                (size.1 + POPOVER.arrow_height).ceil() as u32,
            ),
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
            AppComponent::new(PopoverBody(self.content)),
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

struct PopoverBody(Rc<dyn Fn() -> Element>);

impl App for PopoverBody {
    fn render(&self) -> impl IntoElement {
        let menu = crate::styles::popover_style();
        rect()
            .expanded()
            .background(color(menu.bg))
            .padding(Gaps::new(POPOVER.arrow_height, 0.0, 0.0, 0.0))
            .child((self.0)())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use freya_core::integration::AppComponent;
    use freya_testing::TestingRunner;
    use torin::prelude::Size;

    use super::*;
    use crate::modifiers::ViewModifiers;
    use crate::platform::{ObjectId, PopupCommand};

    struct Case(Rc<RefCell<Option<State<bool>>>>);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            let presented = use_state(|| false);
            *self.0.borrow_mut() = Some(presented);
            rect()
                .width(Size::px(100.0))
                .height(Size::px(30.0))
                .popover(presented, (50.0, 50.0), || rect().into_element())
        }
    }

    #[test]
    fn a_popover_opens_when_presented_closes_when_not_and_dismissal_clears_the_state() {
        let (popups, queue) = Popups::for_test(ObjectId::null(), None);
        let handle = Rc::default();
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Case(Rc::clone(&handle))),
            torin::prelude::Size2D::new(200.0, 100.0),
            move |runner| runner.provide_root_context(move || popups.clone()),
            1.0,
        );
        test.sync_and_update();
        let mut presented = handle.borrow().unwrap();
        presented.set(true);
        test.sync_and_update();
        test.sync_and_update();
        let dismissed = match queue.borrow_mut().drain(..).collect::<Vec<_>>().as_slice() {
            [PopupCommand::Open { spec, .. }] => spec.on_dismissed.clone().expect("dismissal"),
            other => panic!("expected one Open, got {}", other.len()),
        };
        dismissed();
        test.sync_and_update();
        assert!(!*presented.read());
        presented.set(true);
        test.sync_and_update();
        test.sync_and_update();
        presented.set(false);
        test.sync_and_update();
        let commands = queue.borrow_mut().drain(..).collect::<Vec<_>>();
        assert!(matches!(
            commands.as_slice(),
            [PopupCommand::Open { .. }, PopupCommand::Close { .. }]
        ));
    }
}
