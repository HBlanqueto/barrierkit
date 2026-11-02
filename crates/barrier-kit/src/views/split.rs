//! Split view: resizable sidebar, detail and optional inspector.

use freya_core::prelude::*;
use torin::prelude::{Content, Direction, Gaps, Position, Size};
use vow_appearance::SidebarStyle;
use vow_design::{SIDEBAR, SPLIT_VIEW};

use crate::style::Palette;
use crate::theme::color;

#[derive(PartialEq)]
pub struct SplitView {
    sidebar: Element,
    detail: Element,
    inspector: Option<Element>,
    sidebar_visible: bool,
    key: DiffKey,
}

impl SplitView {
    pub fn new(sidebar: impl IntoElement, detail: impl IntoElement) -> Self {
        Self {
            sidebar: sidebar.into_element(),
            detail: detail.into_element(),
            inspector: None,
            sidebar_visible: true,
            key: DiffKey::None,
        }
    }

    pub fn inspector(mut self, inspector: Option<Element>) -> Self {
        self.inspector = inspector;
        self
    }

    pub fn sidebar_visible(mut self, visible: bool) -> Self {
        self.sidebar_visible = visible;
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SplitView {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

fn clamp_sidebar(width: f32) -> f32 {
    width.clamp(SPLIT_VIEW.sidebar_min, SPLIT_VIEW.sidebar_max)
}

impl Component for SplitView {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let palette = Palette::current();
        let island = resolved.sidebar.style == SidebarStyle::Island;
        let inset = if island { SIDEBAR.inset as f32 } else { 0.0 };
        let mut window_width = use_state(|| f32::MAX);
        let mut sidebar_width = use_state(|| SIDEBAR.width as f32);
        let mut dragging = use_state(|| false);
        let narrow = *window_width.read() < SPLIT_VIEW.collapse_below;
        let width = *sidebar_width.read();
        let container = width + 2.0 * inset;

        let sidebar = || {
            rect()
                .width(Size::px(container))
                .height(Size::fill())
                .padding(Gaps::new_all(inset))
                .child(self.sidebar.clone())
        };

        let mut root = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .on_sized(move |event: Event<SizedEventData>| {
                window_width.set_if_modified(event.area.size.width);
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                if *dragging.read() {
                    sidebar_width.set(clamp_sidebar(event.global_location().x as f32 - inset));
                }
            })
            .on_global_pointer_press(move |_| dragging.set_if_modified(false));

        if self.sidebar_visible && !narrow {
            root = root.child(
                sidebar().child(
                    rect()
                        .a11y_role(AccessibilityRole::Splitter)
                        .position(
                            Position::new_absolute()
                                .left(container - SPLIT_VIEW.grip / 2.0)
                                .top(0.0),
                        )
                        .width(Size::px(SPLIT_VIEW.grip))
                        .height(Size::fill())
                        .on_mouse_down(move |_| dragging.set(true)),
                ),
            );
        }
        root = root.child(
            rect()
                .width(Size::flex(1.0))
                .height(Size::fill())
                .child(self.detail.clone()),
        );
        if let Some(inspector) = &self.inspector {
            root = root
                .child(
                    rect()
                        .width(Size::px(1.0))
                        .height(Size::fill())
                        .background(color(palette.separator)),
                )
                .child(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .height(Size::fill())
                        .child(inspector.clone()),
                );
        }
        if self.sidebar_visible && narrow {
            root = root.child(
                sidebar()
                    .position(Position::new_absolute().left(0.0).top(0.0))
                    .layer(Layer::Overlay),
            );
        }
        root.content(Content::flex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_width_stays_between_its_minimum_and_maximum() {
        assert_eq!(clamp_sidebar(0.0), SPLIT_VIEW.sidebar_min);
        assert_eq!(clamp_sidebar(10_000.0), SPLIT_VIEW.sidebar_max);
        assert_eq!(clamp_sidebar(SIDEBAR.width as f32), SIDEBAR.width as f32);
    }
}
