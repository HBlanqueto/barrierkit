use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{CAPTION, COLLECTION, Rgba};

use crate::interaction::use_interaction;
use crate::platform::WindowControls;
use crate::style::{Palette, text};
use crate::theme::color;

#[derive(PartialEq)]
pub struct CollectionView {
    label: &'static str,
    items: Vec<(Element, String)>,
    selected: Vec<usize>,
    on_select: Option<Callback<Vec<usize>, ()>>,
}

impl CollectionView {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            selected: Vec::new(),
            on_select: None,
        }
    }

    pub fn item(mut self, icon: impl IntoElement, name: impl Into<String>) -> Self {
        self.items.push((icon.into_element(), name.into()));
        self
    }

    pub fn selected(mut self, selected: Vec<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<usize>, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

pub fn select(current: &[usize], index: usize, additive: bool) -> Vec<usize> {
    if !additive {
        return vec![index];
    }
    let mut next: Vec<usize> = current.iter().copied().filter(|&i| i != index).collect();
    if next.len() == current.len() {
        next.push(index);
    }
    next
}

impl Component for CollectionView {
    fn render(&self) -> impl IntoElement {
        let mut ctrl = use_state(|| false);
        let status = try_consume_root_context::<WindowControls>().map(|controls| controls.status);
        use_side_effect(move || {
            if status.is_some_and(|status| !status.read().activated) {
                ctrl.set_if_modified(false);
            }
        });
        let tiles = self.items.iter().enumerate().map(|(index, (icon, name))| {
            let selected = self.selected.clone();
            let on_select = self.on_select.clone();
            let is_selected = selected.contains(&index);
            Tile {
                icon: icon.clone(),
                name: name.clone(),
                selected: is_selected,
                on_press: Callback::new(move |_| {
                    if (!is_selected || *ctrl.read())
                        && let Some(cb) = &on_select
                    {
                        cb.call(select(&selected, index, *ctrl.read()));
                    }
                }),
            }
            .into_element()
        });
        rect()
            .a11y_role(AccessibilityRole::Grid)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::wrap_spacing(COLLECTION.gap))
            .spacing(COLLECTION.gap)
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Control) {
                    ctrl.set(true);
                }
            })
            .on_global_key_up(move |event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Control) {
                    ctrl.set(false);
                }
            })
            .children(tiles)
    }
}

#[derive(PartialEq)]
struct Tile {
    icon: Element,
    name: String,
    selected: bool,
    on_press: Callback<(), ()>,
}

impl Component for Tile {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let selected = self.selected;
        let on_press = self.on_press.clone();
        let (name_bg, name_ink) = if selected {
            (Some(palette.accent), palette.on_accent)
        } else {
            (None, palette.label)
        };
        let tile = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::GridCell)
            .a11y_alt(self.name.clone())
            .a11y_builder(move |node| node.set_selected(selected))
            .width(Size::px(COLLECTION.tile))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .padding(Gaps::new_all(COLLECTION.gap))
            .spacing(COLLECTION.gap / 2.0)
            .corner_radius(COLLECTION.radius)
            .on_press(move |_| on_press.call(()))
            .child(
                rect()
                    .width(Size::px(COLLECTION.icon))
                    .height(Size::px(COLLECTION.icon))
                    .center()
                    .corner_radius(COLLECTION.radius)
                    .maybe(selected, |el| {
                        el.background(color(Rgba {
                            a: COLLECTION.selection,
                            ..palette.label
                        }))
                    })
                    .child(self.icon.clone()),
            )
            .child(
                rect()
                    .padding(Gaps::new(0.0, COLLECTION.gap, 0.0, COLLECTION.gap))
                    .corner_radius(COLLECTION.gap)
                    .map(name_bg, |el, background| el.background(color(background)))
                    .child(text(self.name.clone(), CAPTION, name_ink)),
            );
        interaction.focus_ring(tile, COLLECTION.radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_selection_replaces_and_ctrl_selection_toggles() {
        assert_eq!(select(&[1, 2], 3, false), [3]);
        assert_eq!(select(&[1, 2], 3, true), [1, 2, 3]);
        assert_eq!(select(&[1, 2], 2, true), [1]);
    }
}
