use super::*;

use vow_design::MENU_MARK_TEXT_GAP;

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct LeadingColumns {
    pub(super) marks: bool,
    pub(super) icons: bool,
}

impl LeadingColumns {
    pub(super) fn of(items: &[MenuItem]) -> Self {
        let mut columns = Self::default();
        for item in items {
            match item {
                MenuItem::Action { check, icon, .. } => {
                    columns.marks |= !matches!(check, CheckState::None);
                    columns.icons |= icon.is_some();
                }
                MenuItem::Submenu { icon, .. } => columns.icons |= icon.is_some(),
                MenuItem::SearchResult { item, .. } => {
                    let inner = Self::of(std::slice::from_ref(item));
                    columns.marks |= inner.marks;
                    columns.icons |= inner.icons;
                }
                _ => {}
            }
        }
        columns
    }

    pub(super) fn any(self) -> bool {
        self.marks || self.icons
    }

    fn count(self) -> f32 {
        f32::from(u8::from(self.marks) + u8::from(self.icons))
    }
}

pub(super) fn row_text_inset(items: &[MenuItem], m: MenuPalette) -> f32 {
    m.item_padding_x + LeadingColumns::of(items).count() * (m.leading_width + MENU_MARK_TEXT_GAP)
}

pub(crate) fn last_leading_column_inset(items: &[MenuItem], m: MenuPalette) -> f32 {
    row_text_inset(items, m) - m.leading_width - MENU_MARK_TEXT_GAP
}

pub(super) fn svg_rgb(color: Rgba) -> String {
    let [r, g, b, _] = color.to_rgba8();
    format!("rgb({r},{g},{b})")
}

pub fn search_icon_svg(template: &str, text: Rgba, accent: Rgba) -> String {
    template
        .replace("{{text}}", &svg_rgb(text))
        .replace("{{accent}}", &svg_rgb(accent))
}

pub(super) fn row_fg(enabled: bool, focused: bool, idle: Rgba, m: MenuPalette) -> Rgba {
    if !enabled {
        m.disabled_fg
    } else if focused {
        m.hover_fg
    } else {
        idle
    }
}

pub(super) fn trailing_width(item: &MenuItem, m: MenuPalette) -> f32 {
    if matches!(item, MenuItem::Submenu { .. }) {
        return m.chevron_size;
    }
    match item.shortcut() {
        Some(shortcut) => m.shortcut_gap + measure_text(&shortcut, m.font_size),
        None => 0.0,
    }
}

pub(super) struct RowCtx {
    pub(super) m: MenuPalette,
    pub(super) columns: LeadingColumns,
    pub(super) text_inset: f32,
    pub(super) size: (f32, f32),
    pub(super) query: UseState<String>,
    pub(super) selected: UseState<bool>,
    pub(super) result_focused: UseState<Option<usize>>,
    pub(super) menu_search_icon: Option<&'static str>,
    pub(super) help_search_icon: Option<&'static str>,
    pub(super) on_activate: Rc<dyn Fn(String)>,
    pub(super) on_dismiss_all: Rc<dyn Fn()>,
    pub(super) on_row_enter: Rc<dyn Fn(usize, CursorPoint)>,
    pub(super) on_row_move: Rc<dyn Fn(usize, CursorPoint)>,
    pub(super) on_row_click: Rc<dyn Fn(usize)>,
    pub(super) show_mnemonic: bool,
}

impl RowCtx {
    fn fitted_label(&self, item: &MenuItem) -> String {
        let m = self.m;
        let max_width = m.max_width
            - 2.0 * m.padding
            - self.text_inset
            - m.item_padding_x
            - trailing_width(item, m);
        truncate_middle(
            item.label().unwrap_or_default(),
            max_width.max(0.0),
            m.font_size,
        )
    }

