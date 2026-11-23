//! U5 of `docs/research/uikit-roadmap.md`: a stack of pages that the app edits as a path, and the push and pop of UIKit.

use ctdesign::{NAVIGATION_DIM, NAVIGATION_PARALLAX, SPRING_PUSH_MS};
use freya_core::prelude::*;
use torin::prelude::{Position, Size};

use crate::animation::{effective, use_animated};
use crate::styles::Animation;

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
// above come in from the right edge and the pages below leave PARALLAX of the width to the left, as UIKit's push does.
fn page_offset(index: usize, position: f32, width: f32) -> f32 {
    let behind = index as f32 - position;
    if behind >= 0.0 {
        behind * width
    } else {
        NAVIGATION_PARALLAX * width * behind.max(-1.0)
    }
}

// The black laid over a page that a page above is covering: none at rest, DIM when fully covered.
fn page_dim(index: usize, position: f32) -> f32 {
    NAVIGATION_DIM * (position - index as f32).clamp(0.0, 1.0)
}

// The pages worth drawing while the stack is at `position`: those within one page of it.
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

impl<R: Clone + PartialEq + 'static> Component for NavigationStack<R> {
    fn render(&self) -> impl IntoElement {
        use_provide_context(|| self.path);
        let routes = self.path.read().routes.clone();
        let mut size = use_state(|| 0.0f32);
        let mut leaving = use_state(Vec::<R>::new);
        let animation = effective(
            Animation::Spring {
                duration_ms: SPRING_PUSH_MS,
                bounce: 0.0,
            },
            crate::theme::resolved().reduced_motion,
        );
        let target = routes.len() as f32;
        let position = use_animated(target, animation);
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
        let mut stack = rect()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .on_sized(move |event: Event<SizedEventData>| {
                size.set_if_modified(event.area.width());
            });
        for index in visible(position, shown.len() + 1) {
            let page = if index == 0 {
                self.root.clone()
            } else {
                self.destination.call(shown[index - 1].clone())
            };
            let dim = page_dim(index, position);
            stack = stack.child(
                rect()
                    .key(index)
                    .position(
                        Position::new_absolute()
                            .left(page_offset(index, position, width))
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
                            .background(Color::from_argb((dim * 255.0).round() as u8, 0, 0, 0))
                    })),
            );
        }
        stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_pushes_pops_and_goes_back_to_the_root() {
        let mut path = NavigationPath::new();
        path.push("a");
        path.push("b");
        path.push("c");
        assert_eq!(path.len(), 3);
        assert_eq!(path.pop(), Some("c"));
        path.truncate(1);
        assert_eq!(path.len(), 1);
        path.pop_to_root();
        assert!(path.is_empty());
        assert_eq!(path.pop(), None);
    }

    #[test]
    fn a_pushed_page_comes_in_from_the_edge_and_the_old_one_leaves_by_the_parallax() {
        assert_eq!(page_offset(1, 0.0, 700.0), 700.0);
        assert_eq!(page_offset(1, 0.5, 700.0), 350.0);
        assert_eq!(page_offset(1, 1.0, 700.0), 0.0);
        assert_eq!(page_offset(0, 0.0, 700.0), 0.0);
        assert!((page_offset(0, 1.0, 700.0) + 210.0).abs() < 1e-3);
        assert!((page_offset(0, 0.5, 700.0) + 105.0).abs() < 1e-3);
        assert!((page_offset(0, 2.0, 700.0) + 210.0).abs() < 1e-3);
    }

    #[test]
    fn only_a_covered_page_is_dimmed() {
        assert_eq!(page_dim(1, 1.0), 0.0);
        assert!((page_dim(0, 1.0) - NAVIGATION_DIM).abs() < 1e-6);
        assert!((page_dim(0, 0.5) - NAVIGATION_DIM / 2.0).abs() < 1e-6);
    }

    #[test]
    fn at_rest_one_page_is_drawn_and_while_moving_two() {
        assert_eq!(visible(2.0, 3), 2..=2);
        assert_eq!(visible(1.4, 3), 1..=2);
        assert_eq!(visible(0.0, 1), 0..=0);
        assert_eq!(visible(0.6, 2), 0..=1);
    }

    // The table of UIKit animations: the push reached 50 % at 62 ms and 90 % at 165 ms (Mac Catalyst, own probe).
    #[test]
    fn the_push_spring_reaches_half_and_nine_tenths_when_the_measured_push_did() {
        use std::time::Duration;

        use crate::animation::Spring;
        let spring = Spring::new(0.0, 1.0, Duration::from_millis(SPRING_PUSH_MS), 0.0);
        let at = |level: f32| {
            (0..400)
                .find(|ms| spring.value(Duration::from_millis(*ms)) >= level)
                .unwrap()
        };
        assert!((55..=75).contains(&at(0.5)), "{}", at(0.5));
        assert!((145..=175).contains(&at(0.9)), "{}", at(0.9));
    }
}
