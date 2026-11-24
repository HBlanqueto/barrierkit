use std::cell::RefCell;
use std::rc::Rc;

use freya_core::prelude::*;
use torin::prelude::{Position, Size2D};

use crate::animation::use_animated;
use crate::elements::forward_key;
use crate::styles::{Animation, Edge};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    opacity: f32,
    scale: f32,
    offset: (f32, f32),
    edge: Option<Edge>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Placed {
    opacity: f32,
    scale: f32,
    offset: (f32, f32),
}

impl Look {
    const IDENTITY: Self = Self {
        opacity: 1.0,
        scale: 1.0,
        offset: (0.0, 0.0),
        edge: None,
    };

    fn combined(self, other: Self) -> Self {
        Self {
            opacity: self.opacity * other.opacity,
            scale: self.scale * other.scale,
            offset: (
                self.offset.0 + other.offset.0,
                self.offset.1 + other.offset.1,
            ),
            edge: self.edge.or(other.edge),
        }
    }

    // `progress` is 1 while the view is on screen and 0 once it is gone; a spring may pass either end.
    fn placed(self, progress: f32, (width, height): (f32, f32)) -> Placed {
        let away = 1.0 - progress;
        let (edge_x, edge_y) = match self.edge {
            Some(Edge::Top) => (0.0, -height),
            Some(Edge::Bottom) => (0.0, height),
            Some(Edge::Leading) => (-width, 0.0),
            Some(Edge::Trailing) => (width, 0.0),
            None => (0.0, 0.0),
        };
        Placed {
            opacity: (1.0 + (self.opacity - 1.0) * away).clamp(0.0, 1.0),
            scale: (1.0 + (self.scale - 1.0) * away).max(0.0),
            offset: (
                (self.offset.0 + edge_x) * away,
                (self.offset.1 + edge_y) * away,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    insertion: Look,
    removal: Look,
}

impl Transition {
    pub const IDENTITY: Self = Self::both(Look::IDENTITY);
    pub const OPACITY: Self = Self::both(Look {
        opacity: 0.0,
        ..Look::IDENTITY
    });
    pub const SCALE: Self = Self::both(Look {
        scale: 0.0,
        ..Look::IDENTITY
    });

    const fn both(look: Look) -> Self {
        Self {
            insertion: look,
            removal: look,
        }
    }

    pub const fn scale(scale: f32) -> Self {
        Self::both(Look {
            scale,
            ..Look::IDENTITY
        })
    }

    pub const fn offset(x: f32, y: f32) -> Self {
        Self::both(Look {
            offset: (x, y),
            ..Look::IDENTITY
        })
    }

    pub const fn move_edge(edge: Edge) -> Self {
        Self::both(Look {
            edge: Some(edge),
            ..Look::IDENTITY
        })
    }

    pub fn combined(self, other: Self) -> Self {
        Self {
            insertion: self.insertion.combined(other.insertion),
            removal: self.removal.combined(other.removal),
        }
    }

    pub const fn asymmetric(insertion: Self, removal: Self) -> Self {
        Self {
            insertion: insertion.insertion,
            removal: removal.removal,
        }
    }
}

#[derive(PartialEq)]
pub struct Presence {
    content: Option<Element>,
    transition: Transition,
    key: DiffKey,
}

impl Presence {
    pub fn new(content: Option<impl IntoElement>) -> Self {
        Self {
            content: content.map(IntoElement::into_element),
            transition: Transition::OPACITY,
            key: DiffKey::None,
        }
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = transition;
        self
    }
}

forward_key!(Presence);

impl Component for Presence {
    fn render(&self) -> impl IntoElement {
        let last = use_hook(|| Rc::new(RefCell::new(None::<Element>)));
        let mut size = use_state(Size2D::default);
        let present = self.content.is_some();
        if let Some(content) = &self.content {
            last.replace(Some(content.clone()));
        }
        let progress = use_animated(f32::from(present), Animation::None);
        if !present && progress <= 0.0 {
            last.replace(None);
        }
        let Some(shown) = last.borrow().clone() else {
            return rect().position(Position::new_absolute()).into_element();
        };
        let look = if present {
            self.transition.insertion
        } else {
            self.transition.removal
        };
        let measured = *size.read();
        let mut placed = look.placed(progress, (measured.width, measured.height));
        // An edge is the view's own size, which a view only has after its first layout.
        if look.edge.is_some() && measured == Size2D::default() {
            placed.opacity = 0.0;
        }
        rect()
            .opacity(placed.opacity)
            .scale(placed.scale)
            .offset_x(placed.offset.0)
            .offset_y(placed.offset.1)
            .interactive(present)
            .on_sized(move |event: Event<SizedEventData>| size.set_if_modified(event.area.size))
            .child(shown)
            .into_element()
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentTransition {
    Identity,
    Opacity,
}

#[derive(PartialEq)]
pub(crate) struct ContentTransitioned {
    content: Element,
    transition: ContentTransition,
}

impl ContentTransitioned {
    pub(crate) fn new(content: Element, transition: ContentTransition) -> Self {
        Self {
            content,
            transition,
        }
    }
}

impl Component for ContentTransitioned {
    fn render(&self) -> impl IntoElement {
        // The view that shows the content now, and the content it showed last: a change hands the content to the other slot.
        let current = use_hook(|| Rc::new(RefCell::new((0usize, self.content.clone()))));
        if self.transition == ContentTransition::Identity {
            return self.content.clone();
        }
        let active = {
            let mut current = current.borrow_mut();
            if current.1 != self.content {
                *current = (1 - current.0, self.content.clone());
            }
            current.0
        };
        // The slot that leaves lies over the one that enters, so both are one view's size.
        let slot = |index: usize| {
            rect()
                .maybe(index != active, |slot| {
                    slot.position(Position::new_absolute())
                })
                .child(Presence::new(
                    (index == active).then(|| self.content.clone()),
                ))
        };
        rect().child(slot(0)).child(slot(1)).into_element()
    }
}
