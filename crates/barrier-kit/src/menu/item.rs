use super::*;

pub(super) const MARK_TEXT_GAP: f32 = 6.0;

pub(super) fn has_leading_column(items: &[MenuItem]) -> bool {
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

pub(super) fn row_text_inset(items: &[MenuItem], m: MenuPalette) -> f32 {
    if has_leading_column(items) {
        m.item_padding_x + m.leading_width + MARK_TEXT_GAP
    } else {
        m.item_padding_x
    }
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

pub(super) fn shortcut_fg(enabled: bool, focused: bool, m: MenuPalette) -> Rgba {
    if !enabled {
        m.disabled_fg
    } else if focused {
        m.hover_fg
    } else {
        m.shortcut_color
    }
}

pub(super) fn chevron_fg(enabled: bool, focused: bool, m: MenuPalette) -> Rgba {
    if !enabled {
        m.disabled_fg
    } else if focused {
        m.hover_fg
    } else {
        m.chevron_color
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_item(
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
            .background(theme::color(m.separator_color))
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
                row = row.background(theme::color(bg));
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
                        .color(theme::color(shortcut_color))
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
                row = row.background(theme::color(bg));
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
                        .color(theme::color(m.disabled_fg))
                        .max_lines(1),
                );
            if let Some(count) = count {
                row = row.child(
                    label()
                        .text(format!("{count}"))
                        .font_size(m.header_font_size)
                        .font_weight(m.header_weight)
                        .color(theme::color(m.disabled_fg))
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
                        .color(theme::color(text_color))
                        .max_lines(1),
                );

            if has_text && !is_selected {
                text_box = text_box.child(
                    rect()
                        .width(Size::px(1.0))
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
                    .background(theme::color(m.shortcut_color.mix(m.bg, 0.7)))
                    .child(
                        label()
                            .text("×")
                            .font_size(12.0)
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
                row = row.background(theme::color(bg));
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
                        .color(theme::color(fg))
                        .max_lines(1),
                );
            row.child(left)
                .child(
                    label()
                        .text(shortcut_text.unwrap_or_default())
                        .font_size(m.font_size)
                        .color(theme::color(shortcut_color))
                        .width(Size::px(shortcut_w))
                        .max_lines(1),
                )
                .into_element()
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn search_icon_element(
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
                        .color(theme::color(color))
                        .show_loader(false)
                        .into_element(),
                )
                .into_element()
        }
    }
}

pub(super) fn leading_slot(
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
            .color(theme::color(icon_fg))
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

pub(super) fn icon_image(bytes: &[u8], size: f32) -> Option<ImageHandle> {
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
