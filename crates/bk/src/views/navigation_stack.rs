//! U5 of `docs/research/uikit-roadmap.md`: a stack of pages that the app edits as a path, and the push and pop of UIKit.

use std::borrow::Cow;

use ctdesign::{NAVIGATION_DIM, NAVIGATION_PARALLAX, Rgba, SPRING_PUSH_MS};
use freya_core::prelude::*;
use torin::prelude::{Position, Size};

use crate::animation::{pixel_aligned, use_animated};
use crate::controls::Button;
use crate::styles::Animation;
use crate::theme::color;

/// SwiftUI's `NavigationPath`, typed: the routes pushed over the root page, last on top.
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationPath<R> {
    routes: Vec<R>,
}

impl<R> Default for NavigationPath<R> {
    fn default() -> Self {
        Self { routes: Vec::new() }
    }
}

impl<R> NavigationPath<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, route: R) {
        self.routes.push(route);
    }

    pub fn pop(&mut self) -> Option<R> {
        self.routes.pop()
    }

    pub fn pop_to_root(&mut self) {
        self.routes.clear();
    }

    pub fn truncate(&mut self, len: usize) {
        self.routes.truncate(len);
    }

    pub fn len(&self) -> usize {
        self.routes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

// Where page `index` is, in pixels from the left edge, when the stack shows page `position` (a fraction while it moves): the pages
// above come in from the right edge and the pages below leave PARALLAX of the width to the left (0 on macOS: the page below stays).
fn page_offset(index: usize, position: f32, width: f32) -> f32 {
    let behind = index as f32 - position;
    if behind >= 0.0 {
        behind * width
    } else {
        NAVIGATION_PARALLAX * width * behind.max(-1.0)
    }
}

fn page_dim(index: usize, position: f32) -> f32 {
    NAVIGATION_DIM * (position - index as f32).clamp(0.0, 1.0)
}

fn visible(position: f32, pages: usize) -> std::ops::RangeInclusive<usize> {
    let low = (position.floor().max(0.0) as usize).min(pages.saturating_sub(1));
    let high = (position.ceil().max(0.0) as usize).min(pages.saturating_sub(1));
    low..=high
}

/// SwiftUI's `NavigationStack`: the root page, and one page per route of the path through `destination`.
#[derive(PartialEq)]
pub struct NavigationStack<R: Clone + PartialEq + 'static> {
    path: State<NavigationPath<R>>,
    root: Element,
    destination: Callback<R, Element>,
}

impl<R: Clone + PartialEq + 'static> NavigationStack<R> {
    pub fn new<V: IntoElement + 'static>(
        path: State<NavigationPath<R>>,
        root: impl IntoElement,
        destination: impl Fn(&R) -> V + 'static,
    ) -> Self {
        Self {
            path,
            root: root.into_element(),
            destination: Callback::new(move |route: R| destination(&route).into_element()),
        }
    }
}

/// The path of the `NavigationStack` above, so a deep child pushes and pops without a callback chain.
pub fn use_navigation_path<R: Clone + 'static>() -> State<NavigationPath<R>> {
    consume_context()
}

/// SwiftUI's `NavigationLink(value:)`: a button that pushes `route` onto the path of the stack above it.
#[derive(PartialEq)]
pub struct NavigationLink<R: Clone + PartialEq + 'static> {
    label: Cow<'static, str>,
    route: R,
}

impl<R: Clone + PartialEq + 'static> NavigationLink<R> {
    pub fn new(label: impl Into<Cow<'static, str>>, route: R) -> Self {
        Self {
            label: label.into(),
            route,
        }
    }
}

impl<R: Clone + PartialEq + 'static> Component for NavigationLink<R> {
    fn render(&self) -> impl IntoElement {
        let path = try_consume_context::<State<NavigationPath<R>>>();
        let route = self.route.clone();
        Button::new(self.label.clone())
            .disabled(path.is_none())
            .on_press(move |_| {
                if let Some(mut path) = path {
                    path.write().push(route.clone());
                }
            })
    }
}

impl<R: Clone + PartialEq + 'static> Component for NavigationStack<R> {
    fn render(&self) -> impl IntoElement {
        use_provide_context(|| self.path);
        let routes = self.path.read().routes.clone();
        let mut size = use_state(|| 0.0f32);
        let mut leaving = use_state(Vec::<R>::new);
        let target = routes.len() as f32;
        let position = use_animated(
            target,
            Animation::Spring {
                duration_ms: SPRING_PUSH_MS,
                bounce: 0.0,
            },
        );
        let settled = position == target;
        let shown = if settled || leaving.peek().len() <= routes.len() {
            routes.clone()
        } else {
            leaving.peek().clone()
        };
        if settled && *leaving.peek() != routes {
            leaving.set(routes);
        }
        let width = *size.read();
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .on_sized(move |event: Event<SizedEventData>| {
                size.set_if_modified(event.area.width());
            })
            .children(visible(position, shown.len() + 1).map(|index| {
                let page = if index == 0 {
                    self.root.clone()
                } else {
                    self.destination.call(shown[index - 1].clone())
                };
                let dim = page_dim(index, position);
                rect()
                    .key(index)
                    .position(
                        Position::new_absolute()
                            .left(pixel_aligned(page_offset(index, position, width)))
                            .top(0.0),
                    )
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(page)
                    .maybe_child((dim > 0.0).then(|| {
                        rect()
                            .position(Position::new_absolute().left(0.0).top(0.0))
                            .width(Size::fill())
                            .height(Size::fill())
                            .background(color(Rgba {
                                a: dim,
                                ..Rgba::TRANSPARENT
                            }))
                    }))
            }))
    }
}
