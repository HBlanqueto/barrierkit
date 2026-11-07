use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_appearance::{SidebarStyle, SidebarSurface};
use vow_design::{
    BACKDROP, BODY, DURATION_BASE, Rgba, SETTINGS, SIDEBAR, SidebarPalette, TEXT_FIELD, TITLEBAR,
    TITLEBAR_RAIL,
};

use crate::ProbeExt;
use crate::adornments::avatar;
use crate::chrome::TitlebarLayout;
use crate::interaction::use_interaction;
use crate::platform::WindowControls;
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme;
use crate::titlebar::{window_drag_handler, window_traffic_lights};
use freya_core::prelude::try_consume_root_context;

pub(crate) fn sidebar_palette() -> SidebarPalette {
    let palette = crate::theme::resolved().sidebar.palette;
    if crate::backdrop::tinted().is_none() {
        return palette;
    }
    let ink = Palette::current();
    SidebarPalette {
        item_ink: ink.label,
        item_icon: ink.secondary_label,
        selected_bg: ink.accent,
        selected_ink: ink.on_accent,
        inactive_selected_bg: ink.selected_control,
        inactive_ink: ink.secondary_label,
        hover_bg: ink.control_hover,
        section_ink: ink.secondary_label,
        count_ink: ink.secondary_label,
        line: ink.separator,
        ..palette
    }
}

fn over_backdrop(surface: SidebarSurface, backdrop: SidebarBackdrop) -> SidebarSurface {
    let Some(tint) = crate::backdrop::tinted() else {
        return surface;
    };
    let tinted_island =
        surface.style == SidebarStyle::Island && backdrop == SidebarBackdrop::Tinted;
    let (bg, rim) = if tinted_island {
        (
            Rgba {
                a: BACKDROP.island,
                ..tint.average()
            },
            Palette::current().separator,
        )
    } else {
        (Rgba::TRANSPARENT, Rgba::TRANSPARENT)
    };
    SidebarSurface {
        bg,
        rim,
        inner: Rgba::TRANSPARENT,
        palette: sidebar_palette(),
        ..surface
    }
}