    fn activate(&self, id: &str) -> impl Fn(Event<MouseEventData>) + 'static {
        let (id, on_activate, on_dismiss_all) = (
            id.to_string(),
            self.on_activate.clone(),
            self.on_dismiss_all.clone(),
        );
        move |_| {
            on_activate(id.clone());
            on_dismiss_all();
        }
    }

    fn shortcut_label(&self, item: &MenuItem, color: Rgba) -> Element {
        let shortcut = item.shortcut().unwrap_or_default();
        label()
            .width(Size::px(measure_text(&shortcut, self.m.font_size)))
            .text(shortcut)
            .font_size(self.m.font_size)
            .color(theme::color(color))
            .max_lines(1)
            .into_element()
    }
}

fn row_frame(m: MenuPalette, highlighted: bool) -> Rect {
    let row = rect()
        .width(Size::fill())
        .height(Size::px(m.item_height))
        .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
        .corner_radius(m.item_radius)
        .direction(Direction::Horizontal)
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center);
    if highlighted {
        row.background(theme::color(m.hover_bg))
    } else {
        row
    }
}

pub(super) fn render_item(
    ctx: &RowCtx,
    item: &MenuItem,
    index: usize,
    is_focused: bool,
) -> Element {
    let m = ctx.m;
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
            .background(theme::color(m.separator_color))
            .into_element(),
        MenuItem::Action { enabled, icon, .. } | MenuItem::Submenu { enabled, icon, .. } => {
            let enabled = *enabled;
            let fg = row_fg(enabled, is_focused, m.fg, m);
            let shortcut_color = row_fg(enabled, is_focused, m.shortcut_color, m);
            let (on_row_enter, on_row_move) = (ctx.on_row_enter.clone(), ctx.on_row_move.clone());
            let mut row = row_frame(m, is_focused && enabled)
                .on_pointer_enter(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_enter(index, event.global_location());
                    }
                })
                .on_pointer_move(move |event: Event<PointerEventData>| {
                    if enabled {
                        on_row_move(index, event.global_location());
                    }
                });
            let (check, trailing) = match item {
                MenuItem::Action { id, check, .. } => {
                    if enabled {
                        row = row.on_mouse_down(ctx.activate(id));
                    }
                    (*check, ctx.shortcut_label(item, shortcut_color))
                }
                _ => {
                    let on_row_click = ctx.on_row_click.clone();
                    if enabled {
                        row =
                            row.on_mouse_down(move |_: Event<MouseEventData>| on_row_click(index));
                    }
                    (
                        CheckState::None,
                        chevron_element(row_fg(enabled, is_focused, m.chevron_color, m), m),
                    )
                }
            };
            let mut left = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center);
            if ctx.columns.any() {
                left = left.spacing(MENU_MARK_TEXT_GAP).child(leading_slot(
                    icon.clone(),
                    check,
                    ctx.columns,
                    fg,
                    shortcut_color,
                    m,
                ));
            }
            left = left.child(item_label(
                &ctx.fitted_label(item),
                ctx.show_mnemonic,
                m.font_size,
                fg,
            ));
            row.child(left).child(trailing).into_element()
        }
        MenuItem::Header {
            label: header,
            count,
        } => {
            let header_label = |text: String| {
                label()
                    .text(text)
                    .font_size(m.header_font_size)
                    .font_weight(m.header_weight)
                    .color(theme::color(m.disabled_fg))
                    .max_lines(1)
            };
            rect()
                .width(Size::fill())
                .height(Size::px(m.header_height))
                .padding(Gaps::new(0.0, m.item_padding_x, 0.0, m.item_padding_x))
                .direction(Direction::Horizontal)
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::End)
                .child(header_label(header.clone()))
                .maybe_child(count.map(|count| header_label(count.to_string())))
                .into_element()
        }
        MenuItem::SearchField { placeholder } => search_field(ctx, placeholder),
        MenuItem::SearchResult {
            item: inner,
            source,
            ..
        } => {
            let MenuItem::Action { id, enabled, .. } = inner.as_ref() else {
                return rect().into_element();
            };
            let enabled = *enabled;
            let fg = row_fg(enabled, is_focused, m.fg, m);
            let (mut on_enter, mut on_move) = (ctx.result_focused, ctx.result_focused);
            let mut row = row_frame(m, is_focused && enabled)
                .on_pointer_enter(move |_| {
                    if enabled {
                        on_enter.set(Some(index));
                    }
                })
                .on_pointer_move(move |_| {
                    if enabled {
                        on_move.set(Some(index));
                    }
                });
            if enabled {
                row = row.on_mouse_down(ctx.activate(id));
            }
            let left = rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(MENU_MARK_TEXT_GAP)
                .child(search_icon_element(ctx, *source, enabled, is_focused, fg))
                .child(
                    label()
                        .text(strip_mnemonic(&ctx.fitted_label(item)))
                        .font_size(m.font_size)
                        .color(theme::color(fg))
                        .max_lines(1),
                );
            row.child(left)
                .child(ctx.shortcut_label(item, row_fg(enabled, is_focused, m.shortcut_color, m)))
                .into_element()
        }
    }
}

