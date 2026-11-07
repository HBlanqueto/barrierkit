use crate::scroll::ScrollView;
use ctdesign::{BODY, CAPTION, LIST_GROUP, Rgba, SETTINGS, SIDEBAR, TEXT_FIELD};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::interaction::use_interaction;
use crate::sidebar::symbol_tile;
use crate::style::{Palette, text};
use crate::symbol::Symbol;
use crate::theme::{SurfaceKind, color, popover_palette};

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
    fn height(&self) -> f32 {
        match self {
            Self::Header(_) => SETTINGS.search_header_height as f32,
            Self::Result { subtitle: None, .. } => SIDEBAR.item_height as f32,
            Self::Result { .. } => SETTINGS.search_result_height as f32,
        }
    }
}

pub(crate) fn search_popover(
    entries: Vec<Entry>,
    accent_highlight: bool,
    active: Option<usize>,
    mut hovered: State<bool>,
) -> Rect {
    let menu = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
    let content: f32 = entries.iter().map(Entry::height).sum();
    let height = content.min(SETTINGS.search_popover_max_height as f32);
    let mut results = 0..;
    let rows = entries.into_iter().map(|entry| match entry {
        Entry::Header(title) => rect()
            .height(Size::px(SETTINGS.search_header_height as f32))
            .padding(Gaps::new(
                0.0,
                menu.item_padding_x,
                0.0,
                menu.item_padding_x,
            ))
            .main_align(Alignment::Center)
            .child(text(title, CAPTION, Palette::current().secondary_label))
            .into_element(),
        result => SearchResult {
            entry: result,
            accent_highlight,
            active: results.next() == active,
        }
        .into_element(),
    });
    rect()
        .position(
            Position::new_absolute()
                .left(0.0)
                .top(TEXT_FIELD.height + SETTINGS.search_popover_gap as f32),
        )
        .layer(Layer::Overlay)
        .width(Size::fill())
        .padding(Gaps::new_all(menu.padding))
        .corner_radius(LIST_GROUP.radius)
        .background(color(menu.bg))
        .border(
            Border::new()
                .width(ctdesign::HAIRLINE)
                .alignment(BorderAlignment::Outer)
                .fill(color(menu.separator_color)),
        )
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
        let palette = Palette::current();
        let menu = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
        let interaction = use_interaction();
        let highlighted = self.active || interaction.hovered();
        let (background, ink, secondary) = match (highlighted, self.accent_highlight) {
            (true, true) => (Some(palette.accent), palette.on_accent, palette.on_accent),
            (true, false) => (
                Some(
                    crate::theme::resolved()
                        .sidebar
                        .palette
                        .inactive_selected_bg,
                ),
                palette.label,
                palette.secondary_label,
            ),
            (false, _) => (None, palette.label, palette.secondary_label),
        };
        let row = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ListItem)
            .a11y_alt(title)
            .width(Size::fill())
            .height(Size::px(self.entry.height()))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(SIDEBAR.item_gap as f32)
            .padding(Gaps::new(
                0.0,
                menu.item_padding_x,
                0.0,
                menu.item_padding_x,
            ))
            .corner_radius(menu.item_radius)
            .map(background, |el, background| {
                el.background(color(background))
            })
            .on_press(move |_| on_select.call(()))
            .child(symbol_tile(symbol, tint, None))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .direction(Direction::Vertical)
                    .child(text(title, BODY, ink).text_overflow(TextOverflow::Ellipsis))
                    .maybe_child(subtitle.map(|subtitle| text(subtitle, CAPTION, secondary))),
            );
        interaction.focus_ring(row, menu.item_radius)
    }
}
