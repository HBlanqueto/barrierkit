use crate::elements::{Overrides, forward_container, forward_key, forward_overrides};
use ctappearance::{ColorScheme, SidebarSurface};
use ctdesign::{
    GlassSelection, Rgba, SIDEBAR, SIDEBAR_GLASS_SELECTION_DARK, SIDEBAR_GLASS_SELECTION_LIGHT,
    SidebarPalette,
};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::adornments::Avatar;
use crate::focus::{FocusState, Focused, use_focus_state};
use crate::interaction::{control_state, use_interaction};
use crate::platform::WindowControls;
use crate::scroll::ScrollView;
use crate::style::{Palette, text, text_style};
use crate::styles::Painted;
use crate::styles::{
    ControlState, LayoutDirection, Paints, ShapeStyle, SidebarAccountStyle, SidebarItemStyle,
    SidebarSectionStyle, SidebarStyle, StateValue, SymbolTileStyle, Theme, TrafficLightsStyle,
    resolve,
};
use crate::symbol::{Symbol, stroked_symbol, symbol, tile_symbol};
use crate::theme;
use crate::titlebar::use_window_drag;
use crate::traffic::TrafficLights;
use freya_core::{accessibility::tree::ACCESSIBILITY_ROOT_ID, prelude::try_consume_root_context};

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
        inactive_selected_ink: ink.secondary_label,
        section_ink: ink.secondary_label,
        count_ink: ink.secondary_label,
        line: ink.separator,
        ..palette
    }
}

pub(crate) fn over_backdrop(surface: SidebarSurface) -> SidebarSurface {
    if crate::backdrop::tinted().is_none() {
        return surface;
    }
    SidebarSurface {
        bg: Rgba::TRANSPARENT,
        palette: sidebar_palette(),
        glass: None,
    }
}

#[derive(Clone, PartialEq)]
struct InSidebar;

// AppKit's sidebar is not first responder until it is clicked or focused, so its selection starts gray.
#[derive(Clone, Copy)]
struct Engaged(State<bool>);

fn unemphasized(s: SidebarItemStyle) -> SidebarItemStyle {
    SidebarItemStyle {
        selected_background: StateValue::new(ShapeStyle::Color(item_palette(false).selected_bg)),
        selected_label: StateValue::new(ShapeStyle::Color(item_palette(false).selected_ink)),
        ..s
    }
}

// Over glass the selection is a veil the glass shows through, as in SwiftUI; elsewhere it is the opaque token.
fn glass_selection() -> Option<GlassSelection> {
    crate::glass_panel::sidebar_glass().map(|_| match theme::resolved().scheme {
        ColorScheme::Light => SIDEBAR_GLASS_SELECTION_LIGHT,
        ColorScheme::Dark | ColorScheme::DarkTint => SIDEBAR_GLASS_SELECTION_DARK,
    })
}

