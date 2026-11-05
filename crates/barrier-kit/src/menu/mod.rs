//! Menu control for popups, with submenus and keyboard navigation.

mod item;
mod model;
mod nav;
mod search;
mod shortcut;
mod submenu;
#[cfg(test)]
mod tests;
mod text;

pub use item::*;
pub use model::*;
use nav::*;
use search::*;
pub(crate) use shortcut::shortcut_keys;
pub use shortcut::{format_shortcut, shortcut_matches};
use submenu::*;
pub use text::*;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use freya_components::svg_viewer::SvgViewer;
use freya_core::elements::image::ImageHandle;
use freya_core::integration::{AppComponent, default_fonts};
use freya_core::prelude::*;
use freya_engine::prelude::{
    AlphaType, ColorType, FontStyle, ImageInfo, ParagraphBuilder, ParagraphStyle, Pixmap, SkData,
    SkImage, Slant, TextStyle, Width,
};
use keyboard_types::{Key, NamedKey};
use ragnarok::CursorPoint;
use torin::prelude::{Alignment, Direction, Gaps, Size};

type UseState<T> = State<T>;

const SUBMENU_INTENT_DELAY: Duration = Duration::from_millis(vow_design::MENU_SUBMENU_INTENT_MS);
const SCROLL_TICK: Duration = Duration::from_millis(vow_design::MENU_SCROLL_TICK_MS);

use crate::ProbeExt;
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
    Timers, inserts_text,
};
use crate::symbol::{Symbol, symbol};
use crate::theme;
use crate::theme::{SurfaceKind, menu_palette};
use vow_design::{MenuPalette, Rgba, SANS_FAMILIES};

#[derive(Clone, Copy)]
struct PaletteSource {
    surface: SurfaceKind,
    fixed: Option<MenuPalette>,
    popover: bool,
}

impl PaletteSource {
    fn live(surface: SurfaceKind) -> Self {
        Self {
            surface,
            fixed: None,
            popover: false,
        }
    }

    fn fixed(palette: MenuPalette) -> Self {
        Self {
            surface: SurfaceKind::LayerShell,
            fixed: Some(palette),
            popover: false,
        }
    }

    fn current(self) -> MenuPalette {
        self.with_resolved(&crate::theme::resolved())
    }

    fn with_resolved(self, resolved: &vow_appearance::Resolved) -> MenuPalette {
        let palette = self
            .fixed
            .unwrap_or_else(|| menu_palette(resolved, self.surface));
        if self.popover {
            crate::theme::apply_popover_metrics(palette)
        } else {
            palette
        }
    }

    fn inherit(self, mut view: MenuView) -> MenuView {
        view.palette_source = self;
        view
    }
}

pub struct MenuView {
    items: Rc<Vec<MenuItem>>,
    items_source: Option<ItemsSource>,
    on_activate: Rc<dyn Fn(String)>,
    on_dismiss_all: Rc<dyn Fn()>,
    on_open_submenu: Option<Rc<dyn Fn(String)>>,
    submenu_source: Option<SubmenuSource>,
    on_navigation: Option<Rc<dyn Fn(MenuNavigation)>>,
    palette_source: PaletteSource,
    searchable: Option<SearchableSource>,
    help_search: Option<SearchableSource>,
    query_state: Option<Rc<RefCell<SearchQuery>>>,
    menu_search_icon: Option<&'static str>,
    help_search_icon: Option<&'static str>,
    popup_spec: Option<PopupSpec>,
    reveal_roots: Option<RevealRoots>,
    focused_index: Option<usize>,
    probe: Option<&'static str>,
}

impl MenuView {
    pub fn new(
        items: Vec<MenuItem>,
        on_activate: impl Fn(String) + 'static,
        on_dismiss_all: impl Fn() + 'static,
    ) -> Self {
        Self {
            items: Rc::new(items),
            items_source: None,
            on_activate: Rc::new(on_activate),
            on_dismiss_all: Rc::new(on_dismiss_all),
            on_open_submenu: None,
            submenu_source: None,
            on_navigation: None,
            palette_source: PaletteSource::live(SurfaceKind::LayerShell),
            searchable: None,
            help_search: None,
            query_state: None,
            menu_search_icon: None,
            help_search_icon: None,
            popup_spec: None,
            reveal_roots: None,
            focused_index: None,
            probe: None,
        }
    }

