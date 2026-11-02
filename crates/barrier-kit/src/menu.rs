//! Menu control for popups, with submenus and keyboard navigation.

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
    AlphaType, ColorType, ImageInfo, ParagraphBuilder, ParagraphStyle, Pixmap, SkData, SkImage,
    TextStyle,
};
use keyboard_types::{Key, NamedKey};
use ragnarok::CursorPoint;
use torin::prelude::{Alignment, Direction, Gaps, Size};

type UseState<T> = State<T>;

const SUBMENU_INTENT_DELAY: Duration = Duration::from_millis(300);
const SCROLL_TICK: Duration = Duration::from_millis(30);

pub const MNEMONIC_MARK: char = '\u{2063}';

pub fn strip_mnemonic(label: &str) -> String {
    label.replace(MNEMONIC_MARK, "")
}

fn item_label(text: &str, show_mnemonic: bool, size: f32, color: Rgba) -> Element {
    let split = text.split_once(MNEMONIC_MARK).and_then(|(before, rest)| {
        let letter = rest.chars().next()?;
        Some((before, letter, &rest[letter.len_utf8()..]))
    });
    match split {
        Some((before, letter, after)) if show_mnemonic => paragraph()
            .font_size(size)
            .color(rgba(color))
            .max_lines(1)
            .span(Span::new(before.to_string()))
            .span(Span::new(letter.to_string()).text_decoration(TextDecoration::Underline))
            .span(Span::new(strip_mnemonic(after)))
            .into_element(),
        _ => label()
            .text(strip_mnemonic(text))
            .font_size(size)
            .color(rgba(color))
            .max_lines(1)
            .into_element(),
    }
}

#[derive(Default)]
struct HoverState {
    last: Option<(Instant, CursorPoint)>,
    pending: Option<HoverPending>,
    generation: u64,
}

#[derive(Clone, Copy)]
struct HoverPending {
    generation: u64,
    target_index: usize,
    from: CursorPoint,
    to_top: CursorPoint,
    to_bottom: CursorPoint,
}