fn item_palette(emphasized: bool) -> SidebarPalette {
    let palette = sidebar_palette();
    if !crate::platform::window_activated() {
        SidebarPalette {
            selected_bg: palette.inactive_selected_bg,
            selected_ink: palette.inactive_ink,
            item_ink: palette.inactive_ink,
            ..palette
        }
    } else if !emphasized {
        SidebarPalette {
            selected_bg: palette.inactive_selected_bg,
            selected_ink: palette.item_ink,
            ..palette
        }
    } else {
        palette
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct SidebarControl {
    pub(crate) visible: State<bool>,
    pub(crate) auto_hidden: State<bool>,
    pub(crate) occupied: State<f32>,
    pub(crate) settled: State<bool>,
    pub(crate) overlaid: State<bool>,
}

impl SidebarControl {
    pub fn occupied(&self) -> f32 {
        *self.occupied.read()
    }

    pub fn settled(&self) -> bool {
        *self.settled.read()
    }

    pub fn toggle(self) {
        let (mut visible, mut auto_hidden) = (self.visible, self.auto_hidden);
        auto_hidden.set(false);
        let shown = *visible.peek();
        visible.set(!shown);
    }

    pub fn dismiss(self) {
        let mut visible = self.visible;
        if *self.overlaid.peek() {
            visible.set(false);
        }
    }
}

pub fn sidebar_control() -> Option<SidebarControl> {
    try_consume_context()
}

pub fn symbol_tile(glyph: Symbol, tint: Rgba, rim: Option<Rgba>) -> Rect {
    let tile = SIDEBAR.app_icon as f32;
    rect()
        .width(Size::px(tile))
        .height(Size::px(tile))
        .corner_radius(SIDEBAR.item_radius as f32)
        .background(theme::color(tint))
        .map(rim, |el, rim| {
            el.border(
                Border::new()
                    .width(SIDEBAR.rim_width)
                    .alignment(BorderAlignment::Inner)
                    .fill(theme::color(rim)),
            )
        })
        .center()
        .child(symbol(
            glyph,
            SIDEBAR.item_icon as f32,
            Palette::current().on_accent,
        ))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarBackdrop {
    #[default]
    Clear,
    Tinted,
}

#[derive(PartialEq)]
pub struct Sidebar {
    children: Vec<Element>,
    traffic_lights: bool,
    backdrop: SidebarBackdrop,
    probe: Option<&'static str>,
    key: DiffKey,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProbeExt for Sidebar {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            traffic_lights: false,
            backdrop: SidebarBackdrop::Clear,
            probe: None,
            key: DiffKey::None,
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_element));
        self
    }

    pub fn traffic_lights(mut self, show: bool) -> Self {
        self.traffic_lights = show;
        self
    }

    pub fn backdrop(mut self, backdrop: SidebarBackdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Sidebar {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for Sidebar {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let surface = over_backdrop(resolved.sidebar, self.backdrop);
        let island = surface.style == SidebarStyle::Island;

        let controls = try_consume_root_context::<WindowControls>();
        let lights_opacity = crate::chrome::lights_opacity();
        let lights_shown = controls
            .as_ref()
            .is_none_or(|controls| controls.traffic_lights_shown())
            && lights_opacity > 0.0;
        let pushed_titlebar = controls
            .as_ref()
            .filter(|controls| *controls.fullscreen_push.read())
            .map(|controls| *controls.overlay_titlebar_height.read());
        let overlay_top = controls.as_ref().and_then(|controls| {
            controls
                .titlebar_overlay_top()
                .map(|top| top - controls.push_offset())
        });
        let resting_top = SETTINGS.sidebar_padding_top as f32;
        let lights_top =
            (SIDEBAR.traffic_y + TITLEBAR.buttons.diameter + SETTINGS.sidebar_lights_gap) as f32;
        let top_height = match (
            lights_shown,
            controls.as_ref().filter(|_| overlay_top.is_some()),
        ) {
            (false, _) => resting_top,
            (true, None) => lights_top,
            (true, Some(controls)) => {
                let progress = *controls.focus_bar_progress.read();
                let revealed_top = match pushed_titlebar {
                    Some(height) if height > 0.0 => resting_top,
                    Some(_) => lights_top,
                    None => lights_top + *controls.focus_bar_height.read(),
                };
                resting_top + (revealed_top - resting_top) * progress
            }
        };
        let chrome = crate::chrome::chrome_style();
        let rail_layout = chrome.layout == TitlebarLayout::Rail;
        let mut in_rail = use_state(|| rail_layout);
        in_rail.set_if_modified(rail_layout);
        let rail_progress = crate::animation::use_switch_progress(in_rail, DURATION_BASE);
        let inset = if island { SIDEBAR.inset as f32 } else { 0.0 };
        let (bar_height, bar_padding_top) = crate::titlebar::sidebar_bar_metrics();
        // With the Rail the traffic lights leave the sidebar, so its first row rises to the titlebar row.
        let row_top = ((bar_padding_top + bar_height - TEXT_FIELD.height) / 2.0 - inset).max(0.0);
        let top_height = top_height + (row_top - top_height) * rail_progress;
        let padding_x = SIDEBAR.padding_x as f32;
        let radius = SIDEBAR.radius as f32;

        let (status, fixed) = controls
            .as_ref()
            .map(|controls| (*controls.status.read(), controls.fixed))
            .unwrap_or_default();
        let drag = window_drag_handler(fixed);

        let lights_column = chrome.unified(true) && self.traffic_lights;
        let reserved = if lights_column {
            chrome.rail_width
        } else {
            0.0
        };

        let mut sidebar_body = rect()
            .expanded()
            .direction(Direction::Vertical)
            .background(theme::color(surface.bg))
            .a11y_role(AccessibilityRole::Navigation)
            .padding(Gaps::new(
                top_height,
                padding_x,
                if island { padding_x } else { 0.0 },
                padding_x + reserved,
            ))
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .top(-top_height - inset)
                            .left(-padding_x),
                    )
                    .width(Size::px(SIDEBAR.width as f32))
                    .height(Size::px(top_height + inset))
                    .probe("sidebar-drag-strip")
                    .on_mouse_down(drag),
            )
            .child(ScrollView::new().children(self.children.iter().cloned()));

        if island {
            sidebar_body = sidebar_body.corner_radius(crate::style::continuous_corner(radius));

            if surface.rim.a > 0.0 {
                sidebar_body = sidebar_body.border(
                    Border::new()
                        .width(SIDEBAR.rim_width)
                        .alignment(BorderAlignment::Inner)
                        .fill(theme::color(surface.rim)),
                );
            }

            if surface.inner.a > 0.0 {
                sidebar_body = sidebar_body.child(
                    rect()
                        .position(
                            Position::new_absolute()
                                .top(0.0)
                                .left(0.0)
                                .right(0.0)
                                .bottom(0.0),
                        )
                        .margin(Gaps::new_all(SIDEBAR.rim_width))
                        .corner_radius(crate::style::continuous_corner(
                            (radius - SIDEBAR.rim_width).max(0.0),
                        ))
                        .border(
                            Border::new()
                                .width(SIDEBAR.inner_width)
                                .alignment(BorderAlignment::Inner)
                                .fill(theme::color(surface.inner)),
                        ),
                );
            }
        }

        let mut root = rect()
            .expanded()
            .child(sidebar_body)
            .maybe_child((!island).then(|| {
                rect()
                    .position(Position::new_absolute().top(0.0).right(0.0))
                    .width(Size::px(SIDEBAR.inner_width))
                    .height(Size::percent(100.0))
                    .background(theme::color(surface.palette.line))
            }));

        let sidebar_shown = sidebar_control().is_none_or(|control| control.settled());
        if lights_column {
            let items = crate::chrome::rail_items();
            let position = Position::new_absolute().top(0.0);
            root = root.child(
                rect()
                    .position(position.left(0.0))
                    .width(Size::px(chrome.rail_width))
                    .height(Size::percent(100.0))
                    .direction(Direction::Vertical)
                    .content(Content::flex())
                    .cross_align(Alignment::Center)
                    .spacing(TITLEBAR.buttons.spacing as f32)
                    .padding(Gaps::new(
                        SIDEBAR.traffic_y as f32 + overlay_top.unwrap_or_default(),
                        0.0,
                        TITLEBAR_RAIL.padding_bottom,
                        0.0,
                    ))
                    .maybe(overlay_top.is_some(), |el| {
                        el.layer(crate::titlebar::TITLEBAR_OVERLAY_LAYER + 1)
                    })
                    .maybe_child(lights_shown.then(|| {
                        rect()
                            .probe("sidebar-traffic-lights")
                            .opacity(lights_opacity)
                            .child(window_traffic_lights(status.activated, fixed).vertical(true))
                    }))
                    .maybe_child(items.top)
                    .child(rect().height(Size::flex(1.0)))
                    .maybe_child(items.bottom),
            );
        } else if self.traffic_lights
            && lights_shown
            && sidebar_shown
            && chrome.layout == TitlebarLayout::Top
        {
            let (_, _, group_w, _) = vow_appearance::traffic::group_rect();

            root = root.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .top(SIDEBAR.traffic_y as f32 + overlay_top.unwrap_or_default())
                            .left(SIDEBAR.traffic_x as f32),
                    )
                    .maybe(overlay_top.is_some(), |el| {
                        el.layer(crate::titlebar::TITLEBAR_OVERLAY_LAYER + 1)
                    })
                    .width(Size::px(group_w as f32))
                    .height(Size::px(TITLEBAR.buttons.diameter as f32))
                    .probe("sidebar-traffic-lights")
                    .opacity(lights_opacity)
                    .child(window_traffic_lights(status.activated, fixed)),
            );
        }

        root.map(self.probe, ProbeExt::probe).into_element()
    }
}

