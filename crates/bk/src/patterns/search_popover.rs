use crate::scroll::ScrollView;
use ctdesign::Rgba;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::interaction::{control_state, use_interaction};
use crate::sidebar::SymbolTile;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, LayoutDirection, Paints, SearchFieldStyle, SettingsWindowStyle, Theme, resolve,
};
use crate::symbol::Symbol;
use crate::theme::color;

#[derive(Clone, PartialEq)]
pub(crate) enum Entry {
    Header(&'static str),
    Result {
        symbol: Symbol,
        tint: Rgba,
        title: &'static str,
        subtitle: Option<&'static str>,
        on_select: Callback<(), ()>,
    },
}

impl Entry {
    fn height(&self, style: &SettingsWindowStyle) -> f32 {
        match self {
            Self::Header(_) => style.header_height,
            Self::Result { subtitle: None, .. } => style.result_height,
            Self::Result { .. } => style.result_subtitle_height,
        }
    }
}

pub(crate) fn search_popover(
    entries: Vec<Entry>,
    accent_highlight: bool,
    active: Option<usize>,
    mut hovered: State<bool>,
    theme: &Theme,
) -> Rect {
    let style: SettingsWindowStyle = resolve(theme);
    let field: SearchFieldStyle = resolve(&Theme::tengen());
    let state = ControlState::default();
    let paints = Paints::current(state, style.effect);
    let content: f32 = entries.iter().map(|entry| entry.height(&style)).sum();
    let height = content.min(style.popover_max_height);
    let padding_x = style.result_padding_x;
    let mut results = 0..;
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
        result => SearchResult {
            entry: result,
            accent_highlight,
            active: results.next() == active,
            style: theme.clone(),
        }
        .into_element(),
    });
    rect()
        .position(
            Position::new_absolute()
                .left(0.0)
                .top(field.height + style.popover_gap),
        )
        .layer(Layer::Overlay)
        .width(Size::fill())
        .padding(Gaps::new_all(style.popover_padding))
        .corner_radius(
            style
                .popover_shape
                .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight),
        )
        .painted(&paints, &style.popover_background)
        .map(style.popover_border.as_ref(), |el, (paint, width)| {
            el.border(
                Border::new()
                    .width(*width)
                    .alignment(BorderAlignment::Outer)
                    .fill(color(paints.color(paint))),
            )
        })
        .on_pointer_enter(move |_| hovered.set(true))
        .on_pointer_leave(move |_| hovered.set(false))
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::px(height))
                .children(rows),
        )
}

#[derive(PartialEq)]
struct SearchResult {
    entry: Entry,
    accent_highlight: bool,
    active: bool,
    style: Theme,
}

impl Component for SearchResult {
    fn render(&self) -> impl IntoElement {
        let Entry::Result {
            symbol,
            tint,
            title,
            subtitle,
            on_select,
        } = self.entry.clone()
        else {
            return rect();
        };
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
        let height = self.entry.height(&style);
        let radius = style
            .result_shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);
        let row = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ListItem)
            .a11y_alt(title)
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
            .child(SymbolTile::new(symbol, tint))
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
        interaction.focus_ring(row, radius.top_left)
    }
}