fn point_in_triangle(p: CursorPoint, a: CursorPoint, b: CursorPoint, c: CursorPoint) -> bool {
    let sign = |p1: CursorPoint, p2: CursorPoint, p3: CursorPoint| {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    };
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

use crate::ProbeExt;
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupId, PopupSpec, Popups,
    Timers, inserts_text,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::{SurfaceKind, menu_palette};
use vow_design::{MenuPalette, Rgba, SANS_FAMILIES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchSource {
    Menu,
    Help,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchLocation {
    pub title_index: usize,
    pub indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchableEntry {
    pub path: String,
    pub item: MenuItem,
    pub location: Option<SearchLocation>,
    pub source: SearchSource,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchQuery {
    pub text: String,
    pub selected: bool,
}

fn font_families() -> Vec<Cow<'static, str>> {
    let mut families: Vec<Cow<'static, str>> = SANS_FAMILIES
        .iter()
        .map(|name| Cow::Borrowed(*name))
        .collect();
    families.extend(default_fonts());
    families
}

pub fn format_shortcut(text: &str) -> String {
    let text = text.trim();
    let (mut ctrl, mut alt, mut shift, mut sup) = (false, false, false, false);
    let mut key = String::new();
    for token in text.split('+').map(str::trim).filter(|t| !t.is_empty()) {
        match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" | "option" | "opt" => alt = true,
            "shift" => shift = true,
            "super" | "meta" | "cmd" | "command" => sup = true,
            "escape" | "esc" => key.push('⎋'),
            "left" => key.push('←'),
            "right" => key.push('→'),
            "up" => key.push('↑'),
            "down" => key.push('↓'),
            "return" | "enter" => key.push('↩'),
            "tab" => key.push('⇥'),
            _ => key.push_str(token),
        }
    }
    let mut out = String::new();
    if ctrl {
        out.push('⌃');
    }
    if alt {
        out.push('⌥');
    }
    if shift {
        out.push('⇧');
    }
    if sup {
        out.push('❖');
    }
    out.push_str(&key);
    out
}

pub fn measure_text(text: &str, size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let mut paragraph_style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(size);
    text_style.set_font_families(&font_families());
    paragraph_style.set_text_style(&text_style);

    let mut builder = ParagraphBuilder::new(&paragraph_style, crate::fonts::collection());
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(f32::MAX);
    paragraph.max_intrinsic_width()
}

fn truncate_middle(text: &str, max_width: f32, font_size: f32) -> String {
    if max_width <= 0.0 || text.is_empty() || measure_text(text, font_size) <= max_width {
        return text.to_string();
    }
    let ellipsis = "…";
    let ellipsis_w = measure_text(ellipsis, font_size);
    let max_body = (max_width - ellipsis_w).max(0.0);
    let chars: Vec<char> = text.chars().collect();
    let mut left = chars.len() / 2;
    let mut right = chars.len() - left;
    while left > 0 || right > 0 {
        let body: String = chars
            .iter()
            .take(left)
            .chain(chars.iter().skip(chars.len() - right))
            .collect();
        if measure_text(&body, font_size) <= max_body {
            return format!("{body}{ellipsis}");
        }
        if left >= right && left > 0 {
            left -= 1;
        } else {
            right = right.saturating_sub(1);
        }
    }
    ellipsis.to_string()
}

const MARK_TEXT_GAP: f32 = 6.0;

fn has_leading_column(items: &[MenuItem]) -> bool {
    let mut has = false;
    for item in items {
        match item {
            MenuItem::Action { check, icon, .. } => {
                if !matches!(check, CheckState::None) || icon.is_some() {
                    has = true;
                }
            }
            MenuItem::Submenu { icon, .. } => {
                if icon.is_some() {
                    has = true;
                }
            }
            MenuItem::SearchResult { item, .. } => {
                has |= has_leading_column(std::slice::from_ref(item));
            }
            _ => {}
        }
        if has {
            break;
        }
    }
    has
}

fn row_text_inset(items: &[MenuItem], m: MenuPalette) -> f32 {
    if has_leading_column(items) {
        m.item_padding_x + m.leading_width + MARK_TEXT_GAP
    } else {
        m.item_padding_x
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    None,
    Checkbox(bool),
    Radio(bool),
}

pub type SubmenuSource = Rc<dyn Fn(&str) -> Option<Vec<MenuItem>>>;

pub type ItemsSource = Rc<dyn Fn() -> Vec<MenuItem>>;

pub type SearchableSource = Rc<dyn Fn() -> Vec<SearchableEntry>>;

pub type RevealRoots = Rc<dyn Fn() -> Vec<Vec<MenuItem>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuNavigation {
    Prev,
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuIcon {
    Svg(Arc<str>),
    Image(Arc<[u8]>),
}

impl MenuIcon {
    pub const MAX_IMAGE_BYTES: usize = 64 * 1024;

    pub fn svg(s: impl Into<Arc<str>>) -> Self {
        Self::Svg(s.into())
    }

    pub fn image(bytes: impl Into<Arc<[u8]>>) -> Option<Self> {
        let bytes = bytes.into();
        if bytes.len() > Self::MAX_IMAGE_BYTES {
            tracing::debug!(
                bytes = bytes.len(),
                "menu: icon-data is too large; dropping the icon"
            );
            return None;
        }
        Some(Self::Image(bytes))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Action {
        id: String,
        label: String,
        shortcut: Option<String>,
        enabled: bool,
        check: CheckState,
        icon: Option<MenuIcon>,
    },
    Submenu {
        id: Option<String>,
        label: String,
        enabled: bool,
        items: Vec<MenuItem>,
        icon: Option<MenuIcon>,
    },
    Header {
        label: String,
        count: Option<usize>,
    },
    Separator,
    SearchField {
        placeholder: String,
    },
    SearchResult {
        path: String,
        item: Box<MenuItem>,
        source: SearchSource,
        location: Option<SearchLocation>,
    },
}

impl MenuItem {
    fn enabled(&self) -> bool {
        match self {
            MenuItem::Action { enabled, .. } => *enabled,
            MenuItem::Submenu { enabled, .. } => *enabled,
            MenuItem::SearchResult { item, .. } => item.enabled(),
            MenuItem::Header { .. } | MenuItem::Separator | MenuItem::SearchField { .. } => false,
        }
    }

    fn selectable(&self) -> bool {
        matches!(
            self,
            MenuItem::Action { .. } | MenuItem::Submenu { .. } | MenuItem::SearchResult { .. }
        ) && self.enabled()
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            MenuItem::Action { label, .. }
            | MenuItem::Submenu { label, .. }
            | MenuItem::Header { label, .. } => Some(label),
            MenuItem::SearchResult { item, .. } => item.label(),
            MenuItem::Separator | MenuItem::SearchField { .. } => None,
        }
    }

    fn shortcut(&self) -> Option<String> {
        match self {
            MenuItem::Action { shortcut, .. } => shortcut.as_deref().map(format_shortcut),
            MenuItem::SearchResult { item, .. } => item.shortcut(),
            MenuItem::Submenu { .. }
            | MenuItem::Header { .. }
            | MenuItem::Separator
            | MenuItem::SearchField { .. } => None,
        }
    }

    pub(crate) fn height(&self, m: MenuPalette) -> f32 {
        match self {
            MenuItem::Separator => m.separator_height + m.separator_margin_y * 2.0,
            MenuItem::SearchField { .. } => m.search_height + m.search_margin_y * 2.0,
            MenuItem::Header { .. } => m.header_height,
            _ => m.item_height,
        }
    }
}

pub fn normalize_separators(items: &mut Vec<MenuItem>) {
    for item in items.iter_mut() {
        if let MenuItem::Submenu { items, .. } = item {
            normalize_separators(items);
        }
    }
    let mut after_separator = true;
    items.retain(|item| {
        if matches!(item, MenuItem::Separator) {
            let keep = !after_separator;
            after_separator = true;
            keep
        } else {
            after_separator = false;
            true
        }
    });
    while matches!(items.last(), Some(MenuItem::Separator)) {
        items.pop();
    }
}

fn empty_hint() -> MenuItem {
    MenuItem::Action {
        id: String::new(),
        label: "Sin elementos".to_string(),
        shortcut: None,
        enabled: false,
        check: CheckState::None,
        icon: None,
    }
}

fn with_empty_hint(items: Vec<MenuItem>) -> Vec<MenuItem> {
    if items.is_empty() {
        vec![empty_hint()]
    } else {
        items
    }
}

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
    open_path: Option<Rc<Vec<usize>>>,
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
            open_path: None,
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

    pub fn with_open_path(mut self, path: Vec<usize>) -> Self {
        self.open_path = Some(Rc::new(path));
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
            open_path: self.open_path.clone(),
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
            .background(rgba(m.bg))
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
            .background(rgba(m.bg))
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
                .color(rgba(m.chevron_color)),
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

fn rgba(color: Rgba) -> Color {
    let [r, g, b, a] = color.to_rgba8();
    Color::from_argb(a, r, g, b)
}

fn svg_rgb(color: Rgba) -> String {
    let [r, g, b, _] = color.to_rgba8();
    format!("rgb({r},{g},{b})")
}

pub fn search_icon_svg(template: &str, text: Rgba, accent: Rgba) -> String {
    template
        .replace("{{text}}", &svg_rgb(text))
        .replace("{{accent}}", &svg_rgb(accent))
}

fn shortcut_fg(enabled: bool, focused: bool, m: MenuPalette) -> Rgba {
    if !enabled {
        m.disabled_fg
    } else if focused {
        m.hover_fg
    } else {
        m.shortcut_color
    }
}

fn chevron_fg(enabled: bool, focused: bool, m: MenuPalette) -> Rgba {
    if !enabled {
        m.disabled_fg
    } else if focused {
        m.hover_fg
    } else {
        m.chevron_color
    }
}

fn first_selectable(items: &[MenuItem]) -> Option<usize> {
    items.iter().position(|item| item.selectable())
}

fn last_selectable(items: &[MenuItem]) -> Option<usize> {
    items.iter().rposition(|item| item.selectable())
}

fn next_selectable(items: &[MenuItem], current: Option<usize>, delta: isize) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }
    let start = current
        .map(|i| (i as isize + delta).rem_euclid(len as isize) as usize)
        .unwrap_or_else(|| if delta > 0 { 0 } else { len - 1 });
    let mut index = start;
    loop {
        if items[index].selectable() {
            return Some(index);
        }
        index = (index as isize + delta.signum()).rem_euclid(len as isize) as usize;
        if index == start {
            return None;
        }
    }
}

fn activate_at(items: &[MenuItem], index: usize) -> Option<String> {
    match items.get(index)? {
        MenuItem::Action { id, enabled, .. } if *enabled => Some(id.clone()),
        MenuItem::SearchResult { item, .. } => activate_at(std::slice::from_ref(item), 0),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn activate_or_open(
    items: &[MenuItem],
    index: usize,
    popup: Option<PopupHandle>,
    size: (f32, f32),
    palette_source: PaletteSource,
    on_activate: Rc<dyn Fn(String)>,
    on_dismiss_all: Rc<dyn Fn()>,
    on_open_submenu: Option<Rc<dyn Fn(String)>>,
    submenu_source: Option<SubmenuSource>,
    open_child_id: Rc<Cell<Option<PopupId>>>,
    open_child_index: Rc<Cell<Option<usize>>>,
    opens_left: bool,
) {
    if let Some(id) = activate_at(items, index) {
        on_activate(id);
        on_dismiss_all();
    } else if let Some(MenuItem::Submenu { id, items: sub, .. }) = items.get(index)
        && let Some(popup) = popup
    {
        open_child_popup(
            index,
            id.as_deref(),
            sub,
            popup,
            size,
            palette_source,
            on_activate,
            on_dismiss_all,
            on_open_submenu,
            submenu_source,
            open_child_id,
            open_child_index,
            Rc::new(items.to_vec()),
            opens_left,
        );
    }
}

fn normalize_query(text: &str) -> String {
    strip_mnemonic(text)
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace('ñ', "n")
        .replace('Á', "a")
        .replace('É', "e")
        .replace('Í', "i")
        .replace('Ó', "o")
        .replace(['Ú', 'Ü'], "u")
        .replace('Ñ', "n")
}

fn word_start(name: &str, query: &str) -> bool {
    name.char_indices().any(|(index, _)| {
        if index == 0 {
            return false;
        }
        let before = name[..index].chars().next_back();
        let boundary = before.is_none_or(|ch| !ch.is_alphanumeric());
        boundary && name[index..].starts_with(query)
    })
}

fn search_score(label: &str, query: &str, enabled: bool) -> u8 {
    let normalized = normalize_query(label);
    let base = if normalized.starts_with(query) {
        0
    } else if word_start(&normalized, query) {
        1
    } else {
        2
    };
    if enabled { base } else { base + 3 }
}

fn filter_search_results(query: &str, searchable: &[SearchableEntry]) -> Vec<SearchableEntry> {
    let normalized = normalize_query(query);
    let mut matches: Vec<(u8, &SearchableEntry)> = searchable
        .iter()
        .filter_map(|entry| {
            let label = entry.item.label()?;
            if !normalize_query(label).contains(&normalized) {
                return None;
            }
            let score = search_score(label, &normalized, entry.item.enabled());
            Some((score, entry))
        })
        .collect();
    matches.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.item.label().cmp(&b.1.item.label()))
    });
    matches
        .into_iter()
        .map(|(_, entry)| entry.clone())
        .collect()
}

fn search_result_lists(
    searchable: Option<&[SearchableEntry]>,
    help_search: Option<&[SearchableEntry]>,
    query: &str,
) -> Option<(Vec<SearchableEntry>, Vec<SearchableEntry>)> {
    let searchable = searchable?;
    if query.is_empty() {
        return None;
    }
    Some((
        filter_search_results(query, searchable),
        help_search
            .map(|list| filter_search_results(query, list))
            .unwrap_or_default(),
    ))
}

fn visible_items(
    items: &[MenuItem],
    menu_results: &[SearchableEntry],
    help_results: &[SearchableEntry],
    menu_scroll: usize,
    help_scroll: usize,
) -> Vec<MenuItem> {
    let mut out = Vec::new();
    let mut saw_search = false;
    for item in items {
        if matches!(item, MenuItem::SearchField { .. }) {
            out.push(item.clone());
            saw_search = true;
        }
    }
    if !saw_search {
        out.push(MenuItem::SearchField {
            placeholder: "Buscar".to_string(),
        });
    }

    if !menu_results.is_empty() {
        out.push(MenuItem::Header {
            label: "Ítems del menú".to_string(),
            count: Some(menu_results.len()),
        });
        for entry in menu_results
            .iter()
            .skip(menu_scroll)
            .take(MenuView::MAX_MENU_ROWS)
        {
            out.push(MenuItem::SearchResult {
                path: entry.path.clone(),
                item: Box::new(entry.item.clone()),
                source: SearchSource::Menu,
                location: entry.location.clone(),
            });
        }
    }

    if !help_results.is_empty() {
        out.push(MenuItem::Header {
            label: "Temas de ayuda".to_string(),
            count: Some(help_results.len()),
        });
        for entry in help_results
            .iter()
            .skip(help_scroll)
            .take(MenuView::MAX_HELP_ROWS)
        {
            out.push(MenuItem::SearchResult {
                path: entry.path.clone(),
                item: Box::new(entry.item.clone()),
                source: SearchSource::Help,
                location: None,
            });
        }
    }

    if !help_results.is_empty() || !menu_results.is_empty() {
        out.push(MenuItem::Separator);
        out.push(MenuItem::SearchResult {
            path: "Temas de ayuda".to_string(),
            item: Box::new(MenuItem::Action {
                id: "app-help-topics-index".to_string(),
                label: "Mostrar todos los temas de ayuda".to_string(),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            }),
            source: SearchSource::Help,
            location: None,
        });
    }

    out.push(MenuItem::Separator);
    for item in items {
        if !matches!(item, MenuItem::SearchField { .. }) {
            out.push(item.clone());
        }
    }
    out
}

fn find_result_section(items: &[MenuItem], source: SearchSource) -> Option<(usize, usize)> {
    let start = items.iter().position(|item| {
        matches!(item,
            MenuItem::SearchResult { source: s, .. } if *s == source
        )
    })?;
    let count = items
        .iter()
        .skip(start)
        .take_while(|item| {
            matches!(item,
                MenuItem::SearchResult { source: s, .. } if *s == source
            )
        })
        .count();
    Some((start, count))
}

fn item_top(index: usize, items: &[MenuItem], m: MenuPalette) -> f32 {
    let mut y = 0.0;
    for (i, item) in items.iter().enumerate() {
        if i == index {
            return y;
        }
        y += item.height(m);
    }
    0.0
}

fn submenu_positioner(opens_left: bool) -> (PopupAnchor, PopupGravity, i32) {
    if opens_left {
        (PopupAnchor::TopLeft, PopupGravity::BottomLeft, 0)
    } else {
        (PopupAnchor::TopRight, PopupGravity::BottomRight, 0)
    }
}

fn submenu_anchor_rect(
    index: usize,
    items: &[MenuItem],
    m: MenuPalette,
    parent_width: f32,
    shift: f32,
) -> (i32, i32, u32, u32) {
    (
        0,
        (item_top(index, items, m) + m.padding - shift) as i32,
        parent_width.max(1.0) as u32,
        m.item_height as u32,
    )
}

fn close_child(
    open_child_id: Rc<Cell<Option<PopupId>>>,
    open_child_index: Rc<Cell<Option<usize>>>,
) {
    if let Some(id) = open_child_id.get() {
        if let Some(popups) = try_consume_root_context::<Popups>() {
            popups.close(id);
        }
        open_child_id.set(None);
    }
    open_child_index.set(None);
}

fn submenu_contents(id: Option<&str>, items: &[MenuItem]) -> Vec<MenuItem> {
    if id.is_some() && items.is_empty() {
        vec![MenuItem::Action {
            id: String::new(),
            label: "Cargando…".to_string(),
            shortcut: None,
            enabled: false,
            check: CheckState::None,
            icon: None,
        }]
    } else {
        items.to_vec()
    }
}

fn announce_submenu(on_open: &Option<Rc<dyn Fn(String)>>, id: Option<&str>) {
    if let (Some(callback), Some(id)) = (on_open, id) {
        callback(id.to_string());
    }
}

fn child_already_open(
    has_child: bool,
    open_child_index: &Cell<Option<usize>>,
    index: usize,
) -> bool {
    has_child && open_child_index.get() == Some(index)
}

#[allow(clippy::too_many_arguments)]
fn open_child_popup(
    index: usize,
    id: Option<&str>,
    sub_items: &[MenuItem],
    popup: PopupHandle,
    parent_size: (f32, f32),
    palette_source: PaletteSource,
    on_activate: Rc<dyn Fn(String)>,
    on_dismiss_all: Rc<dyn Fn()>,
    on_open_submenu: Option<Rc<dyn Fn(String)>>,
    submenu_source: Option<SubmenuSource>,
    open_child_id: Rc<Cell<Option<PopupId>>>,
    open_child_index: Rc<Cell<Option<usize>>>,
    parent_items: Rc<Vec<MenuItem>>,
    opens_left: bool,
) -> Option<PopupId> {
    if child_already_open(open_child_id.get().is_some(), &open_child_index, index) {
        return open_child_id.get();
    }
    close_child(open_child_id.clone(), open_child_index.clone());
    announce_submenu(&on_open_submenu, id);
    let m = palette_source.current();
    let initial_items = submenu_contents(id, sub_items);
    let child_size = MenuView::preferred_size(&initial_items, m);
    let on_dismiss_for_action = on_dismiss_all.clone();
    let mut child_view = MenuView::new(
        initial_items.clone(),
        move |id| {
            on_activate(id);
            on_dismiss_for_action();
        },
        move || on_dismiss_all(),
    );
    child_view = palette_source.inherit(child_view);
    if let (Some(source), Some(id)) = (&submenu_source, id) {
        let source = source.clone();
        let id = id.to_string();
        let fallback = initial_items;
        child_view = child_view.with_items_source(move || {
            source(&id)
                .filter(|items| !items.is_empty())
                .unwrap_or_else(|| fallback.clone())
        });
    }
    if let Some(callback) = &on_open_submenu {
        let callback = callback.clone();
        child_view = child_view.with_open_submenu(move |id| callback(id));
    }
    if let Some(source) = &submenu_source {
        let source = source.clone();
        child_view = child_view.with_submenu_source(move |id| source(id));
    }
    let open_child_id_set = open_child_id.clone();
    let open_child_index_set = open_child_index.clone();
    let dismissed_id = Rc::new(Cell::new(None::<PopupId>));
    let on_dismissed = {
        let dismissed_id = dismissed_id.clone();
        move || {
            let expected = dismissed_id.get();
            let current = open_child_id_set.get();
            if expected == current {
                open_child_id_set.set(None);
                open_child_index_set.set(None);
            }
        }
    };
    let inset = m.padding;
    let (anchor, gravity, dx) = submenu_positioner(opens_left);
    let spec = PopupSpec {
        anchor_rect: submenu_anchor_rect(
            index,
            &parent_items,
            m,
            parent_size.0,
            popup.scroll_shift(),
        ),
        anchor,
        gravity,
        offset: (dx, -(inset as i32)),
        size: (child_size.0 as u32, child_size.1 as u32),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_x: true,
            slide_y: true,
            resize_y: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: true,
        on_dismissed: Some(Rc::new(on_dismissed)),
    };
    child_view = child_view.with_popup_spec(spec.clone());
    let id = popup.open_child(spec, AppComponent::new(child_view));
    dismissed_id.set(Some(id));
    open_child_id.set(Some(id));
    open_child_index.set(Some(index));
    Some(id)
}

#[allow(clippy::too_many_arguments)]
fn open_reveal_popup(
    items: &[MenuItem],
    location: &SearchLocation,
    popup: PopupHandle,
    parent_size: (f32, f32),
    palette_source: PaletteSource,
    on_activate: Rc<dyn Fn(String)>,
    on_dismiss_all: Rc<dyn Fn()>,
    reveal_child_id: Rc<Cell<Option<PopupId>>>,
    opens_left: bool,
    anchor_x: f32,
    anchor_width: f32,
) -> Option<PopupId> {
    close_child(reveal_child_id.clone(), Rc::new(Cell::new(None)));
    let m = palette_source.current();
    let child_size = MenuView::preferred_size(items, m);
    let parent_width = parent_size.0;
    let parent_left = if opens_left {
        anchor_x + anchor_width - parent_width
    } else {
        anchor_x
    };
    let reveal_x = parent_left - child_size.0 - 8.0;
    if reveal_x < 0.0 {
        return None;
    }
    let on_dismiss_for_action = on_dismiss_all.clone();
    let focused_index = location.indices.first().copied();
    let mut child_view = MenuView::new(
        items.to_vec(),
        move |id| {
            on_activate(id);
            on_dismiss_for_action();
        },
        move || on_dismiss_all(),
    );
    child_view = palette_source.inherit(child_view);
    let child_view = child_view.with_focused_index(focused_index);
    let reveal_child_id_set = reveal_child_id.clone();
    let on_dismissed = move || {
        reveal_child_id_set.set(None);
    };
    let spec = PopupSpec {
        anchor_rect: (0, 0, 1, 1),
        anchor: PopupAnchor::TopLeft,
        gravity: PopupGravity::BottomLeft,
        offset: (-8, 0),
        size: (child_size.0 as u32, child_size.1 as u32),
        constraint_adjustment: PopupConstraintAdjustment::default(),
        grab: false,
        on_dismissed: Some(Rc::new(on_dismissed)),
    };
    let id = popup.open_child(spec, AppComponent::new(child_view));
    reveal_child_id.set(Some(id));
    Some(id)
}

#[allow(clippy::too_many_arguments)]
fn render_item(
    item: &MenuItem,
    index: usize,
    is_focused: bool,
    m: MenuPalette,
    _palette_source: PaletteSource,
    _popup: Option<PopupHandle>,
    _opens_left: bool,
    on_activate: Rc<dyn Fn(String)>,
    on_dismiss_all: Rc<dyn Fn()>,
    items: Rc<Vec<MenuItem>>,
    _focused: UseState<Option<usize>>,
    _open_child_id: Rc<Cell<Option<PopupId>>>,
    _open_child_index: Rc<Cell<Option<usize>>>,
    size: (f32, f32),
    query: UseState<String>,
    selected: UseState<bool>,
    menu_search_icon: Option<&'static str>,
    help_search_icon: Option<&'static str>,
    result_focused: UseState<Option<usize>>,
    _on_open_submenu: Option<Rc<dyn Fn(String)>>,
    _submenu_source: Option<SubmenuSource>,
    on_row_enter: Rc<dyn Fn(usize, CursorPoint)>,
    on_row_move: Rc<dyn Fn(usize, CursorPoint)>,
    on_row_click: Rc<dyn Fn(usize)>,
    show_mnemonic: bool,
) -> Element {
    match item {
        MenuItem::Separator => rect()
            .width(Size::fill())
            .height(Size::px(m.separator_height))
            .margin(Gaps::new(
                m.separator_margin_y,
                0.0,
                m.separator_margin_y,
                0.0,
            ))
            .background(rgba(m.separator_color))
            .into_element(),
        MenuItem::Action {
            id,
            label: label_text,
            shortcut: shortcut_text,
            enabled,
            check,
            icon,
        } => {
            let id = id.clone();
            let label_text = label_text.clone();
            let shortcut_text = shortcut_text.as_ref().map(|s| format_shortcut(s));
            let enabled = *enabled;
            let check = *check;
            let icon = icon.clone();
            let on_row_enter = on_row_enter.clone();
            let on_row_move = on_row_move.clone();
            let shortcut_w = shortcut_text
                .as_ref()
                .map(|s| measure_text(s, m.font_size))
                .unwrap_or(0.0);
            let has_leading = has_leading_column(&items);
            let text_inset = row_text_inset(&items, m);
            let max_label_w = (m.max_width
                - 2.0 * m.padding
                - text_inset
                - m.item_padding_x
                - if shortcut_w > 0.0 {
                    m.shortcut_gap + shortcut_w
                } else {
                    0.0
                })
            .max(0.0);
            let label_text = truncate_middle(&label_text, max_label_w, m.font_size);
            let fg = if enabled {
                if is_focused { m.hover_fg } else { m.fg }
            } else {
                m.disabled_fg
            };
            let shortcut_color = shortcut_fg(enabled, is_focused, m);
            let bg = if is_focused && enabled {
                Some(m.hover_bg)
            } else {
                None
            };
            let mut row = rect()
                .width(Size::fill())
                .height(Size::px(m.item_height))
                .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
                .corner_radius(m.item_radius)
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .on_pointer_enter(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_enter(index, event.global_location());
                    }
                })
                .on_pointer_move(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_move(index, event.global_location());
                    }
                })
                .on_mouse_down(move |_: Event<MouseEventData>| {
                    if enabled {
                        on_activate(id.clone());
                        on_dismiss_all();
                    }
                });
            if let Some(bg) = bg {
                row = row.background(rgba(bg));
            }
            let mut left = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center);
            if has_leading {
                left = left.spacing(MARK_TEXT_GAP).child(leading_slot(
                    icon.clone(),
                    check,
                    fg,
                    shortcut_fg(enabled, is_focused, m),
                    m,
                ));
            }
            left = left.child(item_label(&label_text, show_mnemonic, m.font_size, fg));
            row.child(left)
                .child(
                    label()
                        .text(shortcut_text.unwrap_or_default())
                        .font_size(m.font_size)
                        .color(rgba(shortcut_color))
                        .width(Size::px(shortcut_w))
                        .max_lines(1),
                )
                .into_element()
        }
        MenuItem::Submenu {
            id: _id,
            label: label_text,
            enabled,
            items: _sub_items,
            icon,
        } => {
            let label_text = label_text.clone();
            let enabled = *enabled;
            let icon = icon.clone();
            let on_row_click = on_row_click.clone();
            let fg = if enabled {
                if is_focused { m.hover_fg } else { m.fg }
            } else {
                m.disabled_fg
            };
            let bg = if is_focused && enabled {
                Some(m.hover_bg)
            } else {
                None
            };
            let has_leading = has_leading_column(&items);
            let mut row = rect()
                .width(Size::fill())
                .height(Size::px(m.item_height))
                .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
                .corner_radius(m.item_radius)
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .on_pointer_enter(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_enter(index, event.global_location());
                    }
                })
                .on_pointer_move(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_move(index, event.global_location());
                    }
                })
                .on_mouse_down(move |_: Event<MouseEventData>| {
                    if enabled {
                        on_row_click(index);
                    }
                });
            if let Some(bg) = bg {
                row = row.background(rgba(bg));
            }
            let mut left = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center);
            if has_leading {
                left = left.spacing(MARK_TEXT_GAP).child(leading_slot(
                    icon.clone(),
                    CheckState::None,
                    fg,
                    shortcut_fg(enabled, is_focused, m),
                    m,
                ));
            }
            left = left.child(item_label(&label_text, show_mnemonic, m.font_size, fg));
            row.child(left)
                .child(chevron_element(chevron_fg(enabled, is_focused, m), m))
                .into_element()
        }
        MenuItem::Header {
            label: header,
            count,
        } => {
            let mut row = rect()
                .width(Size::fill())
                .height(Size::px(m.header_height))
                .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::End)
                .child(
                    label()
                        .text(header.clone())
                        .font_size(m.header_font_size)
                        .font_weight(m.header_weight)
                        .color(rgba(m.disabled_fg))
                        .max_lines(1),
                );
            if let Some(count) = count {
                row = row.child(
                    label()
                        .text(format!("{count}"))
                        .font_size(m.header_font_size)
                        .font_weight(m.header_weight)
                        .color(rgba(m.disabled_fg))
                        .max_lines(1),
                );
            }
            row.into_element()
        }
        MenuItem::SearchField { placeholder } => {
            let mut query = query;
            let mut selected = selected;
            let query_text = (*query.read()).clone();
            let is_selected = *selected.read();
            let has_text = !query_text.is_empty();
            let display_text = if has_text {
                query_text.clone()
            } else {
                placeholder.clone()
            };
            let text_color = if has_text { m.fg } else { m.shortcut_color };
            let clear_size = 16.0f32;
            let clear_margin = 8.0f32;
            let clear_gap = if has_text { clear_size + 8.0 } else { 0.0 };
            let row_padding = m.item_padding_x;
            let available_w = (size.0
                - 2.0 * m.padding
                - row_padding
                - clear_margin
                - m.icon_size
                - 8.0
                - clear_gap)
                .max(0.0);

            let mut text_box = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .width(Size::px(available_w))
                .overflow(Overflow::Clip)
                .child(
                    label()
                        .text(display_text)
                        .font_size(m.font_size)
                        .color(rgba(text_color))
                        .max_lines(1),
                );

            if has_text && !is_selected {
                text_box = text_box.child(
                    rect()
                        .width(Size::px(1.0))
                        .height(Size::px(m.font_size))
                        .margin(Gaps::new(0.0, 1.0, 0.0, 0.0))
                        .background(rgba(m.fg))
                        .into_element(),
                );
            }

            let mut content = rect()
                .width(Size::fill())
                .height(Size::px(m.search_height))
                .padding(Gaps::new(0.0, clear_margin, 0.0, row_padding))
                .background(rgba(m.search_bg))
                .corner_radius(m.search_radius)
                .border(
                    Border::new()
                        .width(m.search_ring_width)
                        .alignment(BorderAlignment::Inner)
                        .fill(rgba(m.search_ring)),
                )
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .cross_align(Alignment::Center)
                        .spacing(8.0)
                        .child(
                            symbol(Symbol::MagnifyingGlass, m.icon_size, m.shortcut_color)
                                .into_element(),
                        )
                        .child(text_box),
                );

            if has_text {
                let clear = rect()
                    .width(Size::px(clear_size))
                    .height(Size::px(clear_size))
                    .corner_radius(clear_size / 2.0)
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Center)
                    .background(rgba(m.shortcut_color.mix(m.bg, 0.7)))
                    .child(
                        label()
                            .text("×")
                            .font_size(12.0)
                            .color(rgba(m.fg))
                            .max_lines(1),
                    )
                    .on_mouse_down(move |_: Event<MouseEventData>| {
                        query.set(String::new());
                        selected.set(false);
                    });
                content = content.child(clear);
            }

            content
                .margin(Gaps::new(m.search_margin_y, 0.0, m.search_margin_y, 0.0))
                .into_element()
        }
        MenuItem::SearchResult {
            path: _,
            item,
            source,
            location: _,
        } => {
            let MenuItem::Action {
                id,
                label: label_text,
                shortcut: shortcut_text,
                enabled,
                ..
            } = item.as_ref()
            else {
                return rect().into_element();
            };
            let mut result_focused = result_focused;
            let id = id.clone();
            let label_text = label_text.clone();
            let shortcut_text = shortcut_text.as_ref().map(|s| format_shortcut(s));
            let enabled = *enabled;
            let fg = if enabled {
                if is_focused { m.hover_fg } else { m.fg }
            } else {
                m.disabled_fg
            };
            let shortcut_color = shortcut_fg(enabled, is_focused, m);
            let bg = if is_focused && enabled {
                Some(m.hover_bg)
            } else {
                None
            };
            let shortcut_w = shortcut_text
                .as_ref()
                .map(|s| measure_text(s, m.font_size))
                .unwrap_or(0.0);
            let mut row = rect()
                .width(Size::fill())
                .height(Size::px(m.item_height))
                .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
                .corner_radius(m.item_radius)
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .on_pointer_enter(move |_| {
                    if enabled {
                        result_focused.set(Some(index));
                    }
                })
                .on_pointer_move(move |_| {
                    if enabled {
                        result_focused.set(Some(index));
                    }
                })
                .on_mouse_down(move |_: Event<MouseEventData>| {
                    if enabled {
                        on_activate(id.clone());
                        on_dismiss_all();
                    }
                });
            if let Some(bg) = bg {
                row = row.background(rgba(bg));
            }
            let icon_element = search_icon_element(
                *source,
                enabled,
                is_focused,
                fg,
                m.hover_bg,
                menu_search_icon,
                help_search_icon,
                m,
            );
            let left = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(MARK_TEXT_GAP)
                .child(icon_element)
                .child(
                    label()
                        .text(strip_mnemonic(&label_text))
                        .font_size(m.font_size)
                        .color(rgba(fg))
                        .max_lines(1),
                );
            row.child(left)
                .child(
                    label()
                        .text(shortcut_text.unwrap_or_default())
                        .font_size(m.font_size)
                        .color(rgba(shortcut_color))
                        .width(Size::px(shortcut_w))
                        .max_lines(1),
                )
                .into_element()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn search_icon_element(
    source: SearchSource,
    enabled: bool,
    focused: bool,
    fg: Rgba,
    accent: Rgba,
    menu_icon: Option<&'static str>,
    help_icon: Option<&'static str>,
    m: MenuPalette,
) -> Element {
    match source {
        SearchSource::Menu => {
            let text_color = if focused { m.hover_fg } else { fg };
            let accent_color = if focused {
                m.hover_fg
            } else if enabled {
                accent
            } else {
                fg
            };
            let svg = search_icon_svg(menu_icon.unwrap_or(""), text_color, accent_color);
            rect()
                .width(Size::px(16.0))
                .direction(Direction::Horizontal)
                .main_align(Alignment::Center)
                .child(
                    SvgViewer::new(freya_core::prelude::Bytes::from(svg.into_bytes()))
                        .width(Size::px(16.0))
                        .height(Size::px(14.0))
                        .show_loader(false)
                        .into_element(),
                )
                .into_element()
        }
        SearchSource::Help => {
            let svg = help_icon.unwrap_or("").to_string();
            let color = if focused { m.hover_fg } else { fg };
            rect()
                .width(Size::px(16.0))
                .direction(Direction::Horizontal)
                .main_align(Alignment::Center)
                .child(
                    SvgViewer::new(freya_core::prelude::Bytes::from(svg.into_bytes()))
                        .width(Size::px(16.0))
                        .height(Size::px(16.0))
                        .color(rgba(color))
                        .show_loader(false)
                        .into_element(),
                )
                .into_element()
        }
    }
}

fn leading_slot(
    icon: Option<MenuIcon>,
    check: CheckState,
    fg: Rgba,
    icon_fg: Rgba,
    m: MenuPalette,
) -> Element {
    if !matches!(check, CheckState::None) {
        return mark_element(check, fg, m);
    }
    let icon_elem = match icon {
        Some(MenuIcon::Svg(svg)) => SvgViewer::new(Bytes::from(svg.as_bytes().to_vec()))
            .width(Size::px(m.icon_size))
            .height(Size::px(m.icon_size))
            .color(rgba(icon_fg))
            .show_loader(false)
            .into_element(),
        Some(MenuIcon::Image(bytes)) => icon_image(&bytes, m.icon_size).map_or_else(
            || {
                rect()
                    .width(Size::px(m.icon_size))
                    .height(Size::px(m.icon_size))
                    .into_element()
            },
            |handle| {
                image(handle)
                    .width(Size::px(m.icon_size))
                    .height(Size::px(m.icon_size))
                    .into_element()
            },
        ),
        None => rect()
            .width(Size::px(m.icon_size))
            .height(Size::px(m.icon_size))
            .into_element(),
    };
    rect()
        .width(Size::px(m.leading_width))
        .direction(Direction::Horizontal)
        .main_align(Alignment::Center)
        .child(icon_elem)
        .into_element()
}

thread_local! {
    static IMAGE_CACHE: RefCell<HashMap<u64, Option<ImageHandle>>> = RefCell::new(HashMap::new());
}

fn icon_image(bytes: &[u8], size: f32) -> Option<ImageHandle> {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    size.to_bits().hash(&mut hasher);
    let key = hasher.finish();
    IMAGE_CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(&key) {
            return cached.clone();
        }
        let decoded = decode_icon(bytes, size);
        if decoded.is_none() {
            tracing::debug!(
                bytes = bytes.len(),
                "menu: icon-data could not be decoded; dropping the icon"
            );
        }
        cache.borrow_mut().insert(key, decoded.clone());
        decoded
    })
}