    pub fn with_surface(mut self, kind: SurfaceKind) -> Self {
        self.palette_source = PaletteSource::live(kind);
        self
    }

    pub fn with_fixed_palette(mut self, palette: MenuPalette) -> Self {
        self.palette_source = PaletteSource::fixed(palette);
        self
    }

    pub fn with_popover_metrics(mut self) -> Self {
        self.palette_source.popover = true;
        self
    }

    pub fn with_popup_spec(mut self, spec: PopupSpec) -> Self {
        self.popup_spec = Some(spec);
        self
    }

    pub fn with_searchable(mut self, source: impl Fn() -> Vec<SearchableEntry> + 'static) -> Self {
        self.searchable = Some(Rc::new(source));
        self
    }

    pub fn with_help_topics(mut self, source: impl Fn() -> Vec<SearchableEntry> + 'static) -> Self {
        self.help_search = Some(Rc::new(source));
        self
    }

    pub fn with_search_icons(mut self, menu: &'static str, help: &'static str) -> Self {
        self.menu_search_icon = Some(menu);
        self.help_search_icon = Some(help);
        self
    }

    pub fn with_reveal_roots(mut self, source: impl Fn() -> Vec<Vec<MenuItem>> + 'static) -> Self {
        self.reveal_roots = Some(Rc::new(source));
        self
    }

    pub fn with_focused_index(mut self, index: Option<usize>) -> Self {
        self.focused_index = index;
        self
    }

    pub fn with_items_source(mut self, source: impl Fn() -> Vec<MenuItem> + 'static) -> Self {
        self.items_source = Some(Rc::new(source));
        self
    }

    pub fn with_open_submenu(mut self, callback: impl Fn(String) + 'static) -> Self {
        self.on_open_submenu = Some(Rc::new(callback));
        self
    }

    pub fn with_submenu_source(
        mut self,
        source: impl Fn(&str) -> Option<Vec<MenuItem>> + 'static,
    ) -> Self {
        self.submenu_source = Some(Rc::new(source));
        self
    }

    pub fn with_query_state(mut self, state: Rc<RefCell<SearchQuery>>) -> Self {
        self.query_state = Some(state);
        self
    }

    pub fn with_navigation(mut self, on_navigation: impl Fn(MenuNavigation) + 'static) -> Self {
        self.on_navigation = Some(Rc::new(on_navigation));
        self
    }

    pub const MAX_MENU_ROWS: usize = 9;
    pub const MAX_HELP_ROWS: usize = 5;

    pub fn preferred_size(items: &[MenuItem], m: MenuPalette) -> (f32, f32) {
        let hint;
        let items = if items.is_empty() {
            hint = [empty_hint()];
            hint.as_slice()
        } else {
            items
        };
        let inset = m.padding;
        let content_height: f32 = items.iter().map(|item| item.height(m)).sum();
        let min_width = m.min_width.max(1.0);
        let max_width = m.max_width.max(min_width);
        let max_content_width = (max_width - inset * 2.0).max(0.0);
        let min_content_width = (min_width - inset * 2.0).clamp(0.0, max_content_width);
        let mut content_width = min_content_width;

        for item in items {
            let Some(label) = item.label() else {
                continue;
            };
            let shortcut_w = item
                .shortcut()
                .map(|s| measure_text(&s, m.font_size))
                .unwrap_or(0.0);
            let trailing = if matches!(item, MenuItem::Submenu { .. }) {
                m.chevron_size
            } else if shortcut_w > 0.0 {
                m.shortcut_gap + shortcut_w
            } else {
                0.0
            };
            let text_inset = row_text_inset(items, m);
            let fixed = text_inset + m.item_padding_x + trailing;
            let max_label_w = (max_content_width - fixed).max(0.0);
            let label = truncate_middle(label, max_label_w, m.font_size);
            let label_w = measure_text(&label, m.font_size);
            let content_w = fixed + label_w;
            content_width = content_width.max(content_w);
        }

        let width =
            (content_width.min(max_content_width) + inset * 2.0).clamp(min_width, max_width);
        let height = content_height + inset * 2.0;
        (width.ceil(), height.ceil())
    }
}