pub(crate) fn item_palette(emphasized: bool) -> SidebarPalette {
    let palette = sidebar_palette();
    let veil = glass_selection();
    if !crate::platform::window_activated() {
        SidebarPalette {
            selected_bg: veil.map_or(palette.inactive_selected_bg, |veil| veil.inactive),
            selected_ink: palette.inactive_selected_ink,
            item_ink: palette.inactive_ink,
            item_icon: palette.inactive_ink,
            ..palette
        }
    } else if !emphasized {
        SidebarPalette {
            selected_bg: veil.map_or(palette.inactive_selected_bg, |veil| veil.unfocused),
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
    pub(crate) content: State<f32>,
    pub(crate) settled: State<bool>,
    pub(crate) overlaid: State<bool>,
}

impl SidebarControl {
    pub fn occupied(&self) -> f32 {
        *self.occupied.read()
    }

    pub fn content(&self) -> f32 {
        *self.content.read()
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
    // In an inactive window only the accent turns grey; a tile of its own colour keeps the light ink, as in macOS.
    pub fn new(glyph: Symbol, tint: Rgba) -> Self {
        let ink = theme::resolved().on_accent;
        Self::styled(
            glyph,
            Theme::tengen().style(move |s| SymbolTileStyle {
                background: StateValue::new(ShapeStyle::Color(tint)),
                symbol: StateValue::new(ShapeStyle::Color(ink)),
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
        let paints = Paints::current(ControlState::default());
        let resolved = theme::resolved();
        let (plate, ink) = ctappearance::icon_plate::tile(
            resolved.icon_style,
            resolved.icon_dark,
            resolved.icon_tint,
            paints.color(style.background.resolve(ControlState::default())),
        );
        let corner_radius =
            style
                .shape
                .corner_radius(style.size, style.size, LayoutDirection::LeftToRight);
        let layer = || {
            rect()
                .position(Position::new_absolute().top(0.0).left(0.0))
                .width(Size::px(style.size))
                .height(Size::px(style.size))
                .corner_radius(corner_radius)
        };
        let tile = rect()
            .width(Size::px(style.size))
            .height(Size::px(style.size))
            .center()
            .maybe_child(
                crate::app_icon::tile_plate(&plate, style.size, corner_radius.top_left()).map(
                    |handle| {
                        // Freya paints by depth, not sibling order: at the tile's own depth the plate lies under the symbol.
                        freya_core::elements::image::image(handle)
                            .position(Position::new_absolute().top(0.0).left(0.0))
                            .layer(-1)
                            .width(Size::px(style.size))
                            .height(Size::px(style.size))
                    },
                ),
            )
            .maybe_child(
                style
                    .border
                    .as_ref()
                    .map(|border| layer().bordered(&paints, Some(border))),
            )
            .child(tile_symbol(
                glyph,
                style.symbol_size,
                ink.unwrap_or_else(|| paints.color(style.symbol.resolve(ControlState::default()))),
            ));
        Self(tile, glyph, theme)
    }
}

forward_container!(SymbolTile);

#[derive(PartialEq)]
pub struct Sidebar {
    children: Vec<Element>,
    traffic_lights: bool,
    style: Theme,
    probe: Option<&'static str>,
    key: DiffKey,
    overrides: Overrides,
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
            overrides: Overrides::default(),
        }
    }

    pub fn traffic_lights(mut self, show: bool) -> Self {
        self.traffic_lights = show;
        self
    }

    pub fn style(mut self, restyle: impl Fn(SidebarStyle) -> SidebarStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl ChildrenExt for Sidebar {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

forward_key!(Sidebar);

forward_overrides!(Sidebar);

impl Component for Sidebar {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: SidebarStyle = resolve(&self.style);
        let glass = crate::glass_panel::sidebar_glass();
        let style = if glass.is_some() {
            SidebarStyle {
                background: StateValue::new(ShapeStyle::Clear),
                ..style
            }
        } else {
            style
        };
        let lights: TrafficLightsStyle = resolve(&Theme::tengen());
        let paints = Paints::current(ControlState::default());
        let (traffic_x, traffic_y) = style.lights_position;

        let controls = try_consume_root_context::<WindowControls>();
        let draws_lights = controls
            .as_ref()
            .is_none_or(|controls| controls.draws_traffic_lights());
        let lights_opacity = crate::chrome::lights_opacity();
        let lights_shown = controls
            .as_ref()
            .is_none_or(|controls| controls.traffic_lights_shown())
            && lights_opacity > 0.0;
        let overlay_top = controls
            .as_ref()
            .and_then(|controls| controls.titlebar_overlay_top());
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
                let revealed_top = lights_top + *controls.focus_bar_height.read();
                resting_top + (revealed_top - resting_top) * progress
            }
        };
        let padding_x = style.padding_x;

        let (status, fixed) = controls
            .as_ref()
            .map(|controls| (*controls.status.read(), controls.fixed))
            .unwrap_or_default();
        let drag = use_window_drag(fixed);

        let focus = use_focus_state::<InSidebar>();
        use_provide_context(move || focus);
        let engaged = use_state(|| false);
        use_provide_context(move || Engaged(engaged));
        let mut panel_area = use_state(Area::default);
        let native_pane = crate::glass_panel::native_pane_glass();
        crate::glass_panel::use_publish(glass, 0.0, *panel_area.read(), native_pane);
        let span = controls.as_ref().map(|controls| controls.sidebar_span);
        if let Some(mut span) = span {
            let area = *panel_area.read();
            span.set_if_modified((area.min_x(), area.width()));
        }
        use_drop(move || {
            if let Some(mut span) = span {
                span.set((0.0, 0.0));
            }
        });
        let sidebar_body = rect()
            .expanded()
            .direction(Direction::Vertical)
            .painted(&paints, &style.background)
            .a11y_role(AccessibilityRole::Navigation)
            .on_sized(move |event: Event<SizedEventData>| panel_area.set_if_modified(event.area))
            .padding(Gaps::new(top_height, padding_x, 0.0, padding_x))
            .maybe_child(glass.filter(|_| !native_pane).map(|_| {
                let size = panel_area.read().size;
                crate::glass_panel::hole((-padding_x, -top_height), (size.width, size.height), 0.0)
            }))
            .child(
                rect()
                    .position(Position::new_absolute().top(-top_height).left(-padding_x))
                    .width(Size::px(SIDEBAR.width as f32))
                    .height(Size::px(top_height))
                    .probe("sidebar-drag-strip")
                    .on_mouse_down(drag),
            )
            .child(Focused::new(
                focus,
                InSidebar,
                ScrollView::new().children(self.children.iter().cloned()),
            ));

        let sidebar_shown = sidebar_control().is_none_or(|control| control.settled());
        let floating_lights = self.traffic_lights && lights_shown && draws_lights && sidebar_shown;
        let root = rect()
            .expanded()
            .child(sidebar_body)
            .maybe_child(native_pane.then(crate::platform::backdrop::Declared::pane))
            .maybe_child(floating_lights.then(|| {
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
                    .child(TrafficLights::for_window(status.activated, fixed))
            }));

        self.overrides.apply(root.map(self.probe, ProbeExt::probe))
    }
}

#[derive(PartialEq)]
pub struct SidebarItem {
    label: Cow<'static, str>,
    symbol: Option<(Symbol, Option<Rgba>)>,
    selected: bool,
    style: Theme,
    on_select: Option<Callback<(), ()>>,
    probe: Option<&'static str>,
    key: DiffKey,
    overrides: Overrides,
}

impl ProbeExt for SidebarItem {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl SidebarItem {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            symbol: None,
            selected: false,
            style: Theme::tengen(),
            on_select: None,
            probe: None,
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn symbol(mut self, symbol: Symbol) -> Self {
        self.symbol = Some((symbol, None));
        self
    }

    pub fn symbol_tile(mut self, symbol: Symbol, tint: Rgba) -> Self {
        self.symbol = Some((symbol, Some(tint)));
        self
    }

    /// SwiftUI's `.tag` in a `List(selection:)`: selected while `selection` holds `value`; a click stores it.
    pub fn tag<T: Clone + PartialEq + 'static>(mut self, selection: State<T>, value: T) -> Self {
        self.selected = *selection.read() == value;
        let mut selection = selection;
        self.on_select = Some(Callback::new(move |_| selection.set(value.clone())));
        self
    }

    pub fn emphasized(self, emphasized: bool) -> Self {
        if emphasized {
            self
        } else {
            self.style(unemphasized)
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SidebarItemStyle) -> SidebarItemStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

forward_key!(SidebarItem);

forward_overrides!(SidebarItem);

impl Component for SidebarItem {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: SidebarItemStyle = resolve(&self.style);
        let engaged = try_consume_context::<Engaged>();
        let focus_elsewhere =
            try_consume_context::<FocusState<InSidebar>>().is_some_and(|inside| {
                inside.get().is_none()
                    && (!engaged.is_some_and(|Engaged(engaged)| *engaged.read())
                        || !ACCESSIBILITY_ROOT_ID.is_focused())
            });
        let style = if focus_elsewhere {
            unemphasized(style)
        } else {
            style
        };
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
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

        let row = interaction
            .attach(rect(), true)
            .width(Size::fill())
            .height(Size::px(style.height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.spacing)
            .padding(Gaps::new(
                0.0,
                style.padding_x,
                0.0,
                if self.symbol.is_some() {
                    style.padding_x
                } else {
                    style.text_padding_x
                },
            ))
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
                    if let Some(Engaged(mut engaged)) = engaged {
                        engaged.set_if_modified(true);
                    }
                    callback.call(());
                    if let Some(control) = control {
                        control.dismiss();
                    }
                })
            })
            .maybe_child(self.symbol.map(|(glyph, tint)| -> Element {
                match tint {
                    Some(tint) => {
                        let border = style.selected_tile_border.clone().filter(|_| self.selected);
                        SymbolTile::new(glyph, tint)
                            .map(border, |tile, border| {
                                tile.style(move |s| SymbolTileStyle {
                                    border: Some(border.clone()),
                                    ..s
                                })
                            })
                            .into_element()
                    }
                    None => {
                        let ink = if self.selected {
                            &style.selected_symbol
                        } else {
                            &style.symbol
                        };
                        rect()
                            .width(Size::px(SIDEBAR.app_icon as f32))
                            .height(Size::px(SIDEBAR.app_icon as f32))
                            .center()
                            .child(stroked_symbol(
                                glyph,
                                style.symbol_size,
                                style.symbol_stroke,
                                paints.color(ink.resolve(state)),
                            ))
                            .into_element()
                    }
                }
            }))
            .child(
                label()
                    .text(self.label.clone())
                    .with(|el| text_style(el, font))
                    .color(theme::color(paints.color(ink.resolve(state))))
                    .max_lines(1),
            )
            .map(self.probe, ProbeExt::probe);
        self.overrides
            .apply(interaction.focus_ring(row, corner_radius.top_left()))
    }
}

#[derive(PartialEq)]
pub struct SidebarSection {
    title: Cow<'static, str>,
    children: Vec<Element>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SidebarSection {
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SidebarSectionStyle) -> SidebarSectionStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl ChildrenExt for SidebarSection {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

forward_key!(SidebarSection);

forward_overrides!(SidebarSection);

impl Component for SidebarSection {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let expanded = use_state(|| true);
        let style: SidebarSectionStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
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

        let root = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .margin(Gaps::new(0.0, 0.0, style.margin_bottom, 0.0))
            .child(
                interaction
                    .attach(rect(), true)
                    .width(Size::fill())
                    .height(Size::px(style.height))
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
                            .with(|el| text_style(el, style.font))
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
            );
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct SidebarAccount {
    initials: Cow<'static, str>,
    name: Cow<'static, str>,
    detail: Cow<'static, str>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SidebarAccount {
    pub fn new(
        initials: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
        detail: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            initials: initials.into(),
            name: name.into(),
            detail: detail.into(),
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
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

forward_overrides!(SidebarAccount);

forward_key!(SidebarAccount);

impl Component for SidebarAccount {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: SidebarAccountStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default());
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let root = rect()
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
            );
        self.overrides.apply(root)
    }
}