fn decode_icon(bytes: &[u8], size: f32) -> Option<ImageHandle> {
    let data = SkData::new_copy(bytes);
    let image = SkImage::from_encoded(data)?;
    let image = image.make_raster_image(None, None).unwrap_or(image);
    let target = size.max(1.0).round() as i32;
    let info = ImageInfo::new(
        (target, target),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut pixels = vec![0u8; (target * target * 4) as usize];
    let pixmap = Pixmap::new(&info, &mut pixels, info.min_row_bytes())?;
    if !image.scale_pixels(&pixmap, SamplingMode::Bilinear.sampling_options(), None) {
        return None;
    }
    drop(pixmap);
    ImageHandle::from_rgba(
        target as u32,
        target as u32,
        Bytes::from(pixels),
        AlphaType::Premul,
    )
}

fn mark_element(check: CheckState, color: Rgba, m: MenuPalette) -> Element {
    match check {
        CheckState::None | CheckState::Checkbox(false) | CheckState::Radio(false) => {
            rect().width(Size::px(m.leading_width)).into_element()
        }
        CheckState::Checkbox(true) => label()
            .text("✓")
            .font_size(m.mark_check)
            .color(rgba(color))
            .width(Size::px(m.leading_width))
            .text_align(TextAlign::Center)
            .max_lines(1)
            .into_element(),
        CheckState::Radio(true) => rect()
            .width(Size::px(m.leading_width))
            .direction(Direction::Horizontal)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .width(Size::px(m.mark_dot))
                    .height(Size::px(m.mark_dot))
                    .corner_radius(m.mark_dot / 2.0)
                    .background(rgba(color)),
            )
            .into_element(),
    }
}

