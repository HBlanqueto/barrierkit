use ctdesign::{BODY, PATH_CONTROL};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};

#[derive(PartialEq)]
pub struct PathControl {
    label: &'static str,
    items: Vec<(Symbol, String)>,
    on_select: Option<Callback<usize, ()>>,
}

impl PathControl {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            on_select: None,
        }
    }

    pub fn item(mut self, icon: Symbol, name: impl Into<String>) -> Self {
        self.items.push((icon, name.into()));
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

impl Component for PathControl {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let last = self.items.len().saturating_sub(1);
        let crumbs = self
            .items
            .iter()
            .enumerate()
            .flat_map(|(index, (icon, item))| {
                let on_select = self.on_select.clone();
                let crumb = rect()
                    .a11y_role(AccessibilityRole::Link)
                    .a11y_alt(item.clone())
                    .maybe(on_select.is_some(), |el| {
                        el.on_press(move |_| {
                            if let Some(on_select) = &on_select {
                                on_select.call(index);
                            }
                        })
                    })
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(PATH_CONTROL.icon_gap)
                    .child(symbol(*icon, PATH_CONTROL.icon, palette.secondary_label))
                    .child(text(item.clone(), BODY, palette.label))
                    .into_element();
                let separator = (index < last).then(|| {
                    symbol(
                        Symbol::ChevronRight,
                        PATH_CONTROL.chevron,
                        palette.tertiary_label,
                    )
                    .into_element()
                });
                std::iter::once(crumb).chain(separator)
            });
        rect()
            .a11y_role(AccessibilityRole::Navigation)
            .a11y_alt(self.label)
            .height(Size::px(PATH_CONTROL.height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(PATH_CONTROL.gap)
            .children(crumbs)
    }
}
