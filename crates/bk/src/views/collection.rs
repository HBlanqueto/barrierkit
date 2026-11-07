use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::interaction::{control_state, track_modifiers, use_held_modifiers, use_interaction};
use crate::style::text;
use crate::styles::{CollectionViewStyle, LayoutDirection, Paints, Theme, resolve};

#[derive(PartialEq)]
pub struct CollectionView {
    label: &'static str,
    items: Vec<(Element, String)>,
    selected: Vec<usize>,
    on_select: Option<Callback<Vec<usize>, ()>>,
    style: Theme,
}

impl CollectionView {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            selected: Vec::new(),
            on_select: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(CollectionViewStyle) -> CollectionViewStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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

pub fn select(current: &[usize], index: usize, held: Modifiers) -> Vec<usize> {
    if held.contains(Modifiers::SHIFT)
        && let Some(&anchor) = current.last()
    {
        let mut range: Vec<usize> = if anchor <= index {
            (anchor + 1..=index).collect()
        } else {
            (index..anchor).rev().collect()
        };
        range.push(anchor);
        return range;
    }
    if !held.intersects(Modifiers::CONTROL | Modifiers::META) {
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
        let style: CollectionViewStyle = resolve(&self.style);
        let held = use_held_modifiers();
        let tiles = self.items.iter().enumerate().map(|(index, (icon, name))| {
            let selected = self.selected.clone();
            let on_select = self.on_select.clone();
            let is_selected = selected.contains(&index);
            Tile {
                icon: icon.clone(),
                name: name.clone(),
                selected: is_selected,
                style: self.style.clone(),
                on_press: Callback::new(move |_| {
                    let held = *held.read();
                    if (!is_selected || !held.is_empty())
                        && let Some(cb) = &on_select
                    {
                        cb.call(select(&selected, index, held));
                    }
                }),
            }
            .into_element()
        });
        track_modifiers(rect(), held)
            .a11y_role(AccessibilityRole::Grid)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::wrap_spacing(style.spacing))
            .spacing(style.spacing)
            .children(tiles)
    }
}

#[derive(PartialEq)]
struct Tile {
    icon: Element,
    name: String,
    selected: bool,
    on_press: Callback<(), ()>,
    style: Theme,
}

impl Component for Tile {
    fn render(&self) -> impl IntoElement {
        let style: CollectionViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let selected = self.selected;
        let on_press = self.on_press.clone();
        let (icon_background, label_background, label) = if selected {
            (
                &style.selected_icon_background,
                &style.selected_label_background,
                &style.selected_label,
            )
        } else {
            (
                &style.icon_background,
                &style.label_background,
                &style.label,
            )
        };
        let direction = LayoutDirection::LeftToRight;
        let radius = style
            .shape
            .corner_radius(style.tile_width, style.tile_width, direction);
        let icon_radius = style
            .shape
            .corner_radius(style.icon_size, style.icon_size, direction);
        let line = style.font.line_height as f32;
        let label_radius = style.label_shape.corner_radius(line, line, direction);
        let tile = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::GridCell)
            .a11y_alt(self.name.clone())
            .a11y_builder(move |node| node.set_selected(selected))
            .width(Size::px(style.tile_width))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .padding(Gaps::new_all(style.padding))
            .spacing(style.label_spacing)
            .corner_radius(radius)
            .on_press(move |_| on_press.call(()))
            .child(
                rect()
                    .width(Size::px(style.icon_size))
                    .height(Size::px(style.icon_size))
                    .center()
                    .corner_radius(icon_radius)
                    .background(paints.fill(icon_background))
                    .map(paints.blur(icon_background), Rect::blur)
                    .child(self.icon.clone()),
            )
            .child(
                rect()
                    .padding(Gaps::new(
                        0.0,
                        style.label_padding_x,
                        0.0,
                        style.label_padding_x,
                    ))
                    .corner_radius(label_radius)
                    .background(paints.fill(label_background))
                    .map(paints.blur(label_background), Rect::blur)
                    .child(text(
                        self.name.clone(),
                        style.font,
                        paints.color(label.resolve(state)),
                    )),
            );
        interaction.focus_ring(tile, radius.top_left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_selection_replaces_and_ctrl_selection_toggles() {
        assert_eq!(select(&[1, 2], 3, Modifiers::empty()), [3]);
        assert_eq!(select(&[1, 2], 3, Modifiers::CONTROL), [1, 2, 3]);
        assert_eq!(select(&[1, 2], 2, Modifiers::CONTROL), [1]);
    }

    #[test]
    fn shift_selects_the_range_from_the_last_click_which_stays_the_anchor() {
        assert_eq!(select(&[2], 5, Modifiers::SHIFT), [3, 4, 5, 2]);
        assert_eq!(select(&[3, 4, 5, 2], 0, Modifiers::SHIFT), [1, 0, 2]);
        assert_eq!(select(&[], 4, Modifiers::SHIFT), [4]);
    }
}
