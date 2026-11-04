//! Pop-up button: shows the current choice and opens a menu with the others.

use freya_components::svg_viewer::SvgViewer;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Size};
use vow_design::{MENU_POPOVER, SETTINGS, SYMBOL, TEXT};

use crate::choices::use_choice;
use crate::interaction::use_interaction;
use crate::menu::{CheckState, MenuIcon, MenuItem, MenuView};
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupSpec, Popups,
};
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::{SurfaceKind, color, popover_palette};

#[derive(PartialEq)]
pub struct PopUpButton {
    label: &'static str,
    items: Vec<(&'static str, Option<MenuIcon>)>,
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
        self.items.push((label, None));
        self
    }

    pub fn icon_item(mut self, label: &'static str, icon: MenuIcon) -> Self {
        self.items.push((label, Some(icon)));
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
        let palette = Palette::current();
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
            .maybe_child(
                self.items
                    .get(current)
                    .and_then(|item| match &item.1 {
                        Some(MenuIcon::Svg(svg)) => Some(svg.clone()),
                        _ => None,
                    })
                    .map(|svg| {
                        SvgViewer::new(Bytes::from(svg.as_bytes().to_vec()))
                            .width(Size::px(SYMBOL.grid))
                            .height(Size::px(SYMBOL.grid))
                            .show_loader(false)
                    }),
            )
            .child(text(
                self.items
                    .get(current)
                    .map(|item| item.0)
                    .unwrap_or_default(),
                TEXT,
                palette.label,
            ))
            .child(
                rect()
                    .width(Size::px(chevron))
                    .height(Size::px(chevron))
                    .corner_radius(chevron / 2.0)
                    .center()
                    .background(color(palette.control_background))
                    .child(symbol(
                        Symbol::ChevronUpDown,
                        SETTINGS.dropdown_chevron_glyph as f32,
                        palette.label,
                    )),
            );
        interaction.focus_ring(button, chevron / 2.0)
    }
}

fn open_menu(
    items: &[(&str, Option<MenuIcon>)],
    current: usize,
    area: Area,
    selects: Vec<Callback<(), ()>>,
) {
    let entries: Vec<MenuItem> = items
        .iter()
        .enumerate()
        .map(|(index, (label, icon))| MenuItem::Action {
            id: index.to_string(),
            label: label.to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Radio(index == current),
            icon: icon.clone(),
        })
        .collect();
    open_popover_menu(
        entries,
        (
            area.origin.x.round() as i32,
            area.origin.y.round() as i32,
            area.size.width.round() as u32,
            area.size.height.round() as u32,
        ),
        Some(current),
        move |id| {
            if let Some(select) = id.parse::<usize>().ok().and_then(|i| selects.get(i)) {
                select.call(());
            }
        },
    );
}

pub(crate) fn open_popover_menu(
    entries: Vec<MenuItem>,
    anchor_rect: (i32, i32, u32, u32),
    over: Option<usize>,
    on_activate: impl Fn(String) + 'static,
) {
    let palette = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
    let size = MenuView::preferred_size(&entries, palette);
    let offset = over.map_or((0, 0), |index| {
        let above: f32 = entries[..index]
            .iter()
            .map(|item| item.height(palette))
            .sum();
        let inset =
            MENU_POPOVER.padding + crate::menu::last_leading_column_inset(&entries, palette);
        (
            -(inset.round() as i32),
            -((MENU_POPOVER.padding + above).round() as i32),
        )
    });
    let view = MenuView::new(
        entries,
        move |id| {
            on_activate(id);
            PopupHandle::get().close();
        },
        || PopupHandle::get().close(),
    )
    .with_surface(SurfaceKind::Toplevel)
    .with_popover_metrics();
    let spec = PopupSpec {
        anchor_rect,
        anchor: PopupAnchor::TopLeft,
        gravity: PopupGravity::BottomRight,
        offset,
        size: (size.0.ceil() as u32, size.1.ceil() as u32),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_x: true,
            flip_y: over.is_none(),
            slide_x: true,
            slide_y: true,
            resize_y: false,
        },
        grab: true,
        on_dismissed: None,
    };
    Popups::get().open(spec, AppComponent::new(view));
}
