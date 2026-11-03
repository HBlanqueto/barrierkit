//! Scroll view whose bar follows the appearance: only while scrolling, or always.

use freya_components::scrollviews::{ScrollConfig, ScrollView as Scroller, use_scroll_controller};
use freya_core::prelude::*;
use torin::prelude::{Position, Size};
use vow_appearance::ScrollBars;
use vow_design::{Rgba, SCROLL_BAR};

use crate::style::Palette;
use crate::theme::color;

// Freya paints by tree depth; the thumb must sit above the scrolled content, like Freya's own bar.
const THUMB_LAYER: i16 = 999;

#[derive(Clone, PartialEq)]
pub struct ScrollView {
    children: Vec<Element>,
    width: Size,
    height: Size,
    key: DiffKey,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            width: Size::fill(),
            height: Size::fill(),
            key: DiffKey::None,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }
}

impl ChildrenExt for ScrollView {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl KeyExt for ScrollView {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

fn thumb(viewport: f32, content: f32, scrolled: f32) -> Option<(f32, f32)> {
    if viewport <= 0.0 || content <= viewport {
        return None;
    }
    let length = (viewport * viewport / content)
        .max(SCROLL_BAR.min_length)
        .min(viewport);
    let travel = (-scrolled / (content - viewport)).clamp(0.0, 1.0);
    Some((travel * (viewport - length), length))
}

impl Component for ScrollView {
    fn render(&self) -> impl IntoElement {
        let always = crate::theme::resolved().scroll_bars == ScrollBars::Always;
        let controller = use_scroll_controller(ScrollConfig::default);
        let mut viewport = use_state(|| 0.0f32);
        let mut content = use_state(|| 0.0f32);
        let (_, scrolled): (i32, i32) = controller.into();
        let bar = if always {
            thumb(*viewport.read(), *content.read(), scrolled as f32)
        } else {
            None
        };
        let ink = Palette::current().label;
        rect()
            .width(self.width.clone())
            .height(self.height.clone())
            .on_sized(move |event: Event<SizedEventData>| {
                viewport.set_if_modified(event.area.height())
            })
            .child(
                Scroller::new_controlled(controller)
                    .show_scrollbar(!always)
                    .child(
                        rect()
                            .width(Size::fill())
                            .on_sized(move |event: Event<SizedEventData>| {
                                content.set_if_modified(event.area.height())
                            })
                            .children(self.children.clone()),
                    ),
            )
            .maybe_child(bar.map(|(top, length)| {
                rect()
                    .position(Position::new_absolute().top(top).right(SCROLL_BAR.inset))
                    .layer(THUMB_LAYER)
                    .width(Size::px(SCROLL_BAR.thumb))
                    .height(Size::px(length))
                    .corner_radius(SCROLL_BAR.thumb / 2.0)
                    .background(color(Rgba {
                        a: SCROLL_BAR.opacity,
                        ..ink
                    }))
            }))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_that_fits_has_no_thumb() {
        assert_eq!(thumb(200.0, 200.0, 0.0), None);
        assert_eq!(thumb(0.0, 500.0, 0.0), None);
    }

    #[test]
    fn the_thumb_spans_the_track_from_top_to_bottom() {
        let (top, length) = thumb(200.0, 800.0, 0.0).unwrap();
        assert_eq!((top, length), (0.0, 50.0));
        let (bottom, _) = thumb(200.0, 800.0, -600.0).unwrap();
        assert_eq!(bottom, 150.0);
    }
}
