use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::traffic::TrafficLights;
use ct_appearance::SidebarStyle;
use ct_appearance::palette;
use ct_appearance::window::{
    TitlebarAction, WindowBehavior, double_click_action, green_action, read_window_behavior,
};
use ct_design::NATIVE_APP_ID_PREFIX;
use ct_design::{
    ButtonKind, SETTINGS, SIDEBAR, TITLE_2, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_DOUBLE_CLICK,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::app::DefaultsRevision;
use crate::platform::{Bar, DecorationResponse, WindowControls, WindowStatus};
use crate::theme;

#[derive(PartialEq)]
pub struct Titlebar {
    preview: Option<Preview>,
    title: Option<String>,
    sidebar: bool,
    left: Option<Element>,
    right: Option<Element>,
    probe: Option<&'static str>,
    spacing: f32,
    close_only: bool,
    hairline: bool,
}

#[derive(Clone, PartialEq)]
struct Preview {
    status: WindowStatus,
    title: String,
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
            sidebar: false,
            left: None,
            right: None,
            probe: None,
            spacing: 0.0,
            close_only: false,
            hairline: true,
        }
    }

    pub fn preview(status: WindowStatus, title: impl Into<String>, fixed: bool) -> Self {
        Self {
            preview: Some(Preview {
                status,
                title: title.into(),
                fixed,
            }),
            ..Self::new()
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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

    pub fn right(mut self, child: impl IntoElement) -> Self {
        self.right = Some(child.into_element());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn close_only(mut self, close_only: bool) -> Self {
        self.close_only = close_only;
        self
    }

    pub fn hairline(mut self, hairline: bool) -> Self {
        self.hairline = hairline;
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
        let bar = if close_only {
            Bar::Titlebar
        } else if self.sidebar && theme::resolved().sidebar.style == SidebarStyle::Island {
            Bar::Unified
        } else {
            Bar::Compact
        };
        crate::compact::use_toolbar_items(bar == Bar::Unified);
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
            (None, None) => (WindowStatus::default(), String::new(), false),
        };
        let title = self.title.clone().unwrap_or(title);
        let drag = window_drag_handler(fixed);
        if status.decoration == DecorationResponse::Server {
            return rect()
                .width(Size::fill())
                .height(Size::px(0.0))
                .into_element();
        }

        let layout = match self.preview {
            Some(_) => crate::chrome::TitlebarLayout::Top,
            None => crate::chrome::chrome_style().layout,
        };
        let overlaid = layout != crate::chrome::TitlebarLayout::Rail
            && controls
                .as_ref()
                .is_some_and(WindowControls::titlebar_overlaid);
        let resolved = crate::theme::resolved();
        let theme = palette(resolved.scheme);
        let control = crate::sidebar::sidebar_control();
        let sidebar = self.sidebar && control.is_none_or(|control| control.settled());
        let tinted = crate::backdrop::tinted().is_some();
        let ink = crate::style::Palette::current();
        let background = if tinted || overlaid || close_only {
            ct_design::Rgba::TRANSPARENT
        } else if sidebar || status.activated {
            crate::theme::surface_background(&resolved, crate::theme::SurfaceKind::Toplevel)
        } else {
            resolved.surfaces.titlebar_inactive
        };

        let title_color = match (tinted, status.activated) {
            (true, true) => ink.label,
            (true, false) => ink.secondary_label,
            (false, true) => theme.title.focused,
            (false, false) => theme.title.unfocused,
        };
        let lights_shown = controls
            .as_ref()
            .is_none_or(WindowControls::traffic_lights_shown);
        let vertical = layout != crate::chrome::TitlebarLayout::Top;
        let lights_opacity = crate::chrome::lights_opacity();
        let traffic = if sidebar || !lights_shown || vertical || lights_opacity == 0.0 {
            None
        } else {
            Some(
                rect()
                    .opacity(lights_opacity)
                    .child(window_traffic_lights(status.activated, fixed).close_only(close_only))
                    .into_element(),
            )
        };

        let title_font = if self.sidebar { TITLE_2 } else { TITLEBAR.font };
        let title_label = label()
            .text(title)
            .font_family(title_font.family)
            .font_size(title_font.size as f32)
            .font_weight(FontWeight::from(title_font.weight as i32))
            .color(theme::color(title_color))
            .max_lines(1)
            .text_overflow(TextOverflow::Ellipsis)
            .map(self.probe, |label, id| {
                label.a11y_builder(move |node| node.set_author_id(id))
            });

        let (bar, height) = if self.sidebar {
            let occupied = control.map_or(0.0, |control| control.occupied());
            self.sidebar_bar(traffic, title_label, drag, background, occupied)
        } else {
            self.centered_bar(
                traffic,
                title_label,
                drag,
                background,
                close_only,
                content_widths,
            )
        };
        place_overlaid(
            crate::chrome::revealable_bar(bar, height),
            height,
            controls.as_ref(),
            overlaid,
        )
    }
}

impl Titlebar {
    fn sidebar_bar(
        &self,
        traffic: Option<Element>,
        title_label: Label,
        drag: impl Fn(Event<MouseEventData>) + 'static,
        background: ct_design::Rgba,
        occupied: f32,
    ) -> (Element, f32) {
        let island = theme::resolved().sidebar.style == SidebarStyle::Island;
        let (height, padding_top) = sidebar_bar_metrics();
        let padding_x = if island {
            SETTINGS.toolbar_padding_x as f32
        } else {
            ct_design::TITLEBAR_PADDING_X
        };
        let inset = if island { SIDEBAR.inset as f32 } else { 0.0 };
        let lights_x = inset + SIDEBAR.traffic_x as f32;
        let lights_end = lights_x + ct_appearance::traffic::group_rect().2 as f32;
        let padding_left = if traffic.is_some() {
            padding_x.max(lights_end + padding_x - occupied)
        } else {
            padding_x - inset
        };
        // The lights stay at the sidebar's window position while it folds (fold in place).
        let fixed_lights = traffic.map(|traffic| {
            rect()
                .position(
                    Position::new_absolute()
                        .left(lights_x - occupied - padding_left)
                        .top(inset + SIDEBAR.traffic_y as f32 - padding_top),
                )
                .layer(TITLEBAR_OVERLAY_LAYER + 1)
                .child(traffic)
        });

        let left_slot = rect()
            .width(Size::auto())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .spacing(padding_x)
            .probe("titlebar-left-slot")
            .maybe_child(self.left.clone());

        let title = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .main_align(Alignment::Start)
            .probe("titlebar-title")
            .on_mouse_down(drag)
            .child(title_label);

        let root = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .background(theme::color(background))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(self.spacing)
            .padding(Gaps::new(padding_top, padding_x, 0.0, padding_left))
            .child(left_slot)
            .child(title)
            .maybe_child(self.right.clone().map(|right| {
                rect()
                    .width(Size::auto())
                    .height(Size::fill())
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .probe("titlebar-right-slot")
                    .child(right)
            }))
            .maybe_child(fixed_lights)
            .maybe_child(self.hairline.then(|| {
                rect()
                    .position(Position::new_absolute().bottom(0.0).left(0.0))
                    .width(Size::percent(100.0))
                    .height(Size::px(ct_design::HAIRLINE))
                    .background(theme::color(crate::sidebar::sidebar_palette().line))
            }));
        (root.into_element(), height)
    }

    fn centered_bar(
        &self,
        traffic: Option<Element>,
        title_label: Label,
        drag: impl Fn(Event<MouseEventData>) + Clone + 'static,
        background: ct_design::Rgba,
        close_only: bool,
        mut content_widths: State<(f32, f32)>,
    ) -> (Element, f32) {
        let height = if close_only {
            TITLEBAR.thickness
        } else {
            TITLEBAR_CLIENT_HEIGHT
        };
        let padding_x = if traffic.is_some() {
            ct_appearance::traffic::inset(height) as f32
        } else {
            ct_design::TITLEBAR_PADDING_X
        };
        let (left_width, right_width) = *content_widths.read();
        let slot_width = (ct_appearance::traffic::group_rect().2 as f32)
            .max(left_width)
            .max(right_width);
        let left_slot = rect()
            .width(Size::px(slot_width))
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .probe("titlebar-left-slot")
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(self.spacing)
                    .on_sized(move |event: Event<SizedEventData>| {
                        let width = event.area.size.width;
                        let right = content_widths.peek().1;
                        content_widths.set_if_modified((width, right));
                    })
                    .maybe_child(traffic)
                    .maybe_child(self.left.clone()),
            );

        let right_slot = rect()
            .width(Size::px(slot_width))
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
                    .maybe_child(self.right.clone()),
            )
            .on_mouse_down(drag.clone());

        let title = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .probe("titlebar-title")
            .on_mouse_down(drag)
            .child(title_label);

        let height = height as f32;
        let root = rect()
            .width(Size::fill())
            .height(Size::px(height))
            .background(theme::color(background))
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
    if let Some(controls) = controls {
        let mut reported = controls.overlay_titlebar_height;
        reported.set_if_modified(if overlaid { height } else { 0.0 });
    }
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
    rect()
        .width(Size::fill())
        .height(Size::px(0.0))
        // ADR 0057: the full-screen glass spans the whole window, whatever column holds the titlebar.
        .child(
            rect()
                .position(Position::new_global().top(top).left(0.0))
                .width(Size::window_percent(100.0))
                .layer(TITLEBAR_OVERLAY_LAYER)
                .child(crate::glass::fullscreen_backdrop(
                    *controls.fullscreen_bar.read(),
                    crate::glass::GlassEdge::Top,
                    height,
                    opacity,
                )),
        )
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .top(top - controls.push_offset())
                        .left(0.0),
                )
                .width(Size::fill())
                .layer(TITLEBAR_OVERLAY_LAYER)
                .opacity(opacity)
                .child(bar),
        )
        .into_element()
}

pub(crate) const TITLEBAR_OVERLAY_LAYER: i16 = i16::MAX / 32;

pub fn sidebar_bar_metrics() -> (f32, f32) {
    if theme::resolved().sidebar.style == SidebarStyle::Island {
        (
            SETTINGS.toolbar_height as f32,
            SETTINGS.toolbar_padding_top as f32,
        )
    } else {
        (TITLEBAR_CLIENT_HEIGHT as f32, 0.0)
    }
}

pub fn window_traffic_lights(focused: bool, fixed: bool) -> TrafficLights {
    let green = green_action(window_behavior().green_button, fixed);
    TrafficLights::new()
        .round(
            crate::chrome::chrome_style().round_lights
                || crate::theme::resolved().traffic_shape
                    == ct_appearance::TrafficLightShape::Round,
        )
        .focused(focused)
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

pub fn window_drag_handler(fixed: bool) -> impl Fn(Event<MouseEventData>) + Clone {
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

pub fn app_id(app_name: &str) -> String {
    format!("{NATIVE_APP_ID_PREFIX}{app_name}")
}