impl Clone for MenuView {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            items_source: self.items_source.clone(),
            on_activate: self.on_activate.clone(),
            on_dismiss_all: self.on_dismiss_all.clone(),
            on_open_submenu: self.on_open_submenu.clone(),
            submenu_source: self.submenu_source.clone(),
            on_navigation: self.on_navigation.clone(),
            palette_source: self.palette_source,
            searchable: self.searchable.clone(),
            help_search: self.help_search.clone(),
            query_state: self.query_state.clone(),
            menu_search_icon: self.menu_search_icon,
            help_search_icon: self.help_search_icon,
            popup_spec: self.popup_spec.clone(),
            reveal_roots: self.reveal_roots.clone(),
            focused_index: self.focused_index,
            probe: self.probe,
        }
    }
}

impl ProbeExt for MenuView {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl PartialEq for MenuView {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl App for MenuView {
    fn render(&self) -> impl IntoElement {
        let items = match &self.items_source {
            Some(source) => Rc::new(source()),
            None => self.items.clone(),
        };
        let on_activate = self.on_activate.clone();
        let on_dismiss_all = self.on_dismiss_all.clone();
        let on_open_submenu = self.on_open_submenu.clone();
        let submenu_source = self.submenu_source.clone();
        let on_navigation = self.on_navigation.clone();
        let palette_source = self.palette_source;
        let m = palette_source.current();
        let platform = Platform::get();
        let scale = (*platform.scale_factor.peek()).max(f64::EPSILON);
        let root_size = *platform.root_size.peek();
        let size = (
            (root_size.width / scale as f32).max(1.0),
            (root_size.height / scale as f32).max(1.0),
        );
        let popup: Option<PopupHandle> = try_consume_root_context();
        let opens_left = popup.as_ref().is_some_and(PopupHandle::opens_left);
        let menu_search_icon = self.menu_search_icon;
        let help_search_icon = self.help_search_icon;
        let popup_spec = self.popup_spec.clone();
        let initial_focused = self.focused_index;
        let items_for_focus = items.clone();
        let mut focused =
            use_state(move || initial_focused.or_else(|| first_selectable(&items_for_focus)));
        let open_child_id = use_hook(|| Rc::new(Cell::new(None)));
        let open_child_index = use_hook(|| Rc::new(Cell::new(None::<usize>)));
        let hover_state = use_hook(|| Rc::new(RefCell::new(HoverState::default())));
        let timers: Option<Timers> = try_consume_root_context();
        let query_state = self.query_state.clone();
        let initial = query_state
            .as_ref()
            .map(|q| q.borrow().clone())
            .unwrap_or_default();
        let query = use_state(|| initial.text);
        let selected = use_state(|| initial.selected);
        let result_focused = use_state(|| None::<usize>);
        let prev_query = use_hook(|| Rc::new(RefCell::new(String::new())));
        let searchable = self.searchable.clone();
        let help_search = self.help_search.clone();
        let searchable_entries = searchable.as_ref().map(|source| source());
        let help_entries = help_search.as_ref().map(|source| source());

        let has_search = searchable_entries.is_some() || help_entries.is_some();
        let query_text = (*query.read()).clone();
        let is_selected = *selected.read();

        if let Some(state) = &query_state {
            let current = SearchQuery {
                text: query_text.clone(),
                selected: is_selected,
            };
            if state.borrow().text != current.text || state.borrow().selected != current.selected {
                *state.borrow_mut() = current;
            }
        }

        let (menu_results, help_results) = search_result_lists(
            searchable_entries.as_deref(),
            help_entries.as_deref(),
            &query_text,
        )
        .unwrap_or_default();
        let mut menu_scroll = use_state(|| 0usize);
        let mut help_scroll = use_state(|| 0usize);

        if prev_query.borrow().as_str() != query_text {
            *prev_query.borrow_mut() = query_text.clone();
            menu_scroll.set(0);
            help_scroll.set(0);
        }

        let visible = if query_text.is_empty() {
            with_empty_hint((*items).clone())
        } else {
            visible_items(
                &items,
                &menu_results,
                &help_results,
                *menu_scroll.read(),
                *help_scroll.read(),
            )
        };

        let last_size = use_hook(|| {
            Rc::new(Cell::new(
                popup_spec.as_ref().map(|spec| spec.size).unwrap_or((0, 0)),
            ))
        });
        let wanted = MenuView::preferred_size(&visible, m);
        let wanted = (wanted.0 as u32, wanted.1 as u32);
        if wanted != last_size.get() {
            last_size.set(wanted);
            if let (Some(popup), Some(spec)) = (&popup, &popup_spec) {
                popup.reposition(PopupSpec {
                    size: wanted,
                    ..spec.clone()
                });
            }
        }

        let results_start = visible
            .iter()
            .position(|item| matches!(item, MenuItem::SearchResult { .. }));
        let results_count = if let Some(start) = results_start {
            visible
                .iter()
                .skip(start)
                .take_while(|item| matches!(item, MenuItem::SearchResult { .. }))
                .count()
        } else {
            0
        };
        let menu_section = find_result_section(&visible, SearchSource::Menu);
        let help_section = find_result_section(&visible, SearchSource::Help);

        let focused_index = *focused.read();
        if let Some(index) = focused_index
            && !items.get(index).is_some_and(MenuItem::selectable)
        {
            let next = first_selectable(&items);
            if next != focused_index {
                focused.set(next);
            }
        }
        let result_focused_index = *result_focused.read();
        let reveal_child_id = use_hook(|| Rc::new(Cell::new(None::<PopupId>)));
        let prev_revealed = use_hook(|| Rc::new(RefCell::new(None::<SearchLocation>)));
        let reveal_roots = self.reveal_roots.clone();

        let revealed_location = result_focused_index.and_then(|i| {
            visible.get(i).and_then(|item| match item {
                MenuItem::SearchResult {
                    source: SearchSource::Menu,
                    location: Some(loc),
                    ..
                } => Some(loc.clone()),
                _ => None,
            })
        });
        if revealed_location != *prev_revealed.borrow() {
            *prev_revealed.borrow_mut() = revealed_location.clone();
            close_child(reveal_child_id.clone(), Rc::new(Cell::new(None)));
            if let (Some(loc), Some(roots), Some(popup)) =
                (revealed_location, reveal_roots.as_ref(), popup.clone())
            {
                let roots = roots();
                if let Some(items) = roots.get(loc.title_index) {
                    let (anchor_x, anchor_width) = popup_spec
                        .as_ref()
                        .map(|spec| (spec.anchor_rect.0 as f32, spec.anchor_rect.2 as f32))
                        .unwrap_or((0.0, 1.0));
                    open_reveal_popup(
                        items,
                        &loc,
                        popup,
                        size,
                        palette_source,
                        on_activate.clone(),
                        on_dismiss_all.clone(),
                        reveal_child_id.clone(),
                        opens_left,
                        anchor_x,
                        anchor_width,
                    );
                }
            }
        }

        let visible_rc = Rc::new(visible.clone());

        let alt_held = use_state(|| false);
        let show_mnemonic = *alt_held.read();

        let inset = m.padding;
        let content_height: f32 = visible.iter().map(|item| item.height(m)).sum();
        let available = (size.1 - inset * 2.0).max(0.0);
        let overflow = query_text.is_empty() && content_height > available + m.item_height / 2.0;
        let arrow_height = m.header_height;
        let viewport = if overflow {
            (available - 2.0 * arrow_height).max(m.item_height)
        } else {
            available
        };
        let max_scroll = (content_height - viewport).max(0.0);
        let mut scroll = use_state(|| 0.0f32);
        let mut scroll_now = if overflow {
            scroll.read().clamp(0.0, max_scroll)
        } else {
            0.0
        };
        let last_focus = use_hook(|| Rc::new(Cell::new(None::<usize>)));
        if overflow && last_focus.get() != focused_index {
            last_focus.set(focused_index);
            if let Some(index) = focused_index
                && let Some(item) = visible.get(index)
            {
                let top = item_top(index, &visible, m);
                let bottom = top + item.height(m);
                let wanted = if top < scroll_now {
                    top
                } else if bottom > scroll_now + viewport {
                    bottom - viewport
                } else {
                    scroll_now
                };
                if wanted != scroll_now {
                    scroll_now = wanted;
                    scroll.set(wanted);
                }
            }
        }
        let shift = if overflow {
            scroll_now - arrow_height
        } else {
            0.0
        };
        if let Some(popup) = &popup {
            popup.set_scroll_shift(shift);
        }
        let auto_scroll = use_hook(|| Rc::new(Cell::new(0i8)));

        let execute_target: Rc<dyn Fn(usize)> = {
            let focused = Cell::new(focused);
            let open_child_id = open_child_id.clone();
            let open_child_index = open_child_index.clone();
            let hover_state = hover_state.clone();
            let popup = popup.clone();
            let items = visible_rc.clone();
            let on_activate = on_activate.clone();
            let on_dismiss_all = on_dismiss_all.clone();
            let on_open_submenu = on_open_submenu.clone();
            let submenu_source = submenu_source.clone();
            Rc::new(move |index: usize| {
                hover_state.borrow_mut().pending = None;
                let mut focused = focused.get();
                focused.set(Some(index));
                if open_child_index.get() == Some(index) {
                    return;
                }
                close_child(open_child_id.clone(), open_child_index.clone());
                if let Some(MenuItem::Submenu { id, items: sub, .. }) = items.get(index)
                    && let Some(popup) = popup.clone()
                {
                    open_child_popup(
                        index,
                        id.as_deref(),
                        sub,
                        popup,
                        size,
                        palette_source,
                        on_activate.clone(),
                        on_dismiss_all.clone(),
                        on_open_submenu.clone(),
                        submenu_source.clone(),
                        open_child_id.clone(),
                        open_child_index.clone(),
                        items.clone(),
                        opens_left,
                    );
                }
            })
        };

        let start_delay = {
            let hover_state = hover_state.clone();
            let timers = timers.clone();
            let execute_target = execute_target.clone();
            move |target_index: usize,
                  from: CursorPoint,
                  to_top: CursorPoint,
                  to_bottom: CursorPoint| {
                let mut hs = hover_state.borrow_mut();
                hs.generation += 1;
                let generation = hs.generation;
                hs.pending = Some(HoverPending {
                    generation,
                    target_index,
                    from,
                    to_top,
                    to_bottom,
                });
                drop(hs);
                if let Some(timers) = timers.clone() {
                    let hover_state = hover_state.clone();
                    let execute_target = execute_target.clone();
                    timers.after(SUBMENU_INTENT_DELAY, move || {
                        if hover_state
                            .borrow()
                            .pending
                            .as_ref()
                            .is_some_and(|p| p.generation == generation)
                        {
                            execute_target(target_index);
                        }
                    });
                } else {
                    execute_target(target_index);
                }
            }
        };

        let enter_row: Rc<dyn Fn(usize, CursorPoint)> = {
            let hover_state = hover_state.clone();
            let open_child_index = open_child_index.clone();
            let items = visible_rc.clone();
            let execute_target = execute_target.clone();
            let start_delay = Rc::new(start_delay);
            Rc::new(move |index: usize, pos: CursorPoint| {
                let from = hover_state.borrow().last.map(|(_, p)| p).unwrap_or(pos);
                hover_state.borrow_mut().last = Some((Instant::now(), pos));
                hover_state.borrow_mut().pending = None;
                if let Some(child_index) = open_child_index.get()
                    && child_index != index
                    && let Some(MenuItem::Submenu { .. }) = items.get(child_index)
                {
                    let row_top = m.padding + item_top(child_index, &items, m) - shift;
                    let near_x = if opens_left { 0.0 } else { size.0 };
                    let to_top = CursorPoint::new(near_x as f64, row_top as f64);
                    let to_bottom =
                        CursorPoint::new(near_x as f64, (row_top + m.item_height) as f64);
                    if point_in_triangle(pos, from, to_top, to_bottom) {
                        start_delay(index, from, to_top, to_bottom);
                        return;
                    }
                }
                execute_target(index);
            })
        };

        let move_row: Rc<dyn Fn(usize, CursorPoint)> = {
            let hover_state = hover_state.clone();
            let execute_target = execute_target.clone();
            Rc::new(move |index: usize, pos: CursorPoint| {
                hover_state.borrow_mut().last = Some((Instant::now(), pos));
                let pending = hover_state.borrow().pending;
                if let Some(pending) = pending
                    && pending.target_index == index
                    && !point_in_triangle(pos, pending.from, pending.to_top, pending.to_bottom)
                {
                    execute_target(index);
                }
            })
        };

        let rows: Vec<Element> = visible
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_focused = if !query_text.is_empty() && results_start.is_some() {
                    result_focused_index == Some(index)
                } else {
                    focused_index == Some(index)
                };
                render_item(
                    item,
                    index,
                    is_focused,
                    m,
                    palette_source,
                    popup.clone(),
                    opens_left,
                    on_activate.clone(),
                    on_dismiss_all.clone(),
                    visible_rc.clone(),
                    focused,
                    open_child_id.clone(),
                    open_child_index.clone(),
                    size,
                    query,
                    selected,
                    menu_search_icon,
                    help_search_icon,
                    result_focused,
                    on_open_submenu.clone(),
                    submenu_source.clone(),
                    enter_row.clone(),
                    move_row.clone(),
                    execute_target.clone(),
                    show_mnemonic,
                )
            })
            .collect();
        let rows: Vec<Element> = if overflow {
            let step = m.item_height;
            let arrow = |direction: i8, active: bool| {
                scroll_arrow(
                    direction,
                    active,
                    arrow_height,
                    m,
                    timers.clone(),
                    auto_scroll.clone(),
                    scroll,
                    step / 3.0,
                    max_scroll,
                )
            };
            vec![
                arrow(-1, scroll_now > 0.0),
                rect()
                    .width(Size::fill())
                    .height(Size::px(viewport))
                    .overflow(Overflow::Clip)
                    .offset_y(-scroll_now)
                    .direction(Direction::Vertical)
                    .children(rows)
                    .into_element(),
                arrow(1, scroll_now < max_scroll),
            ]
        } else {
            rows
        };

