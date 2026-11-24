use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::traffic::TrafficLights;
use crate::views::split::{InspectorSpan, SidebarToggle};
use crate::views::toolbar::{ToolbarContent, lay_out, row};
use ctappearance::window::{
    TitlebarAction, WindowBehavior, double_click_action, green_action, read_window_behavior,
};
use ctdesign::{
    ButtonKind, TITLEBAR_DOUBLE_CLICK, TITLEBAR_EXPANDED_TITLE_GAP, TITLEBAR_EXPANDED_TITLE_HEIGHT,
    TITLEBAR_EXPANDED_TOOLBAR_BOTTOM, TITLEBAR_FULLSCREEN_START, TITLEBAR_TOOLBAR_START,
};
use ctdesign::{SIDEBAR, TOOLGROUP};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::app::DefaultsRevision;
use crate::platform::{Bar, DecorationResponse, WindowControls, WindowStatus};
use crate::style::{text_style, with_font};
use crate::styles::{
    ControlState, Paints, ShapeStyle, SidebarStyle, StateValue, Theme, TitlebarStyle,
    TrafficLightsStyle, resolve,
};
use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowToolbarStyle {
    Unified,
    UnifiedCompact,
    Expanded,
}

#[derive(PartialEq)]
pub struct Titlebar {
    preview: Option<Preview>,
    title: Option<Cow<'static, str>>,
    shows_title: bool,
    sidebar: bool,
    toolbar_style: Option<WindowToolbarStyle>,
    left: Option<Element>,
    toolbar: Vec<ToolbarContent>,
    probe: Option<&'static str>,
    close_only: bool,
    style: Theme,
}

#[derive(Clone, PartialEq)]
struct Preview {
    status: WindowStatus,
    title: Cow<'static, str>,
    fixed: bool,
}

impl Default for Titlebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Titlebar {
    pub fn new() -> Self {
        Self {
            preview: None,
            title: None,
            shows_title: true,
            sidebar: false,
            toolbar_style: None,
            left: None,
            toolbar: Vec::new(),
            probe: None,
            close_only: false,
            style: Theme::tengen(),
        }
    }

    pub fn preview(status: WindowStatus, title: impl Into<Cow<'static, str>>, fixed: bool) -> Self {
        Self {
            preview: Some(Preview {
                status,
                title: title.into(),
                fixed,
            }),
            ..Self::new()
        }
    }