#[derive(PartialEq)]
pub struct SidebarItem {
    label: String,
    symbol: Option<(Symbol, Rgba)>,
    selected: bool,
    emphasized: bool,
    on_select: Option<Callback<(), ()>>,
    probe: Option<&'static str>,
    key: DiffKey,
}

impl ProbeExt for SidebarItem {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl SidebarItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            symbol: None,
            selected: false,
            emphasized: true,
            on_select: None,
            probe: None,
            key: DiffKey::None,
        }
    }

    pub fn symbol(mut self, symbol: Symbol, tint: Rgba) -> Self {
        self.symbol = Some((symbol, tint));
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasized = emphasized;
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SidebarItem {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SidebarItem {
    fn render(&self) -> impl IntoElement {
        let palette = item_palette(self.emphasized);
        let interaction = use_interaction();

        let (bg, fg, weight) = if self.selected {
            (
                theme::color(palette.selected_bg),
                theme::color(palette.selected_ink),
                FontWeight::SEMI_BOLD,
            )
        } else if interaction.hovered() {
            (
                theme::color(palette.hover_bg),
                theme::color(palette.item_ink),
                FontWeight::MEDIUM,
            )
        } else {
            (
                Color::TRANSPARENT,
                theme::color(palette.item_ink),
                FontWeight::MEDIUM,
            )
        };

        let mut row = interaction
            .attach(rect(), true)
            .width(Size::fill())
            .height(Size::px(SIDEBAR.item_height as f32))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SIDEBAR.item_gap as f32)
            .padding(Gaps::new(
                0.0,
                SIDEBAR.item_padding_x as f32,
                0.0,
                SIDEBAR.item_padding_x as f32,
            ))
            .corner_radius(SIDEBAR.item_radius as f32)
            .background(bg)
            .a11y_role(AccessibilityRole::ListItem)
            .a11y_alt(self.label.clone())
            .a11y_builder(move |node| {
                node.set_selected(self.selected);
            })
            .map(self.on_select.clone(), |el, callback| {
                let control = sidebar_control();
                el.on_press(move |_| {
                    callback.call(());
                    if let Some(control) = control {
                        control.dismiss();
                    }
                })
            });

        if let Some((glyph, tint)) = self.symbol {
            row = row.child(symbol_tile(
                glyph,
                tint,
                self.selected.then_some(palette.selected_icon_rim),
            ));
        }

        row = row.child(
            label()
                .text(self.label.clone())
                .font_family(BODY.family)
                .font_size(SIDEBAR.item_text_size as f32)
                .font_weight(weight)
                .color(fg)
                .max_lines(1),
        );

        row = row.map(self.probe, ProbeExt::probe);
        interaction.focus_ring(row, SIDEBAR.item_radius as f32)
    }
}

#[derive(PartialEq)]
pub struct SidebarSection {
    title: String,
    children: Vec<Element>,
    key: DiffKey,
}

impl SidebarSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
            key: DiffKey::None,
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_element));
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SidebarSection {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SidebarSection {
    fn render(&self) -> impl IntoElement {
        let expanded = use_state(|| true);
        let palette = sidebar_palette();
        let interaction = use_interaction();
        let show_chevron = interaction.hovered() || interaction.keyboard_focused();

        let toggle = {
            let mut expanded = expanded;
            move |_| expanded.toggle()
        };

        let mut content_height = use_state(|| 0.0f32);
        let progress = crate::animation::use_switch_progress(expanded, DURATION_BASE);
        let measured_height = *content_height.read();
        let height = progress * measured_height;
        let drawer_offset = -(1.0 - progress) * measured_height;

        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .child(
                interaction
                    .attach(rect(), true)
                    .width(Size::fill())
                    .height(Size::px(SIDEBAR.section_height as f32))
                    .margin(Gaps::new(SIDEBAR.section_margin_top as f32, 0.0, 0.0, 0.0))
                    .padding(Gaps::new(
                        0.0,
                        SIDEBAR.section_padding_x as f32,
                        0.0,
                        SIDEBAR.section_padding_x as f32,
                    ))
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(self.title.clone())
                    .a11y_builder(move |node| {
                        node.set_expanded(*expanded.read());
                    })
                    .on_press(toggle)
                    .child(
                        label()
                            .text(self.title.clone())
                            .font_size(SIDEBAR.section_size as f32)
                            .font_weight(FontWeight::from(SIDEBAR.section_weight as i32))
                            .color(theme::color(palette.section_ink))
                            .max_lines(1),
                    )
                    .maybe(show_chevron, |el| {
                        let chevron = if *expanded.read() {
                            Symbol::ChevronDown
                        } else {
                            Symbol::ChevronRight
                        };
                        el.child(symbol(chevron, SIDEBAR.chevron as f32, palette.section_ink))
                    }),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(height))
                    .offset_y(drawer_offset)
                    .overflow(Overflow::Clip)
                    .direction(Direction::Vertical)
                    .child(
                        rect()
                            .width(Size::fill())
                            .direction(Direction::Vertical)
                            .on_sized(move |event: Event<SizedEventData>| {
                                content_height.set_if_modified(event.data().area.size.height);
                            })
                            .children(self.children.iter().cloned()),
                    ),
            )
            .into_element()
    }
}

#[derive(PartialEq)]
pub struct SidebarAccount {
    initials: String,
    name: String,
    detail: String,
}

impl SidebarAccount {
    pub fn new(
        initials: impl Into<String>,
        name: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            initials: initials.into(),
            name: name.into(),
            detail: detail.into(),
        }
    }
}

impl Component for SidebarAccount {
    fn render(&self) -> impl IntoElement {
        let palette = item_palette(true);
        rect()
            .a11y_alt(self.name.clone())
            .width(Size::fill())
            .height(Size::px(SETTINGS.account_height as f32))
            .margin(Gaps::new(
                0.0,
                0.0,
                SETTINGS.account_margin_bottom as f32,
                0.0,
            ))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SETTINGS.account_gap as f32)
            .padding(Gaps::new(
                0.0,
                SETTINGS.account_padding_x as f32,
                0.0,
                SETTINGS.account_padding_x as f32,
            ))
            .child(avatar(&self.initials))
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .child(text(
                        self.name.clone(),
                        SETTINGS.account_name_font,
                        palette.item_ink,
                    ))
                    .child(text(
                        self.detail.clone(),
                        SETTINGS.account_detail_font,
                        palette.count_ink,
                    )),
            )
    }
}