        let content = rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new(m.padding, m.padding, m.padding, m.padding))
            .background(theme::color(m.bg))
            .direction(Direction::Vertical)
            .on_wheel(move |event: Event<WheelEventData>| {
                if overflow {
                    let next = (*scroll.peek() - event.delta_y as f32).clamp(0.0, max_scroll);
                    let mut scroll = scroll;
                    scroll.set(next);
                }
            })
            .children(rows);

        let mut root = rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(theme::color(m.bg))
            .overflow(Overflow::Clip)
            .child(content);
        for family in SANS_FAMILIES {
            root = root.font_family(family);
        }
        let hover_state_leave = hover_state.clone();
        root = root.on_pointer_leave(move |event: Event<PointerEventData>| {
            let pos = event.global_location();
            let mut hs = hover_state_leave.borrow_mut();
            hs.last = Some((Instant::now(), pos));
            hs.pending = None;
        });
        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }
        let visible_for_keys = Rc::new(visible.clone());
        let mut alt_up = alt_held;
        root = root.on_global_key_up(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Alt) {
                alt_up.set(false);
            }
        });
        let mut alt_down = alt_held;
        root.on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Alt) {
                alt_down.set(true);
                return;
            }
            hover_state.borrow_mut().pending = None;
            let mut focused = focused;
            let mut query = query;
            let mut result_focused = result_focused;
            let mut selected = selected;
            let mut menu_scroll = menu_scroll;
            let mut help_scroll = help_scroll;
            let open_child_id = open_child_id.clone();
            let current = *focused.read();
            let query_text = (*query.read()).clone();
            let result_current = *result_focused.read();
            let searching = has_search && !query_text.is_empty();
            let is_selected = *selected.read();
            let menu_results_len = menu_results.len();
            let help_results_len = help_results.len();
            match event.key {
                Key::Named(NamedKey::ArrowDown) => {
                    selected.set(false);
                    if searching {
                        if results_count > 0 {
                            let start = results_start.unwrap();
                            let end = start + results_count - 1;
                            if let Some(i) = result_current {
                                if i < end {
                                    result_focused.set(Some(i + 1));
                                } else {
                                    if let Some((m_start, m_count)) = menu_section {
                                        let m_end = m_start + m_count - 1;
                                        if i == m_end
                                            && *menu_scroll.read() + m_count < menu_results_len
                                        {
                                            menu_scroll.set(*menu_scroll.read() + 1);
                                        }
                                    }
                                    if let Some((h_start, h_count)) = help_section {
                                        let h_end = h_start + h_count - 1;
                                        if i == h_end
                                            && *help_scroll.read() + h_count < help_results_len
                                        {
                                            help_scroll.set(*help_scroll.read() + 1);
                                        }
                                    }
                                }
                            } else {
                                result_focused.set(Some(start));
                            }
                        }
                    } else {
                        focused.set(next_selectable(&items, current, 1));
                    }
                }
                Key::Named(NamedKey::ArrowUp) => {
                    selected.set(false);
                    if searching {
                        if results_count > 0 {
                            let start = results_start.unwrap();
                            if let Some(i) = result_current {
                                if i > start {
                                    result_focused.set(Some(i - 1));
                                } else {
                                    if let Some((m_start, _m_count)) = menu_section
                                        && i == m_start
                                        && *menu_scroll.read() > 0
                                    {
                                        menu_scroll.set(*menu_scroll.read() - 1);
                                    }
                                    if let Some((h_start, _h_count)) = help_section
                                        && i == h_start
                                        && *help_scroll.read() > 0
                                    {
                                        help_scroll.set(*help_scroll.read() - 1);
                                    }
                                }
                            } else {
                                result_focused.set(Some(start + results_count - 1));
                            }
                        }
                    } else {
                        focused.set(next_selectable(&items, current, -1));
                    }
                }
                Key::Named(NamedKey::Home) => {
                    if !searching {
                        focused.set(first_selectable(&items));
                    }
                }
                Key::Named(NamedKey::End) => {
                    if !searching {
                        focused.set(last_selectable(&items));
                    }
                }
                Key::Named(NamedKey::Enter) => {
                    selected.set(false);
                    let (index, source): (Option<usize>, &[MenuItem]) = if searching {
                        (result_current, visible_for_keys.as_slice())
                    } else {
                        (current, items.as_slice())
                    };
                    if let Some(index) = index {
                        activate_or_open(
                            source,
                            index,
                            popup.clone(),
                            size,
                            palette_source,
                            on_activate.clone(),
                            on_dismiss_all.clone(),
                            on_open_submenu.clone(),
                            submenu_source.clone(),
                            open_child_id.clone(),
                            open_child_index.clone(),
                            opens_left,
                        );
                    }
                }
                Key::Character(ref ch)
                    if ch == " "
                        && inserts_text(event.modifiers)
                        && !(has_search && !query_text.is_empty()) =>
                {
                    selected.set(false);
                    let (index, source): (Option<usize>, &[MenuItem]) = if searching {
                        (result_current, visible_for_keys.as_slice())
                    } else {
                        (current, items.as_slice())
                    };
                    if let Some(index) = index {
                        activate_or_open(
                            source,
                            index,
                            popup.clone(),
                            size,
                            palette_source,
                            on_activate.clone(),
                            on_dismiss_all.clone(),
                            on_open_submenu.clone(),
                            submenu_source.clone(),
                            open_child_id.clone(),
                            open_child_index.clone(),
                            opens_left,
                        );
                    }
                }
                Key::Character(ref ch) if has_search && inserts_text(event.modifiers) => {
                    let mut text = query_text;
                    if is_selected {
                        text.clear();
                        selected.set(false);
                    }
                    text.push_str(ch);
                    query.set(text);
                }
                Key::Named(NamedKey::Backspace) if has_search => {
                    if is_selected {
                        query.set(String::new());
                        selected.set(false);
                    } else {
                        let mut text = query_text;
                        text.pop();
                        query.set(text);
                    }
                }
                Key::Named(NamedKey::Escape) if has_search => {
                    if query_text.is_empty() {
                        if let Some(popup) = &popup {
                            popup.close();
                        }
                    } else {
                        query.set(String::new());
                        selected.set(false);
                        result_focused.set(None);
                        focused.set(first_selectable(&items));
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    if searching {
                        return;
                    }
                    if let Some(index) = current {
                        if let Some(MenuItem::Submenu { id, items: sub, .. }) = items.get(index)
                            && let Some(popup) = popup.clone()
                        {
                            open_child_popup(
                                index,
                                id.as_deref(),
                                sub,
                                popup,
                                size,
                                palette_source,
                                on_activate.clone(),
                                on_dismiss_all.clone(),
                                on_open_submenu.clone(),
                                submenu_source.clone(),
                                open_child_id.clone(),
                                open_child_index.clone(),
                                Rc::new(items.to_vec()),
                                opens_left,
                            );
                        } else if popup.is_some()
                            && let Some(on_navigation) = &on_navigation
                        {
                            on_navigation(MenuNavigation::Next);
                        }
                    }
                }
                Key::Named(NamedKey::ArrowLeft) if popup.is_some() && on_navigation.is_some() => {
                    on_navigation.as_ref().unwrap()(MenuNavigation::Prev);
                }
                Key::Named(NamedKey::ArrowLeft) if !searching => {
                    if let Some(popup) = &popup {
                        popup.close();
                    }
                }
                _ => {}
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn scroll_arrow(
    direction: i8,
    active: bool,
    height: f32,
    m: MenuPalette,
    timers: Option<Timers>,
    auto: Rc<Cell<i8>>,
    scroll: UseState<f32>,
    step: f32,
    max_scroll: f32,
) -> Element {
    let strip = rect().width(Size::fill()).height(Size::px(height)).center();
    if !active {
        return strip.into_element();
    }
    let leave = auto.clone();
    strip
        .on_pointer_enter(move |_| {
            auto.set(direction);
            if let Some(timers) = timers.clone() {
                scroll_tick(timers, auto.clone(), direction, scroll, step, max_scroll);
            }
        })
        .on_pointer_leave(move |_| leave.set(0))
        .child(
            label()
                .text(if direction < 0 { "▲" } else { "▼" })
                .font_size(m.chevron_size)
                .color(theme::color(m.chevron_color)),
        )
        .into_element()
}

fn scroll_tick(
    timers: Timers,
    auto: Rc<Cell<i8>>,
    direction: i8,
    mut scroll: UseState<f32>,
    step: f32,
    max_scroll: f32,
) {
    if auto.get() != direction {
        return;
    }
    let next = (*scroll.peek() + f32::from(direction) * step).clamp(0.0, max_scroll);
    if next == *scroll.peek() {
        auto.set(0);
        return;
    }
    scroll.set(next);
    let again = timers.clone();
    timers.after(SCROLL_TICK, move || {
        scroll_tick(
            again.clone(),
            auto.clone(),
            direction,
            scroll,
            step,
            max_scroll,
        );
    });
}
