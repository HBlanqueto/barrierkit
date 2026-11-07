use crate::elements::forward_container;
use ctappearance::{SidebarStyle as SidebarKind, SidebarSurface};
use ctdesign::{BACKDROP, Rgba, SIDEBAR, SidebarPalette, TEXT_FIELD};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::adornments::Avatar;
use crate::chrome::TitlebarLayout;
use crate::controls::inner_border;
use crate::interaction::{control_state, use_interaction};
use crate::platform::WindowControls;
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::styles::Painted;
use crate::styles::{
    ControlState, LayoutDirection, Paints, ShapeStyle, SidebarAccountStyle, SidebarItemStyle,
    SidebarSectionStyle, SidebarStyle, SplitViewStyle, StateValue, SymbolTileStyle, Theme,
    TrafficLightsStyle, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme;
use crate::titlebar::window_drag_handler;
use crate::traffic::TrafficLights;
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

pub(crate) fn over_backdrop(surface: SidebarSurface, backdrop: SidebarBackdrop) -> SidebarSurface {
    let Some(tint) = crate::backdrop::tinted() else {
        return surface;
    };
    let tinted_island = surface.style == SidebarKind::Island && backdrop == SidebarBackdrop::Tinted;
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

pub(crate) fn item_palette(emphasized: bool) -> SidebarPalette {
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

pub struct SymbolTile(Rect, Symbol, Theme);

impl SymbolTile {
    pub fn new(glyph: Symbol, tint: Rgba) -> Self {
        Self::styled(
            glyph,
            Theme::tengen().style(move |s| SymbolTileStyle {
                background: StateValue::new(ShapeStyle::Color(tint)),
                ..s
            }),
        )
    }

    pub fn rim(self, rim: Rgba) -> Self {
        self.style(move |s| SymbolTileStyle {
            border: Some((ShapeStyle::Color(rim), SIDEBAR.rim_width)),
            ..s
        })
    }

    pub fn style(self, restyle: impl Fn(SymbolTileStyle) -> SymbolTileStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle))
    }

    fn styled(glyph: Symbol, theme: Theme) -> Self {
        let style: SymbolTileStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let tile = rect()
            .width(Size::px(style.size))
            .height(Size::px(style.size))
            .corner_radius(style.shape.corner_radius(
                style.size,
                style.size,
                LayoutDirection::LeftToRight,
            ))
            .painted(&paints, &style.background)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .center()
            .child(symbol(
                glyph,
                style.symbol_size,
                paints.color(style.symbol.resolve(ControlState::default())),
            ));
        Self(tile, glyph, theme)
    }
}

forward_container!(SymbolTile);

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
    style: Theme,
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
            style: Theme::tengen(),
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

    pub fn backdrop(self, backdrop: SidebarBackdrop) -> Self {
        match backdrop {
            SidebarBackdrop::Clear => self,
            SidebarBackdrop::Tinted => self.style(|s| {
                s.surface(over_backdrop(
                    crate::theme::resolved().sidebar,
                    SidebarBackdrop::Tinted,
                ))
            }),
        }
    }

    pub fn style(mut self, restyle: impl Fn(SidebarStyle) -> SidebarStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
        let style: SidebarStyle = resolve(&self.style);
        let split: SplitViewStyle = resolve(&Theme::tengen());
        let lights: TrafficLightsStyle = resolve(&Theme::tengen());
        let paints = Paints::current(ControlState::default(), style.effect);
        let (traffic_x, traffic_y) = style.lights_position;

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
        let resting_top = style.padding_top;
        let lights_top = traffic_y + lights.diameter + style.lights_gap;
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
        let rail_progress =
            crate::animation::use_animated(f32::from(u8::from(rail_layout)), style.animation);
        let inset = split.inset;
        let (bar_height, bar_padding_top) = crate::titlebar::sidebar_bar_metrics();
        // With the Rail the traffic lights leave the sidebar, so its first row rises to the titlebar row.
        let row_top = ((bar_padding_top + bar_height - TEXT_FIELD.height) / 2.0 - inset).max(0.0);
        let top_height = top_height + (row_top - top_height) * rail_progress;
        let padding_x = style.padding_x;

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

        let corner_radius = style
            .shape
            .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight);
        let sidebar_body = rect()
            .expanded()
            .direction(Direction::Vertical)
            .painted(&paints, &style.background)
            .a11y_role(AccessibilityRole::Navigation)
            .padding(Gaps::new(
                top_height,
                padding_x,
                style.padding_bottom,
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
            .child(ScrollView::new().children(self.children.iter().cloned()))
            .corner_radius(corner_radius)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .map(style.inner_border.as_ref(), |el, inner| {
                let rim = style.border.as_ref().map_or(0.0, |(_, width)| *width);
                let radius = CornerRadius {
                    top_left: (corner_radius.top_left - rim).max(0.0),
                    top_right: (corner_radius.top_right - rim).max(0.0),
                    bottom_right: (corner_radius.bottom_right - rim).max(0.0),
                    bottom_left: (corner_radius.bottom_left - rim).max(0.0),
                    ..corner_radius
                };
                el.child(
                    rect()
                        .position(
                            Position::new_absolute()
                                .top(0.0)
                                .left(0.0)
                                .right(0.0)
                                .bottom(0.0),
                        )
                        .margin(Gaps::new_all(rim))
                        .corner_radius(radius)
                        .border(inner_border(&paints, inner)),
                )
            });

        let mut root = rect()
            .expanded()
            .child(sidebar_body)
            .maybe_child(style.line.as_ref().map(|(paint, width)| {
                rect()
                    .position(Position::new_absolute().top(0.0).right(0.0))
                    .width(Size::px(*width))
                    .height(Size::percent(100.0))
                    .background(paints.fill(&StateValue::new(paint.clone())))
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
                    .spacing(chrome.rail_spacing)
                    .padding(Gaps::new(
                        traffic_y + overlay_top.unwrap_or_default(),
                        0.0,
                        chrome.rail_padding_bottom,
                        0.0,
                    ))
                    .maybe(overlay_top.is_some(), |el| {
                        el.layer(crate::titlebar::TITLEBAR_OVERLAY_LAYER + 1)
                    })
                    .maybe_child(lights_shown.then(|| {
                        rect()
                            .probe("sidebar-traffic-lights")
                            .opacity(lights_opacity)
                            .child(
                                TrafficLights::for_window(status.activated, fixed).vertical(true),
                            )
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
            root = root.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .top(traffic_y + overlay_top.unwrap_or_default())
                            .left(traffic_x),
                    )
                    .maybe(overlay_top.is_some(), |el| {
                        el.layer(crate::titlebar::TITLEBAR_OVERLAY_LAYER + 1)
                    })
                    .width(Size::px(lights.length()))
                    .height(Size::px(lights.diameter))
                    .probe("sidebar-traffic-lights")
                    .opacity(lights_opacity)
                    .child(TrafficLights::for_window(status.activated, fixed)),
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
    style: Theme,
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
            style: Theme::tengen(),
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

    pub fn emphasized(self, emphasized: bool) -> Self {
        if emphasized {
            self
        } else {
            self.style(|s| SidebarItemStyle {
                selected_background: StateValue::new(ShapeStyle::Color(
                    item_palette(false).selected_bg,
                )),
                selected_label: s.label.clone(),
                ..s
            })
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SidebarItemStyle) -> SidebarItemStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
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
        let style: SidebarItemStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let (background, ink, font) = if self.selected {
            (
                &style.selected_background,
                &style.selected_label,
                style.selected_font,
            )
        } else {
            (&style.background, &style.label, style.font)
        };
        let corner_radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);

        let mut row = interaction
            .attach(rect(), true)
            .width(Size::fill())
            .height(Size::px(style.height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.spacing)
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(corner_radius)
            .painted(&paints, background)
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
            let border = style.selected_tile_border.clone().filter(|_| self.selected);
            row = row.child(SymbolTile::new(glyph, tint).map(border, |tile, border| {
                tile.style(move |s| SymbolTileStyle {
                    border: Some(border.clone()),
                    ..s
                })
            }));
        }

        row = row.child(
            label()
                .text(self.label.clone())
                .font_family(font.family)
                .font_size(font.size as f32)
                .font_weight(FontWeight::from(font.weight as i32))
                .color(theme::color(paints.color(ink.resolve(state))))
                .max_lines(1),
        );

        row = row.map(self.probe, ProbeExt::probe);
        interaction.focus_ring(row, corner_radius.top_left)
    }
}

#[derive(PartialEq)]
pub struct SidebarSection {
    title: String,
    children: Vec<Element>,
    style: Theme,
    key: DiffKey,
}

impl SidebarSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SidebarSectionStyle) -> SidebarSectionStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: SidebarSectionStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let show_chevron = interaction.hovered() || interaction.keyboard_focused();

        let toggle = {
            let mut expanded = expanded;
            move |_| expanded.toggle()
        };

        let mut content_height = use_state(|| 0.0f32);
        let progress =
            crate::animation::use_animated(f32::from(u8::from(*expanded.read())), style.animation);
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
                    .height(Size::px(style.height))
                    .margin(Gaps::new(style.margin_top, 0.0, 0.0, 0.0))
                    .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
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
                            .font_family(style.font.family)
                            .font_size(style.font.size as f32)
                            .font_weight(FontWeight::from(style.font.weight as i32))
                            .color(theme::color(paints.color(style.title.resolve(state))))
                            .max_lines(1),
                    )
                    .maybe(show_chevron, |el| {
                        let chevron = if *expanded.read() {
                            Symbol::ChevronDown
                        } else {
                            Symbol::ChevronRight
                        };
                        el.child(symbol(
                            chevron,
                            style.chevron_size,
                            paints.color(style.chevron.resolve(state)),
                        ))
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
    style: Theme,
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
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SidebarAccountStyle) -> SidebarAccountStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Component for SidebarAccount {
    fn render(&self) -> impl IntoElement {
        let style: SidebarAccountStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        rect()
            .a11y_alt(self.name.clone())
            .width(Size::fill())
            .height(Size::px(style.height))
            .margin(Gaps::new(0.0, 0.0, style.margin_bottom, 0.0))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.spacing)
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .child(Avatar::new(&self.initials))
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .child(text(self.name.clone(), style.name_font, ink(&style.name)))
                    .child(text(
                        self.detail.clone(),
                        style.detail_font,
                        ink(&style.detail),
                    )),
            )
    }
}
