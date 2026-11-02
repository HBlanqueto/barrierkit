//! Browser window (a Finder-like layout): sidebar, toolbar, content and inspector.

use freya_core::prelude::*;
use torin::prelude::{Direction, Size};

use crate::titlebar::Titlebar;
use crate::views::split::SplitView;

#[derive(PartialEq)]
pub struct BrowserWindow {
    title: String,
    sidebar: Element,
    leading: Element,
    trailing: Element,
    content: Element,
    inspector: Option<Element>,
    sidebar_visible: bool,
}

impl BrowserWindow {
    pub fn new(
        title: impl Into<String>,
        sidebar: impl IntoElement,
        content: impl IntoElement,
    ) -> Self {
        Self {
            title: title.into(),
            sidebar: sidebar.into_element(),
            leading: rect().into_element(),
            trailing: rect().into_element(),
            content: content.into_element(),
            inspector: None,
            sidebar_visible: true,
        }
    }

    pub fn toolbar(mut self, leading: impl IntoElement, trailing: impl IntoElement) -> Self {
        self.leading = leading.into_element();
        self.trailing = trailing.into_element();
        self
    }

    pub fn inspector(mut self, inspector: Option<Element>) -> Self {
        self.inspector = inspector;
        self
    }

    pub fn sidebar_visible(mut self, visible: bool) -> Self {
        self.sidebar_visible = visible;
        self
    }
}

impl Component for BrowserWindow {
    fn render(&self) -> impl IntoElement {
        let detail = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(
                Titlebar::new()
                    .with_sidebar(self.sidebar_visible)
                    .title(self.title.clone())
                    .left(self.leading.clone())
                    .right(self.trailing.clone()),
            )
            .child(self.content.clone());
        SplitView::new(self.sidebar.clone(), detail)
            .sidebar_visible(self.sidebar_visible)
            .inspector(self.inspector.clone())
    }
}
