use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

use crate::controls::dimmed;
use crate::style::with_font;
use crate::styles::{ControlState, Paints, PathControlStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};

#[derive(PartialEq)]
pub struct PathControl {
    label: &'static str,
    items: Vec<(Symbol, String)>,
    on_select: Option<Callback<usize, ()>>,
    style: Theme,
}

impl PathControl {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            on_select: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(PathControlStyle) -> PathControlStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: PathControlStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let icon_ink = paints.color(style.icon.resolve(state));
        let chevron_ink = paints.color(style.chevron.resolve(state));
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
                    .spacing(style.icon_gap)
                    .child(symbol(*icon, style.icon_size, icon_ink))
                    .child(
                        with_font(label().text(item.clone()), style.font)
                            .max_lines(1)
                            .color(paints.fill(&style.label)),
                    )
                    .into_element();
                let separator = (index < last).then(|| {
                    symbol(Symbol::ChevronRight, style.chevron_size, chevron_ink).into_element()
                });
                std::iter::once(crumb).chain(separator)
            });
        dimmed(
            rect()
                .a11y_role(AccessibilityRole::Navigation)
                .a11y_alt(self.label)
                .height(Size::px(style.height))
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(style.gap)
                .children(crumbs),
            style.opacity,
        )
    }
}
