use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use crate::scroll::ScrollView;
use ctdesign::Rgba;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Content, Direction, Gaps, Size};

use crate::interaction::{control_state, use_interaction};
use crate::layout::ambient;
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupId, PopupSpec, Popups, anchor_rect,
};
use crate::sidebar::SymbolTile;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{ControlState, Paints, SettingsWindowStyle, Theme, resolve};
use crate::symbol::Symbol;

#[derive(Clone, PartialEq)]
pub(crate) enum Entry {
    Header(&'static str),
    Result {
        symbol: Symbol,
        tint: Rgba,
        title: Cow<'static, str>,
        subtitle: Option<Cow<'static, str>>,
        on_select: Callback<(), ()>,
    },
}

impl Entry {
    fn height(&self, style: &SettingsWindowStyle) -> f32 {
        match self {
            Self::Header(_) => style.header_height,
            Self::Result { subtitle, .. } => result_height(style, subtitle.as_deref()),
        }
    }
}

fn result_height(style: &SettingsWindowStyle, subtitle: Option<&str>) -> f32 {
    match subtitle {
        None => style.result_height,
        Some(_) => style.result_subtitle_height,
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct SearchResults {
    pub(crate) entries: Vec<Entry>,
    pub(crate) accent_highlight: bool,
    pub(crate) active: Option<usize>,
}

type Placement = ((i32, i32, u32, u32), (u32, u32));

fn popover_height(entries: &[Entry], style: &SettingsWindowStyle) -> f32 {
    let content: f32 = entries.iter().map(|entry| entry.height(style)).sum();
    content.min(style.popover_max_height)
}

fn close(popups: &Popups, shown: &Cell<Option<(PopupId, Placement)>>) {
    if let Some((id, _)) = shown.take() {
        popups.close(id);
    }
}

pub(crate) fn use_search_popover(
    results: Option<SearchResults>,
    field: Area,
    theme: &Theme,
    on_dismissed: impl Fn() + 'static,
) {
    let popups: Option<Popups> = try_consume_root_context();
    let shown = use_hook(|| Rc::new(Cell::new(None::<(PopupId, Placement)>)));
    let mut content = use_state(SearchResults::default);
    let (owned_popups, owned_shown) = (popups.clone(), shown.clone());
    use_drop(move || {
        if let Some(popups) = &owned_popups {
            close(popups, &owned_shown);
        }
    });
    let Some(popups) = popups else {
        return;
    };
    let style: SettingsWindowStyle = resolve(theme);
    let Some(results) = results.filter(|_| field.size.width > 0.0) else {
        close(&popups, &shown);
        return;
    };
    let height = popover_height(&results.entries, &style) + 2.0 * style.popover_padding;
    let placement = (
        anchor_rect(field),
        (field.size.width.round() as u32, height.ceil() as u32),
    );
    content.set(results);
    let spec = |on_dismissed| PopupSpec {
        anchor_rect: placement.0,
        anchor: PopupAnchor::BottomLeft,
        gravity: PopupGravity::BottomRight,
        offset: (0, style.popover_gap.round() as i32),
        size: placement.1,
        constraint_adjustment: PopupConstraintAdjustment {
            slide_x: true,
            slide_y: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: false,
        on_dismissed,
    };
    match shown.get() {
        Some((_, placed)) if placed == placement => {}
        Some((id, _)) => {
            popups.reposition(id, spec(None));
            shown.set(Some((id, placement)));
        }
        None => {
            let opened = Rc::new(Cell::new(None));
            let dismissed: Rc<dyn Fn()> = {
                let (shown, opened) = (shown.clone(), opened.clone());
                Rc::new(move || {
                    if shown.get().map(|(id, _)| id) == opened.get() {
                        shown.set(None);
                        on_dismissed();
                    }
                })
            };
            let view = AppComponent::new(SearchPopover {
                results: content,
                theme: theme.clone(),
            });
            let id = popups.open(spec(Some(dismissed)), view);
            opened.set(Some(id));
            shown.set(Some((id, placement)));
        }
    }
}

struct SearchPopover {
    results: State<SearchResults>,
    theme: Theme,
}

impl App for SearchPopover {
    fn render(&self) -> impl IntoElement {
        let style: SettingsWindowStyle = resolve(&self.theme);
        let SearchResults {
            entries,
            accent_highlight,
            active,
        } = (*self.results.read()).clone();
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let height = popover_height(&entries, &style);
        let padding_x = style.result_padding_x;
        let mut index = 0..;
        let rows = entries.into_iter().map(|entry| match entry {
            Entry::Header(title) => rect()
                .height(Size::px(style.header_height))
                .padding(Gaps::new(0.0, padding_x, 0.0, padding_x))
                .main_align(Alignment::Center)
                .child(text(
                    title,
                    style.header_font,
                    paints.color(style.header.resolve(state)),
                ))
                .into_element(),
            Entry::Result {
                symbol,
                tint,
                title,
                subtitle,
                on_select,
            } => SearchResult {
                symbol,
                tint,
                title,
                subtitle,
                on_select,
                accent_highlight,
                active: index.next() == active,
                style: self.theme.clone(),
            }
            .into_element(),
        });
        rect()
            .expanded()
            .padding(Gaps::new_all(style.popover_padding))
            .painted(&paints, &style.popover_background)
            .child(
                ScrollView::new()
                    .width(Size::fill())
                    .height(Size::px(height))
                    .children(rows),
            )
    }
}

#[derive(PartialEq)]
struct SearchResult {
    symbol: Symbol,
    tint: Rgba,
    title: Cow<'static, str>,
    subtitle: Option<Cow<'static, str>>,
    on_select: Callback<(), ()>,
    accent_highlight: bool,
    active: bool,
    style: Theme,
}

impl Component for SearchResult {
    fn render(&self) -> impl IntoElement {
        let (title, subtitle) = (self.title.clone(), self.subtitle.clone());
        let on_select = self.on_select.clone();
        let style: SettingsWindowStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = ControlState {
            hovered: self.active || interaction.hovered(),
            ..control_state(interaction, false)
        };
        let paints = Paints::current(state, style.effect);
        let (background, title_ink, subtitle_ink) = if self.accent_highlight {
            (
                &style.suggestion_background,
                &style.suggestion_title,
                &style.suggestion_subtitle,
            )
        } else {
            (
                &style.result_background,
                &style.result_title,
                &style.result_subtitle,
            )
        };
        let height = result_height(&style, subtitle.as_deref());
        let radius = style.result_shape.corner_radius(height, height, ambient());
        let row = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ListItem)
            .a11y_alt(&*title)
            .width(Size::fill())
            .height(Size::px(height))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(style.result_spacing)
            .padding(Gaps::new(
                0.0,
                style.result_padding_x,
                0.0,
                style.result_padding_x,
            ))
            .corner_radius(radius)
            .painted(&paints, background)
            .on_press(move |_| on_select.call(()))
            .child(SymbolTile::new(self.symbol, self.tint))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .direction(Direction::Vertical)
                    .child(
                        text(
                            title,
                            style.title_font,
                            paints.color(title_ink.resolve(state)),
                        )
                        .text_overflow(TextOverflow::Ellipsis),
                    )
                    .maybe_child(subtitle.map(|subtitle| {
                        text(
                            subtitle,
                            style.subtitle_font,
                            paints.color(subtitle_ink.resolve(state)),
                        )
                    })),
            );
        interaction.focus_ring(row, radius.top_left())
    }
}
