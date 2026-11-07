use std::cell::Cell;
use std::rc::Rc;

use ctdesign::MENU_POPOVER;
use freya_components::svg_viewer::SvgViewer;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Size};

use crate::choices::use_choice;
use crate::controls::{dimmed, shadowed};
use crate::interaction::{control_state, use_interaction};
use crate::menu::{CheckState, MenuIcon, MenuItem, MenuView};
use crate::platform::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupSpec, Popups};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{LayoutDirection, Paints, PopUpButtonStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};
use crate::theme::SurfaceKind;

#[derive(PartialEq)]
pub struct PopUpButton {
    label: &'static str,
    items: Vec<(&'static str, Option<MenuIcon>)>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
}

impl PopUpButton {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            selected: 0,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(PopUpButtonStyle) -> PopUpButtonStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: PopUpButtonStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let mut area = use_state(Area::default);
        let current = (0..self.items.len()).find(|&i| choice.is(i)).unwrap_or(0);
        let items = self.items.clone();
        let selects: Vec<_> = (0..items.len()).map(|i| choice.select(i)).collect();
        let size = style.indicator_size;
        let radius = style
            .indicator_shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let indicator = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .center()
            .painted(&paints, &style.indicator)
            .bordered(&paints, style.indicator_border.as_ref())
            .child(symbol(
                Symbol::ChevronUpDown,
                style.chevron_size,
                paints.color(style.chevron.resolve(state)),
            ));

        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ComboBox)
            .a11y_alt(self.label)
            .height(Size::px(style.height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.gap)
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
                            .width(Size::px(style.icon_size))
                            .height(Size::px(style.icon_size))
                            .show_loader(false)
                    }),
            )
            .child(
                with_font(
                    label().text(
                        self.items
                            .get(current)
                            .map(|item| item.0)
                            .unwrap_or_default(),
                    ),
                    style.font,
                )
                .max_lines(1)
                .color(paints.fill(&style.label)),
            )
            .child(shadowed(indicator, &style.indicator_shadows));
        interaction.focus_ring(dimmed(button, style.opacity), radius.top_left)
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
        .map(|(index, (label, icon))| {
            MenuItem::action(index.to_string(), *label)
                .with_check(CheckState::Radio(index == current))
                .with_icon(icon.clone())
        })
        .collect();
    open_popover_menu(
        entries,
        PopupAnchor::TopLeft,
        crate::platform::anchor_rect(area),
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
    anchor: PopupAnchor,
    anchor_rect: (i32, i32, u32, u32),
    over: Option<usize>,
    on_activate: impl Fn(String) + 'static,
) {
    let palette = crate::styles::popover_style();
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
    let popups = Popups::get();
    let root = Rc::new(Cell::new(None));
    let close_root = {
        let (popups, root) = (popups.clone(), root.clone());
        move || {
            if let Some(id) = root.get() {
                popups.close(id);
            }
        }
    };
    let view = MenuView::new(entries, on_activate, close_root)
        .with_surface(SurfaceKind::Toplevel)
        .with_popover_metrics();
    let spec = PopupSpec {
        anchor_rect,
        anchor,
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
    root.set(Some(popups.open(spec, AppComponent::new(view))));
}