fn chevron_element(color: Rgba, m: MenuPalette) -> Element {
    label()
        .text("›")
        .font_size(m.font_size)
        .color(rgba(color))
        .width(Size::px(m.chevron_size))
        .max_lines(1)
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::cell::RefCell;
    use std::rc::Rc;

    use freya_core::elements::label::LabelElement;
    use freya_core::integration::AppComponent;
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;
    use vow_design::{MENU_DARK, MENU_LIGHT};

    use crate::platform::{ObjectId, PopupCommand, PopupHandle, PopupId, Popups};

    fn sample() -> Vec<MenuItem> {
        vec![
            MenuItem::Action {
                id: "new".to_string(),
                label: "New Window".to_string(),
                shortcut: Some("Ctrl+N".to_string()),
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
            MenuItem::Separator,
            MenuItem::Action {
                id: "open".to_string(),
                label: "Open".to_string(),
                shortcut: None,
                enabled: false,
                check: CheckState::Checkbox(true),
                icon: None,
            },
            MenuItem::Submenu {
                id: None,
                label: "Recent".to_string(),
                enabled: true,
                icon: None,
                items: vec![MenuItem::Action {
                    id: "r1".to_string(),
                    label: "a.txt".to_string(),
                    shortcut: None,
                    enabled: true,
                    check: CheckState::None,
                    icon: None,
                }],
            },
        ]
    }

    #[test]
    fn preferred_size_is_positive() {
        let (w, h) = MenuView::preferred_size(&sample(), MENU_LIGHT);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }

    #[test]
    fn popover_metrics_keep_the_live_menu_colours() {
        use vow_appearance::{
            AppearanceConfig, ColorScheme, WindowColorMode, WindowTintScope, resolve,
        };

        let accent = Rgba {
            r: 0.9,
            g: 0.2,
            b: 0.7,
            a: 1.0,
        };
        let dark = resolve(
            &AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                reduce_transparency: true,
                ..Default::default()
            },
            None,
        );
        let tinted = resolve(
            &AppearanceConfig {
                color_scheme: ColorScheme::Dark,
                accent_color: Some([accent.r, accent.g, accent.b]),
                reduce_transparency: true,
                window_color: WindowColorMode::Accent,
                window_tint_scope: WindowTintScope::Desktop,
                ..Default::default()
            },
            None,
        );

        for resolved in [dark, tinted] {
            let expected = menu_palette(&resolved, SurfaceKind::Toplevel);
            let view = MenuView::new(sample(), |_| {}, || {})
                .with_surface(SurfaceKind::Toplevel)
                .with_popover_metrics();
            let actual = view.palette_source.with_resolved(&resolved);
            assert_eq!(actual.bg, expected.bg);
            assert_eq!(actual.separator_color, expected.separator_color);
        }
    }

    #[test]
    fn root_box_follows_the_configured_popup_size() {
        use freya_testing::TestingRunner;
        use torin::prelude::Size2D;

        let items: Vec<MenuItem> = std::iter::once(MenuItem::SearchField {
            placeholder: "Buscar".to_string(),
        })
        .chain((0..3).map(|i| MenuItem::Action {
            id: format!("item-{i}"),
            label: format!("Item {i}"),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
        }))
        .collect();
        assert_eq!(MenuView::preferred_size(&items, MENU_LIGHT), (220.0, 116.0));

        let view = MenuView::new(items, |_| {}, || {}).with_fixed_palette(MENU_LIGHT);
        let app = AppComponent::new(view);
        let (mut test, _) = TestingRunner::new(app, Size2D::new(340.0, 300.0), |_| {}, 1.0);
        test.sync_and_update();

        let area = test
            .find(|node, element| {
                let style = element.style();
                (style.background == Fill::Color(rgba(MENU_LIGHT.bg))).then(|| node.layout().area)
            })
            .expect("menu root box");
        assert_eq!((area.size.width, area.size.height), (340.0, 300.0));
    }

    #[test]
    fn preferred_size_accounts_for_separators() {
        let with_sep = sample();
        let without_sep: Vec<MenuItem> = with_sep
            .iter()
            .filter(|item| !matches!(item, MenuItem::Separator))
            .cloned()
            .collect();
        let (_, h_with) = MenuView::preferred_size(&with_sep, MENU_LIGHT);
        let (_, h_without) = MenuView::preferred_size(&without_sep, MENU_LIGHT);
        assert!(h_with > h_without);
    }

    #[test]
    fn navigation_skips_separators_and_disabled_items() {
        let items = sample();
        assert_eq!(first_selectable(&items), Some(0));
        assert_eq!(next_selectable(&items, Some(0), 1), Some(3));
        assert_eq!(next_selectable(&items, Some(3), 1), Some(0));
        assert_eq!(last_selectable(&items), Some(3));
    }

    fn action(label: &str) -> MenuItem {
        MenuItem::Action {
            id: format!("id:{label}"),
            label: label.to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
        }
    }

    #[test]
    fn separators_at_the_start_and_end_are_dropped() {
        let mut items = vec![
            MenuItem::Separator,
            action("A"),
            MenuItem::Separator,
            MenuItem::Separator,
            action("B"),
            MenuItem::Separator,
        ];
        normalize_separators(&mut items);
        assert_eq!(
            items.iter().filter_map(MenuItem::label).collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }

    #[test]
    fn separator_runs_collapse_into_one() {
        let mut items = vec![
            action("A"),
            MenuItem::Separator,
            MenuItem::Separator,
            MenuItem::Separator,
            action("B"),
        ];
        normalize_separators(&mut items);
        assert_eq!(items.len(), 3);
        assert!(matches!(items[1], MenuItem::Separator));
    }

    #[test]
    fn separators_are_normalized_inside_submenus() {
        let mut items = vec![MenuItem::Submenu {
            id: None,
            label: "Sub".to_string(),
            enabled: true,
            items: vec![
                MenuItem::Separator,
                action("A"),
                MenuItem::Separator,
                MenuItem::Separator,
            ],
            icon: None,
        }];
        normalize_separators(&mut items);
        let MenuItem::Submenu { items, .. } = &items[0] else {
            panic!("the submenu survives");
        };
        assert_eq!(
            items.iter().filter_map(MenuItem::label).collect::<Vec<_>>(),
            vec!["A"]
        );
    }

    #[test]
    fn a_menu_of_only_separators_becomes_empty() {
        let mut items = vec![
            MenuItem::Separator,
            MenuItem::Separator,
            MenuItem::Separator,
        ];
        normalize_separators(&mut items);
        assert!(items.is_empty());
    }

    #[test]
    fn an_empty_menu_shows_the_placeholder() {
        let rows = with_empty_hint(Vec::new());
        assert!(matches!(
            rows.as_slice(),
            [MenuItem::Action {
                label,
                enabled: false,
                ..
            }] if label == "Sin elementos"
        ));
        assert_eq!(
            MenuView::preferred_size(&[], MENU_LIGHT),
            MenuView::preferred_size(&rows, MENU_LIGHT)
        );
    }

    #[test]
    fn the_placeholder_is_not_selectable_or_activatable() {
        let rows = with_empty_hint(Vec::new());
        assert!(!rows[0].selectable());
        assert_eq!(first_selectable(&rows), None);
        assert_eq!(next_selectable(&rows, None, 1), None);
        assert_eq!(last_selectable(&rows), None);
        assert_eq!(activate_at(&rows, 0), None);
    }

    #[test]
    fn the_placeholder_is_not_in_the_search_results() {
        let searched = visible_items(&[], &[], &[], 0, 0);
        assert!(
            !searched
                .iter()
                .any(|item| item.label() == Some("Sin elementos"))
        );
    }

    #[test]
    fn live_items_replace_the_placeholder() {
        assert_eq!(
            with_empty_hint(Vec::new())[0].label(),
            Some("Sin elementos")
        );
        assert_eq!(
            with_empty_hint(vec![action("Guardar")])[0].label(),
            Some("Guardar")
        );
    }

    #[test]
    fn empty_remote_submenu_shows_a_disabled_loading_row() {
        let loading = submenu_contents(Some("7"), &[]);
        assert_eq!(loading.len(), 1);
        assert!(matches!(
            &loading[0],
            MenuItem::Action {
                label,
                enabled: false,
                ..
            } if label == "Cargando…"
        ));
        assert!(submenu_contents(None, &[]).is_empty());
    }

    #[test]
    fn opening_a_remote_submenu_notifies_once() {
        let calls = Rc::new(Cell::new(0));
        let counter = calls.clone();
        let callback: Option<Rc<dyn Fn(String)>> = Some(Rc::new(move |_| {
            counter.set(counter.get() + 1);
        }));
        let open_index = Cell::new(None::<usize>);
        for _ in 0..2 {
            if !child_already_open(true, &open_index, 2) {
                announce_submenu(&callback, Some("7"));
                open_index.set(Some(2));
            }
        }
        assert_eq!(calls.get(), 1);
        assert!(!child_already_open(true, &open_index, 3));
        assert!(!child_already_open(false, &open_index, 2));
        announce_submenu(&callback, None);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn activate_at_ignores_disabled_and_submenus() {
        let items = sample();
        assert_eq!(activate_at(&items, 0), Some("new".to_string()));
        assert_eq!(activate_at(&items, 1), None);
        assert_eq!(activate_at(&items, 2), None);
        assert_eq!(activate_at(&items, 3), None);
    }

    #[test]
    fn navigation_skips_headers_and_search_fields() {
        let items = vec![
            MenuItem::SearchField {
                placeholder: "Buscar".to_string(),
            },
            MenuItem::Header {
                label: "Mitades".to_string(),
                count: None,
            },
            MenuItem::Action {
                id: "a".to_string(),
                label: "A".to_string(),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
            MenuItem::Header {
                label: "Cuartos".to_string(),
                count: None,
            },
            MenuItem::SearchField {
                placeholder: "Filtrar".to_string(),
            },
            MenuItem::Submenu {
                id: None,
                label: "B".to_string(),
                enabled: true,
                items: Vec::new(),
                icon: None,
            },
        ];
        assert_eq!(first_selectable(&items), Some(2));
        assert_eq!(next_selectable(&items, Some(2), 1), Some(5));
        assert_eq!(next_selectable(&items, Some(5), 1), Some(2));
        assert_eq!(last_selectable(&items), Some(5));
    }

    #[test]
    fn icons_widen_the_menu_by_the_leading_column_and_gap() {
        let label = "Configuración avanzada del sistema";
        let built = |icon: Option<MenuIcon>| {
            vec![
                MenuItem::Action {
                    id: "a".to_string(),
                    label: label.to_string(),
                    shortcut: Some("Super+,".to_string()),
                    enabled: true,
                    check: CheckState::None,
                    icon: icon.clone(),
                },
                MenuItem::Submenu {
                    id: None,
                    label: "Disposición".to_string(),
                    enabled: true,
                    items: Vec::new(),
                    icon,
                },
            ]
        };
        let plain = MenuView::preferred_size(&built(None), MENU_LIGHT);
        let with_icons = MenuView::preferred_size(&built(Some(MenuIcon::svg("x"))), MENU_LIGHT);
        assert_eq!(
            with_icons.0 - plain.0,
            MENU_LIGHT.leading_width + MARK_TEXT_GAP,
            "an icon column is only reserved when the menu actually uses one"
        );
    }

    #[test]
    fn left_handed_submenus_keep_their_descendants_on_the_left() {
        let (anchor, gravity, dx) = submenu_positioner(true);
        assert_eq!(anchor, PopupAnchor::TopLeft);
        assert_eq!(gravity, PopupGravity::BottomLeft);
        assert_eq!(dx, 0, "the panel-edge anchor needs no horizontal offset");

        let mut parent_left = 1000.0;
        for _ in 0..3 {
            let child_right = parent_left + dx as f32;
            assert!(
                child_right <= parent_left,
                "a left-handed child overlaps its ancestor"
            );
            parent_left = child_right;
        }

        let (anchor, gravity, dx) = submenu_positioner(false);
        assert_eq!(anchor, PopupAnchor::TopRight);
        assert_eq!(gravity, PopupGravity::BottomRight);
        assert_eq!(dx, 0);
    }

    #[test]
    fn submenu_anchor_rect_matches_the_row_top() {
        let items = vec![
            MenuItem::Action {
                id: "one".to_string(),
                label: "One".to_string(),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
            MenuItem::Separator,
            MenuItem::Action {
                id: "two".to_string(),
                label: "Two".to_string(),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                id: None,
                label: "More".to_string(),
                enabled: true,
                icon: None,
                items: vec![MenuItem::Action {
                    id: "deep".to_string(),
                    label: "Deep".to_string(),
                    shortcut: None,
                    enabled: true,
                    check: CheckState::None,
                    icon: None,
                }],
            },
        ];
        let m = MENU_LIGHT;
        let (width, _) = MenuView::preferred_size(&items, m);
        let (x, y, w, h) = submenu_anchor_rect(4, &items, m, width, 0.0);
        let row_top = item_top(4, &items, m) + m.padding;
        assert_eq!(y as f32, row_top);
        assert_eq!(x, 0);
        assert_eq!(w as f32, width);
        assert_eq!(h as f32, m.item_height);

        let (_, _, dx) = submenu_positioner(false);
        let inset = m.padding;
        assert_eq!(dx, 0);
        let child_top = row_top - inset;
        let child_first_row_top = child_top + inset;
        assert_eq!(child_first_row_top, row_top);
    }

    #[test]
    fn shortcuts_use_macos_glyphs_in_order() {
        assert_eq!(format_shortcut("Ctrl+Shift+G"), "⌃⇧G");
        assert_eq!(format_shortcut("Alt+Inicio"), "⌥Inicio");
        assert_eq!(format_shortcut("Super+L"), "❖L");
        assert_eq!(format_shortcut("Ctrl+Shift+Z"), "⌃⇧Z");
        assert_eq!(format_shortcut("Shift+Ctrl+G"), "⌃⇧G");
        assert_eq!(format_shortcut("Cmd+D"), "❖D");
        assert_eq!(format_shortcut("Alt+Super+Escape"), "⌥❖⎋");
        assert!(!format_shortcut("Command+D").contains('\u{2318}'));
        assert_eq!(format_shortcut("F5"), "F5");
        assert_eq!(format_shortcut("Ctrl+Super+Left"), "⌃❖←");
        assert_eq!(format_shortcut("Ctrl+Shift+Super+Right"), "⌃⇧❖→");
        assert_eq!(format_shortcut("Up"), "↑");
        assert_eq!(format_shortcut("Down"), "↓");
        assert_eq!(format_shortcut("Return"), "↩");
        assert_eq!(format_shortcut("Enter"), "↩");
        assert_eq!(format_shortcut("Ctrl+Tab"), "⌃⇥");
    }

    #[test]
    fn mark_sizes_fit_the_leading_column() {
        for (name, m) in [("light", MENU_LIGHT), ("dark", MENU_DARK)] {
            assert!(
                (6.0..=8.0).contains(&m.mark_dot),
                "{name}: radio dot {} is outside 6-8 px",
                m.mark_dot
            );
            assert!(
                m.mark_dot <= m.leading_width,
                "{name}: radio dot is wider than its column"
            );
            assert!(
                m.mark_check > 0.0 && m.mark_check <= m.leading_width,
                "{name}: check glyph does not fit its column"
            );
        }
    }

    #[test]
    fn focused_shortcut_and_chevron_keep_4_5_contrast() {
        for (name, m) in [("light", MENU_LIGHT), ("dark", MENU_DARK)] {
            for color in [
                shortcut_fg(true, true, m),
                chevron_fg(true, true, m),
                m.hover_fg,
            ] {
                let ratio = color.contrast_ratio(m.hover_bg);
                assert!(
                    ratio >= 4.5,
                    "{name}: focused row color has {ratio:.2}:1 contrast on accent"
                );
            }
        }
    }

    #[test]
    fn preferred_size_handles_accents_and_long_shortcuts() {
        let items = vec![
            MenuItem::Action {
                id: "images".to_string(),
                label: "Imágenes".to_string(),
                shortcut: Some("Ctrl+Shift+G".to_string()),
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
            MenuItem::Action {
                id: "videos".to_string(),
                label: "Vídeos".to_string(),
                shortcut: Some("Ctrl+Shift+Z".to_string()),
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
        ];
        let m = MENU_LIGHT;
        let (w, h) = MenuView::preferred_size(&items, m);

        let label_w = measure_text("Imágenes", m.font_size);
        let shortcut_w = measure_text(&format_shortcut("Ctrl+Shift+G"), m.font_size);
        let expected_min =
            m.item_padding_x * 2.0 + label_w + m.shortcut_gap + shortcut_w + m.padding * 2.0;
        assert!(
            w >= expected_min,
            "menu width {w} should fit the widest accented label + shortcut ({expected_min})"
        );

        let expected_height = m.item_height * 2.0 + m.padding * 2.0;
        assert!(
            h >= expected_height,
            "menu height {h} should fit two rows ({expected_height})"
        );
    }

    fn label_x(runner: &TestingRunner, text: &str) -> f32 {
        runner
            .find(|node, element| {
                (element as &dyn Any)
                    .downcast_ref::<LabelElement>()
                    .filter(|label| label.text == text)
                    .map(|_| node.layout().area.origin.x)
            })
            .unwrap_or_else(|| panic!("label '{text}' not found"))
    }

    fn action_with(label: &str, check: CheckState, icon: Option<MenuIcon>) -> MenuItem {
        MenuItem::Action {
            id: format!("id-{label}"),
            label: label.to_string(),
            shortcut: None,
            enabled: true,
            check,
            icon,
        }
    }

    #[test]
    fn plain_menu_text_starts_at_item_padding() {
        let items = vec![action("Solo texto")];
        let (runner, _) = make_runner(items);
        let m = MENU_LIGHT;
        let x = label_x(&runner, "Solo texto");
        let expected = m.padding + m.item_padding_x;
        assert!(
            (x - expected).abs() <= 1.0,
            "plain text starts at {x}, expected {expected}"
        );
    }

    #[test]
    fn checkbox_false_shifts_text_by_leading_column_and_gap() {
        let plain = vec![action("Solo texto")];
        let checked = vec![action_with("Solo texto", CheckState::Checkbox(false), None)];
        let (plain_runner, _) = make_runner(plain);
        let (checked_runner, _) = make_runner(checked);
        let diff = label_x(&checked_runner, "Solo texto") - label_x(&plain_runner, "Solo texto");
        let expected = MENU_LIGHT.leading_width + MARK_TEXT_GAP;
        assert!(
            (diff - expected).abs() <= 1.0,
            "checkbox adds {diff}, expected {expected}"
        );
    }

    #[test]
    fn icon_shifts_text_by_leading_column_and_gap() {
        let plain = vec![action("Solo texto")];
        let with_icon = vec![action_with(
            "Solo texto",
            CheckState::None,
            Some(MenuIcon::svg("x")),
        )];
        let (plain_runner, _) = make_runner(plain);
        let (icon_runner, _) = make_runner(with_icon);
        let diff = label_x(&icon_runner, "Solo texto") - label_x(&plain_runner, "Solo texto");
        let expected = MENU_LIGHT.leading_width + MARK_TEXT_GAP;
        assert!(
            (diff - expected).abs() <= 1.0,
            "icon adds {diff}, expected {expected}"
        );
    }

    #[test]
    fn submenu_keeps_its_own_inset() {
        let child = vec![action_with("Hijo", CheckState::Checkbox(false), None)];
        let parent = vec![MenuItem::Submenu {
            id: None,
            label: "Padre".to_string(),
            enabled: true,
            icon: None,
            items: child.clone(),
        }];
        let m = MENU_LIGHT;
        assert_eq!(row_text_inset(&parent, m), m.item_padding_x);
        assert_eq!(
            row_text_inset(&child, m),
            m.item_padding_x + m.leading_width + MARK_TEXT_GAP
        );
    }

    #[test]
    fn menu_without_columns_is_narrower_by_leading_column_and_gap() {
        let label = "Opción bastante larga del menú sin columnas";
        let plain = vec![action(label)];
        let with_columns = vec![action_with(
            label,
            CheckState::Checkbox(false),
            Some(MenuIcon::svg("x")),
        )];
        let m = MENU_LIGHT;
        let (plain_w, _) = MenuView::preferred_size(&plain, m);
        let (columns_w, _) = MenuView::preferred_size(&with_columns, m);
        assert_eq!(
            columns_w - plain_w,
            m.leading_width + MARK_TEXT_GAP,
            "plain menu should be narrower by the single unused leading column plus gap"
        );
    }

    #[test]
    fn menu_with_mark_or_icon_keeps_full_inset() {
        let items = vec![action_with(
            "Mixto",
            CheckState::Checkbox(true),
            Some(MenuIcon::svg("x")),
        )];
        let (runner, _) = make_runner(items);
        let m = MENU_LIGHT;
        let x = label_x(&runner, "Mixto");
        let expected = m.padding + m.item_padding_x + m.leading_width + MARK_TEXT_GAP;
        assert!(
            (x - expected).abs() <= 1.0,
            "full-inset text starts at {x}, expected {expected}"
        );
    }

    #[test]
    fn mark_takes_precedence_over_icon_in_leading_column() {
        let plain = vec![action_with("X", CheckState::Checkbox(true), None)];
        let with_icon = vec![action_with(
            "X",
            CheckState::Checkbox(true),
            Some(MenuIcon::svg("x")),
        )];
        let (plain_runner, _) = make_runner(plain);
        let (icon_runner, _) = make_runner(with_icon);
        assert_eq!(
            label_x(&plain_runner, "X"),
            label_x(&icon_runner, "X"),
            "a checked row with an icon must align exactly like a checked row without one"
        );
    }

    fn searchable(label: &str, enabled: bool) -> SearchableEntry {
        SearchableEntry {
            path: "Ventana".to_string(),
            item: MenuItem::Action {
                id: format!("id-{label}"),
                label: label.to_string(),
                shortcut: None,
                enabled,
                check: CheckState::None,
                icon: None,
            },
            location: None,
            source: SearchSource::Menu,
        }
    }

    #[test]
    fn search_orders_enabled_before_disabled_and_word_start_before_substring() {
        let entries = vec![
            searchable("zzz foo", false),
            searchable("foo bar", true),
            searchable("bar foo", true),
            searchable("afoo", true),
        ];
        let results = filter_search_results("foo", &entries);
        let labels: Vec<&str> = results.iter().map(|e| e.item.label().unwrap()).collect();
        assert_eq!(labels, vec!["foo bar", "bar foo", "afoo", "zzz foo"]);
    }

    #[test]
    fn visible_items_limits_menu_results_to_nine_rows() {
        let items = vec![
            MenuItem::SearchField {
                placeholder: "Buscar".to_string(),
            },
            MenuItem::Action {
                id: "report".to_string(),
                label: "Informar".to_string(),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            },
        ];
        let searchable: Vec<SearchableEntry> = (0..24)
            .map(|i| searchable(&format!("result {i}"), true))
            .collect();
        let visible = visible_items(&items, &searchable, &[], 0, 0);
        let menu_header = visible.iter().find(
            |item| matches!(item, MenuItem::Header { label, .. } if label == "Ítems del menú"),
        );
        assert!(menu_header.is_some());
        let count = match menu_header {
            Some(MenuItem::Header { count, .. }) => *count,
            _ => None,
        };
        assert_eq!(count, Some(24));
        let menu_result_count = visible
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    MenuItem::SearchResult {
                        source: SearchSource::Menu,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(menu_result_count, MenuView::MAX_MENU_ROWS);
    }

    fn png_16() -> Vec<u8> {
        let mut pixels = vec![0u8; 16 * 16 * 4];
        for pixel in pixels.chunks_mut(4) {
            pixel.copy_from_slice(&[255, 0, 0, 255]);
        }
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, 16, 16);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&pixels).unwrap();
        }
        out
    }

    #[test]
    fn a_16px_png_icon_is_decoded_scaled_and_cached() {
        let bytes = png_16();
        let handle = icon_image(&bytes, 16.0).expect("the PNG decodes");
        assert_eq!(handle.image.width(), 16);
        assert_eq!(handle.image.height(), 16);
        assert_eq!(&handle.bytes[..4], &[255, 0, 0, 255]);

        let again = icon_image(&bytes, 16.0).expect("the cached PNG decodes");
        assert!(handle == again, "the cache returns the same Skia image");

        let too_big = vec![0u8; MenuIcon::MAX_IMAGE_BYTES + 1];
        assert!(MenuIcon::image(too_big).is_none());
        assert!(decode_icon(b"not a png", 16.0).is_none());
    }

    #[test]
    fn visible_items_scrolls_to_keep_twelfth_result_visible() {
        let items = vec![MenuItem::SearchField {
            placeholder: "Buscar".to_string(),
        }];
        let searchable: Vec<SearchableEntry> = (0..24)
            .map(|i| searchable(&format!("result {i}"), true))
            .collect();
        let scroll = 11;
        let visible = visible_items(&items, &searchable, &[], scroll, 0);
        let first_label = visible
            .iter()
            .find_map(|item| match item {
                MenuItem::SearchResult { item, .. } => item.label(),
                _ => None,
            })
            .unwrap();
        assert_eq!(first_label, "result 11");
        let menu_result_count = visible
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    MenuItem::SearchResult {
                        source: SearchSource::Menu,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(menu_result_count, MenuView::MAX_MENU_ROWS);
    }

    fn submenu_test(label: &str) -> MenuItem {
        MenuItem::Submenu {
            id: None,
            label: label.to_string(),
            enabled: true,
            icon: None,
            items: vec![MenuItem::Action {
                id: format!("{label}-child"),
                label: format!("{label} child"),
                shortcut: None,
                enabled: true,
                check: CheckState::None,
                icon: None,
            }],
        }
    }

    fn drain_commands(queue: &Rc<RefCell<Vec<PopupCommand>>>) -> Vec<PopupCommand> {
        queue.borrow_mut().drain(..).collect()
    }

    type Opens = Vec<(PopupId, Option<Rc<dyn Fn()>>)>;

    fn opens_and_dismissals(commands: &[PopupCommand]) -> (Opens, Vec<PopupId>) {
        let mut opens = Opens::new();
        let mut closes = Vec::new();
        for cmd in commands {
            match cmd {
                PopupCommand::Open { id, spec, .. } => {
                    opens.push((*id, spec.on_dismissed.clone()));
                }
                PopupCommand::Close { id } => closes.push(*id),
                PopupCommand::Reposition { .. } => {}
            }
        }
        (opens, closes)
    }

    fn make_runner(items: Vec<MenuItem>) -> (TestingRunner, Rc<RefCell<Vec<PopupCommand>>>) {
        let palette = MENU_LIGHT;
        let size = MenuView::preferred_size(&items, palette);
        let app =
            AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
        let owner = ObjectId::null();
        let root_id = PopupId::test(0);
        let (popups, queue) = Popups::for_test(owner, Some(root_id));
        let (mut runner, _) = TestingRunner::new(
            app,
            Size2D::new(size.0, size.1),
            |r| {
                let popups = popups.clone();
                let handle = PopupHandle::for_test(popups.clone(), root_id);
                r.provide_root_context(move || popups.clone());
                r.provide_root_context(move || handle.clone());
            },
            1.0,
        );
        runner.sync_and_update();
        (runner, queue)
    }

    #[test]
    fn stale_submenu_dismissal_does_not_orphan_sibling_child() {
        let items = vec![submenu_test("A"), submenu_test("B")];
        let (mut runner, queue) = make_runner(items);
        let palette = MENU_LIGHT;
        let size = MenuView::preferred_size(&[submenu_test("A"), submenu_test("B")], palette);
        let x = size.0 / 2.0;
        let row0_y = palette.padding + palette.item_height / 2.0;
        let row1_y = palette.padding + palette.item_height + palette.item_height / 2.0;

        runner.move_cursor([x as f64, row0_y as f64]);
        runner.sync_and_update();
        let mut commands = drain_commands(&queue);
        let (mut opens, closes) = opens_and_dismissals(&commands);
        assert!(closes.is_empty(), "first open should not close anything");
        assert_eq!(opens.len(), 1, "first hover should open one child");
        let (id0, dismiss0) = opens.remove(0);

        runner.move_cursor([x as f64, row1_y as f64]);
        runner.sync_and_update();
        commands = drain_commands(&queue);
        let (opens, closes) = opens_and_dismissals(&commands);
        assert!(
            closes.contains(&id0),
            "moving to sibling must close old child"
        );
        assert_eq!(opens.len(), 1, "second hover should open one child");
        let (id1, _) = opens.into_iter().next().unwrap();
        assert_ne!(id0, id1, "new child must have a different id");

        if let Some(dismiss) = dismiss0 {
            dismiss();
        }

        runner.move_cursor([x as f64, row0_y as f64]);
        runner.sync_and_update();
        let commands = drain_commands(&queue);
        let (_, closes) = opens_and_dismissals(&commands);
        assert!(
            closes.contains(&id1),
            "stale dismissal must not orphan the sibling submenu"
        );
    }
}