    pub fn title(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// SwiftUI's `showsTitle:`: without the title, the trailing items follow the traffic lights.
    pub fn shows_title(mut self, shows_title: bool) -> Self {
        self.shows_title = shows_title;
        self
    }

    pub fn with_sidebar(mut self, sidebar: bool) -> Self {
        self.sidebar = sidebar;
        self
    }

    pub fn left(mut self, child: impl IntoElement) -> Self {
        self.left = Some(child.into_element());
        self
    }

    pub fn toolbar(mut self, content: impl Into<ToolbarContent>) -> Self {
        self.toolbar.push(content.into());
        self
    }

    pub fn spacing(self, spacing: f32) -> Self {
        self.style(move |s| TitlebarStyle {
            spacing,
            sidebar_spacing: spacing,
            ..s
        })
    }

    pub fn toolbar_style(mut self, style: WindowToolbarStyle) -> Self {
        self.toolbar_style = Some(style);
        self
    }

    pub fn close_only(mut self, close_only: bool) -> Self {
        self.close_only = close_only;
        self
    }

    pub fn hairline(self, hairline: bool) -> Self {
        self.style(move |s| TitlebarStyle {
            hairline: hairline.then(TitlebarStyle::line),
            ..s
        })
    }

    pub fn style(mut self, restyle: impl Fn(TitlebarStyle) -> TitlebarStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl ProbeExt for Titlebar {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Titlebar {
    fn render(&self) -> impl IntoElement {
        let controls =
            try_consume_root_context::<WindowControls>().filter(|_| self.preview.is_none());
        let close_only = self.close_only
            || controls
                .as_ref()
                .is_some_and(|controls| controls.hud || controls.panel);
        let panel = controls.as_ref().is_some_and(|controls| controls.panel);
        let toolbar_style = self.toolbar_style.unwrap_or(
            if self.sidebar || self.left.is_some() || !self.toolbar.is_empty() {
                WindowToolbarStyle::Unified
            } else {
                WindowToolbarStyle::UnifiedCompact
            },
        );
        let unified = toolbar_style == WindowToolbarStyle::Unified;
        let expanded = toolbar_style == WindowToolbarStyle::Expanded;
        let bar = if panel {
            Bar::Utility
        } else if close_only || expanded {
            Bar::Titlebar
        } else if unified {
            Bar::Unified
        } else {
            Bar::UnifiedCompact
        };
        crate::compact::use_toolbar_items((!panel && !close_only).then_some(toolbar_style));
        let bar_controls = controls.clone();
        use_side_effect_with_deps(&bar, move |bar| {
            if let Some(controls) = &bar_controls {
                controls.set_bar(*bar);
            }
        });
        let content_widths = use_state(|| (0.0f32, 0.0f32));
        let (status, title, fixed) = match (&self.preview, &controls) {
            (Some(preview), _) => (preview.status, preview.title.clone(), preview.fixed),
            (None, Some(controls)) => (
                *controls.status.read(),
                controls.title.clone(),
                controls.fixed,
            ),
            (None, None) => (WindowStatus::default(), Cow::default(), false),
        };
        let title = match self.shows_title {
            true => self.title.clone().unwrap_or(title),
            false => Cow::default(),
        };
        let drag = use_window_drag(fixed);
        // With `OnHover` the pinned bar gives way to the content; it shows only while AppKit's bar is down, where its row rides.
        let on_hover = controls
            .as_ref()
            .is_some_and(WindowControls::toolbar_slides_over);
        let bar_down = controls
            .as_ref()
            .is_some_and(|controls| *controls.focus_bar.read());
        if status.decoration == DecorationResponse::Server && !self.toolbar.is_empty() {
            static WARNED: std::sync::Once = std::sync::Once::new();
            WARNED.call_once(|| {
                tracing::warn!(
                    "BarrierKit: the system draws this window's title bar, so the Titlebar's toolbar items do not show; on macOS set WindowStyle::HiddenTitleBar"
                )
            });
        }
        if status.decoration == DecorationResponse::Server || (on_hover && !bar_down) {
            return rect()
                .width(Size::fill())
                .height(Size::px(0.0))
                .into_element();
        }

        let overlaid = controls
            .as_ref()
            .is_some_and(WindowControls::titlebar_overlaid);
        let style: TitlebarStyle = resolve(&self.style);
        let control = crate::sidebar::sidebar_control();
        let sidebar = self.sidebar && control.is_none_or(|control| control.settled());
        // Beside a sidebar, over content that does not scroll under it, AppKit's bar draws its separator, as SwiftUI's.
        let style = TitlebarStyle {
            hairline: style.hairline.clone().or_else(|| {
                (sidebar
                    && !status.fullscreen
                    && crate::theme::window_material_here()
                    && !scroll_under(
                        style.sidebar_height,
                        control.map_or(0.0, |control| control.occupied()),
                    ))
                .then(TitlebarStyle::line)
            }),
            ..style
        };
        let state = ControlState {
            focused: status.activated,
            ..ControlState::default()
        };
        let paints = Paints::current(state);
        let background = if overlaid || on_hover {
            paints.fill(&StateValue::new(ShapeStyle::Clear))
        } else if close_only {
            paints.fill(&style.close_only_background)
        } else if sidebar && !status.fullscreen {
            paints.fill(&style.sidebar_background)
        } else {
            paints.fill(&style.background)
        };

        let title_ink = if expanded {
            &style.row_title
        } else {
            &style.title
        };
        let title_color = paints.color(title_ink.resolve(state));
        let lights_shown = controls
            .as_ref()
            .is_none_or(WindowControls::traffic_lights_shown);
        let draws_lights = controls
            .as_ref()
            .is_none_or(WindowControls::draws_traffic_lights);
        let lights_opacity = crate::chrome::lights_opacity();
        let traffic = if sidebar || !lights_shown || !draws_lights || lights_opacity == 0.0 {
            None
        } else {
            Some(
                rect()
                    .opacity(lights_opacity)
                    .child(
                        TrafficLights::for_window(status.activated, fixed).close_only(close_only),
                    )
                    .into_element(),
            )
        };

        let slots = self.toolbar_slots(&style);
        let title_font = if unified {
            style.sidebar_title_font
        } else {
            style.title_font
        };
        let title_label = if unified {
            with_font(label(), title_font)
        } else {
            text_style(label(), title_font)
        }
        .text(title)
        .color(theme::color(title_color))
        .max_lines(1)
        .text_overflow(TextOverflow::Ellipsis)
        .map(self.probe, |label, id| {
            label.a11y_builder(move |node| node.set_author_id(id))
        });
        let title_label = if status.fullscreen && status.decoration == DecorationResponse::Hybrid {
            rect()
                .child(crate::platform::backdrop::Declared::layer())
                .child(title_label)
                .into_element()
        } else {
            title_label.into_element()
        };

        let shield = crate::views::feedback::OpenSheets::any().then(|| drag.clone());
        let native_lights = status.decoration == DecorationResponse::Hybrid
            && (!status.fullscreen
                || controls
                    .as_ref()
                    .is_some_and(|controls| *controls.focus_bar.read()));
        let system_bar = |height: f32| {
            !close_only
                && !panel
                && !overlaid
                && !sidebar
                && !status.fullscreen
                && *style.background.resolve(state) == ShapeStyle::WindowBackground
                && crate::glass::native_glass()
                && !scroll_under(height, 0.0)
        };
        // On AppKit's own bar, AppKit draws the expanded title row, as SwiftUI's.
        let system_title = system_bar(TITLEBAR_EXPANDED_TITLE_HEIGHT + style.height);
        let (bar, height) = if expanded && !close_only {
            self.expanded_bar(
                &style,
                traffic,
                native_lights,
                slots,
                (!status.fullscreen && !system_title).then_some(title_label),
                status.fullscreen,
                system_title,
                drag,
                background,
            )
        } else if unified {
            let lights = match traffic {
                Some(traffic) => Lights::Drawn(traffic),
                None if native_lights && !sidebar => Lights::Native,
                None => Lights::Absent,
            };
            self.sidebar_bar(
                &style,
                &paints,
                lights,
                slots,
                title_label,
                status.fullscreen,
                drag,
                background,
            )
        } else {
            let native_gap = native_lights.then(|| {
                let lights: TrafficLightsStyle = resolve(&Theme::tengen());
                rect().width(Size::px(lights.length())).into_element()
            });
            self.centered_bar(
                &style,
                traffic.or(native_gap),
                slots,
                title_label,
                status.fullscreen,
                drag,
                background,
                close_only,
                content_widths,
            )
        };
        let bar = match shield {
            Some(drag) => rect()
                .width(Size::fill())
                .child(bar)
                .child(
                    rect()
                        .position(Position::new_absolute().top(0.0).left(0.0))
                        .width(Size::fill())
                        .height(Size::px(height))
                        .layer(crate::views::feedback::SHEET_TITLEBAR)
                        .on_mouse_down(drag),
                )
                .into_element(),
            None => bar,
        };
        // Over content that does not scroll under it, AppKit draws its own title bar beneath the row, as under SwiftUI's toolbar.
        let bar = if system_bar(height) && !expanded {
            rect()
                .width(Size::fill())
                .child(crate::platform::backdrop::Declared::titlebar(height))
                .child(bar)
                .into_element()
        } else {
            bar
        };
        let bar = if on_hover {
            rect()
                .width(Size::fill())
                .height(Size::px(0.0))
                .child(
                    rect()
                        .position(Position::new_absolute().top(0.0).left(0.0))
                        .width(Size::fill())
                        .child(bar),
                )
                .into_element()
        } else {
            bar
        };
        let inspector = try_consume_context::<InspectorSpan>().map_or(0.0, |span| *span.0.read());
        let bar = if inspector > 0.0 {
            rect()
                .width(Size::fill())
                .margin(Gaps::new(0.0, -inspector, 0.0, 0.0))
                .layer(TITLEBAR_OVERLAY_LAYER)
                .child(bar)
                .into_element()
        } else {
            bar
        };
        if let Some(controls) = &controls {
            let mut taken = controls.titlebar_height;
            taken.set_if_modified(if overlaid || on_hover { 0.0 } else { height });
        }
        place_overlaid(
            crate::chrome::revealable_bar(bar, height),
            height,
            controls.as_ref(),
            overlaid,
        )
    }
}

type ScrollKey = State<(f32, f32)>;

#[derive(Clone, Copy)]
pub(crate) struct ScrollFrames(pub(crate) State<Vec<(ScrollKey, torin::prelude::Area)>>);

impl ScrollFrames {
    pub(crate) fn report(mut self, key: ScrollKey, area: torin::prelude::Area) {
        let known = self
            .0
            .peek()
            .iter()
            .find(|(other, _)| *other == key)
            .map(|(_, area)| *area);
        if known != Some(area) {
            let mut frames = self.0.write();
            frames.retain(|(other, _)| *other != key);
            frames.push((key, area));
        }
    }

    pub(crate) fn forget(mut self, key: ScrollKey) {
        if let Some(mut frames) = self.0.try_write() {
            frames.retain(|(other, _)| *other != key);
        }
    }
}

// Whether a scroll view right of `left` reaches under the bar.
fn scroll_under(bar_height: f32, left: f32) -> bool {
    try_consume_root_context::<ScrollFrames>().is_some_and(|frames| {
        frames.0.read().iter().any(|(_, area)| {
            area.min_x() >= left && area.min_y() <= bar_height + 1.0 && area.max_y() > bar_height
        })
    })
}

struct ToolbarSlots {
    navigation: Option<Element>,
    principal: Option<Element>,
    trailing: Option<Element>,
}

enum Lights {
    Drawn(Element),
    Native,
    Absent,
}

impl Titlebar {
    fn toolbar_slots(&self, style: &TitlebarStyle) -> ToolbarSlots {
        let gap = style.sidebar_padding_x;
        let sections = lay_out(&self.toolbar, gap);
        if self.shows_title {
            ToolbarSlots {
                navigation: row(sections.navigation, gap),
                principal: row(sections.center, gap),
                trailing: row(sections.trailing, gap),
            }
        } else {
            ToolbarSlots {
                navigation: row(
                    sections
                        .navigation
                        .into_iter()
                        .chain(sections.trailing)
                        .collect(),
                    gap,
                ),
                principal: row(sections.center, gap),
                trailing: None,
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sidebar_bar(
        &self,
        style: &TitlebarStyle,
        paints: &Paints,
        traffic: Lights,
        slots: ToolbarSlots,
        title_label: Element,
        fullscreen: bool,
        drag: impl Fn(Event<MouseEventData>) + 'static,
        background: Fill,
    ) -> (Element, f32) {
        let occupied = crate::sidebar::sidebar_control()
            .map_or(0.0, |control| control.occupied() + control.content());
        let height = style.sidebar_height;
        let padding_x = style.sidebar_padding_x;
        let sidebar: SidebarStyle = resolve(&Theme::tengen());
        let lights: TrafficLightsStyle = resolve(&Theme::tengen());
        let (traffic_x, traffic_y) = sidebar.lights_position;
        let lights_end = traffic_x + lights.length();
        let lights_space = !matches!(traffic, Lights::Absent);
        let item_first = self.left.is_some()
            || crate::sidebar::sidebar_control().is_some()
            || slots.navigation.is_some();
        let lights_beside_title = lights_space && self.left.is_none() && slots.navigation.is_none();
        let lights_gap = if item_first {
            TITLEBAR_TOOLBAR_START.unified
        } else {
            padding_x
        };
        let padding_left = if lights_space {
            padding_x.max(lights_end + lights_gap - occupied)
        } else if fullscreen && occupied == 0.0 {
            // The bar's spacing also follows the empty left slot, so the title lands at the start and an item 13 before it, as SwiftUI's.
            TITLEBAR_FULLSCREEN_START.unified - style.sidebar_spacing
        } else {
            padding_x
        };
        // The lights stay at the sidebar's window position while it folds (fold in place).
        let fixed_lights = match traffic {
            Lights::Drawn(traffic) => Some(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(traffic_x - occupied - padding_left)
                            .top(traffic_y),
                    )
                    .layer(TITLEBAR_OVERLAY_LAYER + 1)
                    .child(traffic),
            ),
            Lights::Native | Lights::Absent => None,
        };

        let left_slot = rect()
            .width(Size::auto())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .spacing(padding_x)
            .probe("titlebar-left-slot")
            .maybe_child(self.left.clone().or_else(|| {
                crate::sidebar::sidebar_control().map(|_| SidebarToggle.into_element())
            }))
            .maybe_child(slots.navigation);

        let principal = slots.principal.map(|principal| {
            // AppKit centers the principal item on the window, not on the room the lights leave.
            rect()
                .position(
                    Position::new_absolute()
                        .left(-occupied - padding_left)
                        .top(0.0),
                )
                .width(Size::window_percent(100.0))
                .height(Size::px(height))
                .center()
                .child(principal)
        });
        let title = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .main_align(style.sidebar_title_alignment.clone())
            .probe("titlebar-title")
            .on_mouse_down(drag)
            .maybe_child(principal.is_none().then_some(title_label));

        let root = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .background(background)
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(if lights_beside_title {
                style.lights_spacing
            } else {
                style.sidebar_spacing
            })
            .padding(Gaps::new(0.0, padding_x, 0.0, padding_left))
            .child(left_slot)
            .child(title)
            .maybe_child(slots.trailing.map(|right| {
                rect()
                    .width(Size::auto())
                    .height(Size::fill())
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .probe("titlebar-right-slot")
                    .child(right)
            }))
            .maybe_child(fixed_lights)
            .maybe_child(principal);
        let bar = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .child(root)
            .maybe_child(style.hairline.as_ref().map(|(paint, thickness)| {
                rect()
                    .position(Position::new_absolute().top(height).left(0.0))
                    .width(Size::percent(100.0))
                    .height(Size::px(*thickness))
                    .background(paints.fill(&StateValue::new(paint.clone())))
            }));
        (bar.into_element(), height)
    }

    #[allow(clippy::too_many_arguments)]
    fn expanded_bar(
        &self,
        style: &TitlebarStyle,
        traffic: Option<Element>,
        native_lights: bool,
        slots: ToolbarSlots,
        title_label: Option<Element>,
        fullscreen: bool,
        system_bar: bool,
        drag: impl Fn(Event<MouseEventData>) + Clone + 'static,
        background: Fill,
    ) -> (Element, f32) {
        let lights: TrafficLightsStyle = resolve(&Theme::tengen());
        let (lights_x, lights_y, diameter) = ctappearance::traffic::button_rect(0);
        let title_height = if fullscreen {
            fullscreen_title_height()
        } else {
            TITLEBAR_EXPANDED_TITLE_HEIGHT
        };
        let lights_center = (lights_y + diameter / 2.0) as f32;
        let title_row = rect()
            .width(Size::fill())
            .height(Size::px(title_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(
                2.0 * lights_center - title_height,
                style.padding_x,
                0.0,
                lights_x as f32,
            ))
            .spacing(TITLEBAR_EXPANDED_TITLE_GAP)
            .probe("titlebar-title")
            .on_mouse_down(drag.clone())
            .maybe_child(traffic)
            .maybe_child(native_lights.then(|| {
                rect()
                    .width(Size::px(lights.length()))
                    .height(Size::px(diameter as f32))
            }))
            .maybe_child(title_label);
        let toolbar_row = rect()
            .width(Size::fill())
            .height(Size::px(style.height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(
                0.0,
                style.sidebar_padding_x,
                TITLEBAR_EXPANDED_TOOLBAR_BOTTOM,
                TITLEBAR_TOOLBAR_START.expanded,
            ))
            .spacing(style.sidebar_spacing)
            .probe("titlebar-left-slot")
            // Only the toolbar row lies on AppKit's bar, so it rides over it while AppKit draws the title row.
            .maybe_child(
                system_bar.then(|| crate::platform::backdrop::Declared::titlebar(style.height)),
            )
            .maybe_child(self.left.clone())
            .maybe_child(slots.navigation)
            .maybe_child(slots.trailing)
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Center)
                    .on_mouse_down(drag)
                    .maybe_child(slots.principal),
            );
        let height = title_height + style.height;
        let bar = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .background(background)
            .content(Content::flex())
            .child(title_row)
            .child(toolbar_row)
            // Native lights in full screen mean AppKit's bar is down; its row rides there and SwiftUI drops this line.
            .maybe_child((fullscreen && !native_lights).then(|| {
                let (paint, thickness) = TitlebarStyle::line();
                rect()
                    .position(Position::new_absolute().top(height).left(0.0))
                    .width(Size::percent(100.0))
                    .height(Size::px(thickness))
                    .background(
                        Paints::current(ControlState::default()).fill(&StateValue::new(paint)),
                    )
            }));
        (bar.into_element(), height)
    }

    #[allow(clippy::too_many_arguments)]
    fn centered_bar(
        &self,
        style: &TitlebarStyle,
        traffic: Option<Element>,
        slots: ToolbarSlots,
        title_label: Element,
        fullscreen: bool,
        drag: impl Fn(Event<MouseEventData>) + Clone + 'static,
        background: Fill,
        close_only: bool,
        mut content_widths: State<(f32, f32)>,
    ) -> (Element, f32) {
        let height = if close_only {
            style.close_only_height
        } else {
            style.height
        };
        let lights: TrafficLightsStyle = resolve(&Theme::tengen());
        let padding_x = if traffic.is_some() {
            ctappearance::traffic::inset(height as u32) as f32
        } else if fullscreen {
            TITLEBAR_FULLSCREEN_START.compact
        } else {
            style.padding_x
        };
        // A compact bar starts its title where an item would start, as macOS 26 does; a utility panel keeps it centered.
        let leading = !close_only;
        let items_first = self.left.is_some() || slots.navigation.is_some();
        let lights_gap = if leading || items_first {
            TITLEBAR_TOOLBAR_START.compact
        } else {
            0.0
        };
        let (left_width, right_width) = *content_widths.read();
        let slot_width = match leading {
            true => Size::auto(),
            false => Size::px(lights.length().max(left_width).max(right_width)),
        };
        let left_slot = rect()
            .width(slot_width.clone())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .probe("titlebar-left-slot")
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(style.spacing)
                    .on_sized(move |event: Event<SizedEventData>| {
                        let width = event.area.size.width;
                        let right = content_widths.peek().1;
                        content_widths.set_if_modified((width, right));
                    })
                    .maybe_child(traffic.map(|lights| {
                        rect()
                            .padding(Gaps::new(0.0, lights_gap, 0.0, 0.0))
                            .child(lights)
                    }))
                    .maybe_child(self.left.clone())
                    .maybe_child(slots.navigation),
            );

        let right_slot = rect()
            .width(slot_width)
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .main_align(Alignment::End)
            .cross_align(Alignment::Center)
            .probe("titlebar-right-slot")
            .child(
                rect()
                    .on_sized(move |event: Event<SizedEventData>| {
                        let width = event.area.size.width;
                        let left = content_widths.peek().0;
                        content_widths.set_if_modified((left, width));
                    })
                    .maybe_child(slots.trailing),
            )
            .on_mouse_down(drag.clone());

        let title = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .padding(Gaps::new(
                0.0,
                0.0,
                0.0,
                if leading && items_first {
                    style.sidebar_spacing
                } else {
                    0.0
                },
            ))
            .cross_align(if leading {
                Alignment::Start
            } else {
                style.title_alignment.clone()
            })
            .main_align(Alignment::Center)
            .probe("titlebar-title")
            .on_mouse_down(drag)
            .child(slots.principal.unwrap_or(title_label));

        let root = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .background(background)
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .padding(Gaps::new(0.0, padding_x, 0.0, padding_x))
            .child(left_slot)
            .child(title)
            .child(right_slot);
        (root.into_element(), height)
    }
}

fn place_overlaid(
    bar: Element,
    height: f32,
    controls: Option<&WindowControls>,
    overlaid: bool,
) -> Element {
    let Some(controls) = controls.filter(|_| overlaid) else {
        return bar;
    };
    let Some(top) = controls.titlebar_overlay_top() else {
        return rect()
            .width(Size::fill())
            .height(Size::px(0.0))
            .into_element();
    };
    let opacity = *controls.focus_bar_progress.read();
    let global = Position::new_global().top(top).left(0.0);
    let bar = rect()
        .position(Position::new_absolute().top(top).left(0.0))
        .width(Size::fill())
        .layer(TITLEBAR_OVERLAY_LAYER)
        .opacity(opacity)
        .child(bar);
    // The band spans the whole window, whatever column holds the titlebar.
    rect()
        .width(Size::fill())
        .height(Size::px(0.0))
        .child(
            rect()
                .position(global)
                .width(Size::window_percent(100.0))
                .layer(TITLEBAR_OVERLAY_LAYER)
                .child(crate::glass::fullscreen_backdrop(height, opacity)),
        )
        .child(bar)
        .into_element()
}

pub(crate) const TITLEBAR_OVERLAY_LAYER: i16 = i16::MAX / 32;

pub fn sidebar_bar_height() -> f32 {
    let style: TitlebarStyle = resolve(&Theme::tengen());
    style.sidebar_height
}

// Pinned in full screen, an expanded bar is as tall as a unified one, as SwiftUI's.
pub(crate) fn fullscreen_title_height() -> f32 {
    (ctdesign::SETTINGS.toolbar_height - ctdesign::TITLEBAR_CLIENT_HEIGHT) as f32
}

// The toggle sits at the sidebar's right end while it is docked and after the traffic lights once it is not. Returns how far it moves from where the bar's left zone puts it, and the width it keeps there.
pub(crate) fn sidebar_toggle_placement(occupied: f32, content: f32) -> (f32, f32) {
    let style: TitlebarStyle = resolve(&Theme::tengen());
    let sidebar: SidebarStyle = resolve(&Theme::tengen());
    let lights: TrafficLightsStyle = resolve(&Theme::tengen());
    let platter = TOOLGROUP.height as f32;
    let padding_x = style.sidebar_padding_x;
    let fullscreen = try_consume_root_context::<WindowControls>()
        .is_some_and(|controls| controls.status.read().fullscreen);
    let start = if fullscreen {
        SIDEBAR.toggle_start_fullscreen
    } else {
        TITLEBAR_TOOLBAR_START.unified
    };
    let hidden_x = sidebar.lights_position.0 + lights.length() + start;
    let docked_x = occupied - platter - SIDEBAR.toggle_inset;
    // AppKit docks the toggle at the sidebar's end only once that leaves it `toggle_dock` past its place after the lights.
    let toggle_x = if docked_x >= hidden_x + SIDEBAR.toggle_dock {
        docked_x
    } else {
        hidden_x
    };
    let zone_x = (occupied + content + padding_x).max(hidden_x);
    (
        toggle_x - zone_x,
        (toggle_x + platter - zone_x).clamp(0.0, platter),
    )
}

impl TrafficLights {
    pub fn for_window(focused: bool, fixed: bool) -> Self {
        let fixed = fixed
            || try_consume_root_context::<WindowControls>()
                .is_some_and(|controls| controls.fixed_width);
        let green = green_action(window_behavior().green_button, fixed);
        let fullscreen = try_consume_root_context::<WindowControls>()
            .is_some_and(|controls| controls.status.read().fullscreen);
        TrafficLights::new()
            .focused(focused)
            .fullscreen(fullscreen)
            .zoom_supported(green != TitlebarAction::None)
            .fixed_width(fixed)
            .on_action(move |kind| {
                let controls = WindowControls::get();
                match kind {
                    ButtonKind::Close => controls.close(),
                    ButtonKind::Minimize => controls.minimize(),
                    ButtonKind::Maximize => run_action(&controls, green),
                }
            })
    }
}

pub fn use_window_drag(fixed: bool) -> impl Fn(Event<MouseEventData>) + Clone {
    let last_click = use_hook(|| Rc::new(Cell::new(None::<(Instant, (f64, f64))>)));
    let fixed_cell = use_hook(|| Rc::new(Cell::new(fixed)));
    fixed_cell.set(fixed);
    move |event: Event<MouseEventData>| {
        if event.button != Some(MouseButton::Left) {
            return;
        }
        let controls = WindowControls::get();
        let now = Instant::now();
        let point = (event.element_location.x, event.element_location.y);
        let double = last_click.get().is_some_and(|(then, before)| {
            now.duration_since(then)
                <= Duration::from_millis(u64::from(TITLEBAR_DOUBLE_CLICK.interval_ms))
                && (before.0 - point.0).abs() <= TITLEBAR_DOUBLE_CLICK.slop
                && (before.1 - point.1).abs() <= TITLEBAR_DOUBLE_CLICK.slop
        });
        if double {
            last_click.set(None);
            run_action(
                &controls,
                double_click_action(window_behavior().double_click_titlebar, !fixed_cell.get()),
            );
        } else {
            last_click.set(Some((now, point)));
            controls.move_window();
        }
    }
}

fn window_behavior() -> WindowBehavior {
    thread_local! {
        static LOADED: Cell<Option<(u64, WindowBehavior)>> = const { Cell::new(None) };
    }
    let revision = try_consume_root_context::<DefaultsRevision>()
        .map_or(0, |DefaultsRevision(revision)| *revision.read());
    LOADED.with(|loaded| match loaded.get() {
        Some((seen, behavior)) if seen == revision => behavior,
        _ => {
            let behavior = read_window_behavior();
            loaded.set(Some((revision, behavior)));
            behavior
        }
    })
}

fn run_action(controls: &WindowControls, action: TitlebarAction) {
    match action {
        TitlebarAction::Maximize => controls.toggle_maximize(),
        TitlebarAction::Minimize => controls.minimize(),
        TitlebarAction::Focus => controls.toggle_fullscreen(),
        TitlebarAction::None => {}
    }
}