fn search_field(ctx: &RowCtx, placeholder: &str) -> Element {
    let m = ctx.m;
    let (mut query, mut selected) = (ctx.query, ctx.selected);
    let query_text = (*query.read()).clone();
    let is_selected = *selected.read();
    let has_text = !query_text.is_empty();
    let display_text = if has_text {
        query_text.clone()
    } else {
        placeholder.to_string()
    };
    let text_color = if has_text { m.fg } else { m.shortcut_color };
    let clear_size = vow_design::MENU_SEARCH_CLEAR;
    let clear_margin = vow_design::MENU_SEARCH_CLEAR_MARGIN;
    let clear_gap = if has_text {
        clear_size + clear_margin
    } else {
        0.0
    };
    let row_padding = m.item_padding_x;
    let available_w = (ctx.size.0
        - 2.0 * m.padding
        - row_padding
        - clear_margin
        - m.icon_size
        - vow_design::MENU_SEARCH_ICON_GAP
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
                .color(theme::color(text_color))
                .max_lines(1),
        );

    if has_text && !is_selected {
        text_box = text_box.child(
            rect()
                .width(Size::px(vow_design::HAIRLINE))
                .height(Size::px(m.font_size))
                .margin(Gaps::new(0.0, 1.0, 0.0, 0.0))
                .background(theme::color(m.fg))
                .into_element(),
        );
    }

    let mut content = rect()
        .width(Size::fill())
        .height(Size::px(m.search_height))
        .padding(Gaps::new(0.0, clear_margin, 0.0, row_padding))
        .background(theme::color(m.search_bg))
        .corner_radius(m.search_radius)
        .border(
            Border::new()
                .width(m.search_ring_width)
                .alignment(BorderAlignment::Inner)
                .fill(theme::color(m.search_ring)),
        )
        .direction(Direction::Horizontal)
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(vow_design::MENU_SEARCH_ICON_GAP)
                .child(
                    symbol(Symbol::MagnifyingGlass, m.icon_size, m.shortcut_color).into_element(),
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
            .background(theme::color(m.shortcut_color.mix(m.bg, 0.7)))
            .child(
                label()
                    .text("×")
                    .font_size(vow_design::MENU_SEARCH_CLEAR_GLYPH)
                    .color(theme::color(m.fg))
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

fn search_icon_element(
    ctx: &RowCtx,
    source: SearchSource,
    enabled: bool,
    focused: bool,
    fg: Rgba,
) -> Element {
    let m = ctx.m;
    let color = if focused { m.hover_fg } else { fg };
    let (svg, [width, height], tint) = match source {
        SearchSource::Menu => {
            let accent = if enabled && !focused {
                m.hover_bg
            } else {
                color
            };
            let svg = search_icon_svg(ctx.menu_search_icon.unwrap_or(""), color, accent);
            (svg, vow_design::MENU_SEARCH_ICON, None)
        }
        SearchSource::Help => {
            let side = vow_design::MENU_HELP_ICON;
            let svg = ctx.help_search_icon.unwrap_or("").to_string();
            (svg, [side, side], Some(theme::color(color)))
        }
    };
    let mut icon = SvgViewer::new(Bytes::from(svg.into_bytes()))
        .width(Size::px(width))
        .height(Size::px(height))
        .show_loader(false);
    if let Some(tint) = tint {
        icon = icon.color(tint);
    }
    rect()
        .width(Size::px(width))
        .direction(Direction::Horizontal)
        .main_align(Alignment::Center)
        .child(icon.into_element())
        .into_element()
}

pub(super) fn leading_slot(
    icon: Option<MenuIcon>,
    check: CheckState,
    columns: LeadingColumns,
    fg: Rgba,
    icon_fg: Rgba,
    m: MenuPalette,
) -> Element {
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(MENU_MARK_TEXT_GAP)
        .maybe_child(columns.marks.then(|| mark_element(check, fg, m)))
        .maybe_child(columns.icons.then(|| icon_slot(icon, icon_fg, m)))
        .into_element()
}

fn icon_slot(icon: Option<MenuIcon>, icon_fg: Rgba, m: MenuPalette) -> Element {
    let side = Size::px(m.icon_size);
    let blank = || rect().width(side.clone()).height(side.clone());
    let icon_elem = match icon {
        Some(MenuIcon::Svg(svg)) => SvgViewer::new(Bytes::from(svg.as_bytes().to_vec()))
            .width(side.clone())
            .height(side.clone())
            .color(theme::color(icon_fg))
            .show_loader(false)
            .into_element(),
        Some(MenuIcon::Image(bytes)) => match icon_image(&bytes, m.icon_size) {
            Some(handle) => image(handle)
                .width(side.clone())
                .height(side.clone())
                .into_element(),
            None => blank().into_element(),
        },
        Some(MenuIcon::Swatch(fill)) => blank()
            .corner_radius(m.icon_size / 2.0)
            .background(theme::color(fill))
            .into_element(),
        None => blank().into_element(),
    };
    rect()
        .width(Size::px(m.leading_width))
        .direction(Direction::Horizontal)
        .main_align(Alignment::Center)
        .child(icon_elem)
        .into_element()
}

// The menubar lives for the whole session and receives icon-data from every app.
const ICON_CACHE_CAPACITY: usize = 128;

type IconCache = HashMap<(Arc<[u8]>, u32), Option<ImageHandle>>;

thread_local! {
    static IMAGE_CACHE: RefCell<IconCache> = RefCell::new(HashMap::new());
}

pub(super) fn icon_image(bytes: &Arc<[u8]>, size: f32) -> Option<ImageHandle> {
    let key = (bytes.clone(), size.to_bits());
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
        let mut cache = cache.borrow_mut();
        if cache.len() >= ICON_CACHE_CAPACITY {
            cache.clear();
        }
        cache.insert(key, decoded.clone());
        decoded
    })
}

pub(super) fn decode_icon(bytes: &[u8], size: f32) -> Option<ImageHandle> {
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

pub(super) fn mark_element(check: CheckState, color: Rgba, m: MenuPalette) -> Element {
    match check {
        CheckState::None | CheckState::Checkbox(false) | CheckState::Radio(false) => {
            rect().width(Size::px(m.leading_width)).into_element()
        }
        CheckState::Checkbox(true) => label()
            .text("✓")
            .font_size(m.mark_check)
            .color(theme::color(color))
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
                    .background(theme::color(color)),
            )
            .into_element(),
    }
}

pub(super) fn chevron_element(color: Rgba, m: MenuPalette) -> Element {
    label()
        .text("›")
        .font_size(m.font_size)
        .color(theme::color(color))
        .width(Size::px(m.chevron_size))
        .max_lines(1)
        .into_element()
}
