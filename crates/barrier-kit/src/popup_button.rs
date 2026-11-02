//! Pop-up button: shows the current choice and opens a menu with the others.

use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Size};
use vow_design::{MENU_POPOVER, SETTINGS, TEXT};

use crate::choices::use_choice;
use crate::interaction::use_interaction;
use crate::menu::{CheckState, MenuItem, MenuView};
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupSpec, Popups,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::{SurfaceKind, color, popover_palette};

#[derive(PartialEq)]
pub struct PopUpButton {
    label: &'static str,
    items: Vec<&'static str>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
}

impl PopUpButton {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            selected: 0,
            on_change: None,
        }
    }

    pub fn item(mut self, label: &'static str) -> Self {
        self.items.push(label);
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for PopUpButton {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let interaction = use_interaction();
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let mut area = use_state(Area::default);
        let current = (0..self.items.len()).find(|&i| choice.is(i)).unwrap_or(0);
        let items = self.items.clone();
        let selects: Vec<_> = (0..items.len()).map(|i| choice.select(i)).collect();
        let chevron = SETTINGS.popup_chevron as f32;

        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ComboBox)
            .a11y_alt(self.label)
            .height(Size::px(SETTINGS.popup_height as f32))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SETTINGS.popup_text_gap as f32)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| open_menu(&items, current, *area.read(), selects.clone()))
            .child(
                label()
                    .text(self.items.get(current).copied().unwrap_or_default())
                    .font_family(TEXT.family)
                    .font_size(TEXT.size as f32)
                    .font_weight(TEXT.weight as i32)
                    .max_lines(1)
                    .color(color(base.ink_strong)),
            )
            .child(
                rect()
                    .width(Size::px(chevron))
                    .height(Size::px(chevron))
                    .corner_radius(chevron / 2.0)
                    .center()
                    .background(color(resolved.surfaces.control))
                    .child(symbol(
                        Symbol::ChevronUpDown,
                        SETTINGS.dropdown_chevron_glyph as f32,
                        base.ink_strong,
                    )),
            );
        interaction.focus_ring(button, chevron / 2.0)
    }
}

fn open_menu(items: &[&str], current: usize, area: Area, selects: Vec<Callback<(), ()>>) {
    let entries: Vec<MenuItem> = items
        .iter()
        .enumerate()
        .map(|(index, label)| MenuItem::Action {
            id: index.to_string(),
            label: label.to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Radio(index == current),
            icon: None,
        })
        .collect();
    let palette = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
    let size = MenuView::preferred_size(&entries, palette);
    let view = MenuView::new(
        entries,
        move |id| {
            if let Some(select) = id.parse::<usize>().ok().and_then(|i| selects.get(i)) {
                select.call(());
            }
            PopupHandle::get().close();
        },
        || PopupHandle::get().close(),
    )
    .with_surface(SurfaceKind::Toplevel)
    .with_popover_metrics();
    let inset =
        MENU_POPOVER.padding + MENU_POPOVER.item_padding_start + MENU_POPOVER.leading_column;
    let spec = PopupSpec {
        anchor_rect: (
            area.origin.x.round() as i32,
            area.origin.y.round() as i32,
            area.size.width.round() as u32,
            area.size.height.round() as u32,
        ),
        anchor: PopupAnchor::TopLeft,
        gravity: PopupGravity::BottomRight,
        offset: (
            -(inset.round() as i32),
            -(MENU_POPOVER.padding.round() as i32),
        ),
        size: (size.0.ceil() as u32, size.1.ceil() as u32),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_x: true,
            flip_y: true,
            slide_x: true,
            slide_y: true,
            resize_y: false,
        },
        grab: true,
        on_dismissed: None,
    };
    Popups::get().open(spec, AppComponent::new(view));
}
