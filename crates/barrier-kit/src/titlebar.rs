//! Client-side titlebar with traffic lights, title and toolbar area.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::traffic::TrafficLights;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_appearance::SidebarStyle;
use vow_appearance::palette;
use vow_design::NATIVE_APP_ID_PREFIX;
use vow_design::{
    ButtonKind, Rgba, SETTINGS, SIDEBAR, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_DOUBLE_CLICK,
};

use crate::ProbeExt;
use crate::platform::{DecorationResponse, WindowControls, WindowEdge, WindowStatus};

#[derive(PartialEq)]
pub struct Titlebar {
    preview: Option<Preview>,
    title: Option<String>,
    on_close: Option<Callback<(), ()>>,
    sidebar: bool,
    left: Option<Element>,
    right: Option<Element>,
    probe: Option<&'static str>,
    spacing: f32,
    padding_x: Option<f32>,
    title_font: Option<vow_design::Font>,
    title_line_height: Option<f32>,
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
            on_close: None,
            sidebar: false,
            left: None,
            right: None,
            probe: None,
            spacing: 0.0,
            padding_x: None,
            title_font: None,
            title_line_height: None,
        }
    }

    pub fn preview(status: WindowStatus, title: impl Into<String>, fixed: bool) -> Self {
        Self {
            preview: Some(Preview {
                status,
                title: title.into(),
                fixed,
            }),
            title: None,
            on_close: None,
            sidebar: false,
            left: None,
            right: None,
            probe: None,
            spacing: 0.0,
            padding_x: None,
            title_font: None,
            title_line_height: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn on_close(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_close = Some(callback.into());
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

    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x = Some(padding_x);
        self
    }

    pub fn title_font(mut self, font: vow_design::Font) -> Self {
        self.title_font = Some(font);
        self
    }

    pub fn title_line_height(mut self, line_height: f32) -> Self {
        self.title_line_height = Some(line_height);
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
        let (status, title, fixed) = if let Some(preview) = &self.preview {
            let title = self.title.clone().unwrap_or_else(|| preview.title.clone());
            (preview.status, title, preview.fixed)
        } else if let Some(controls) = try_consume_root_context::<WindowControls>() {
            let status = *controls.status.read();
            let title = self.title.clone().unwrap_or_else(|| controls.title.clone());
            (status, title, controls.fixed)
        } else {
            (
                WindowStatus::default(),
                self.title.clone().unwrap_or_default(),
                false,
            )
        };
        let drag = window_drag_handler(fixed);
        if status.decoration == DecorationResponse::Server {
            return rect()
                .width(Size::fill())
                .height(Size::px(0.0))
                .into_element();
        }

        let controls = freya_core::prelude::try_consume_root_context::<WindowControls>();
        let overlaid = controls
            .as_ref()
            .is_some_and(|controls| controls.titlebar_overlaid());
        let overlay_top = controls
            .as_ref()
            .and_then(|controls| controls.titlebar_overlay_top());
        let place = move |bar: Element| -> Element {
            if !overlaid {
                return bar;
            }
            rect()
                .width(Size::fill())
                .height(Size::px(0.0))
                .maybe(overlay_top.is_some(), |el| {
                    el.child(
                        rect()
                            .position(
                                Position::new_absolute()
                                    .top(overlay_top.unwrap_or_default())
                                    .left(0.0)
                                    .right(0.0),
                            )
                            .width(Size::fill())
                            .layer(TITLEBAR_OVERLAY_LAYER)
                            .child(bar),
                    )
                })
                .into_element()
        };

        let resolved = crate::theme::resolved();
        let theme = palette(resolved.scheme);
        let sidebar = self.sidebar
            && crate::views::split::sidebar_control().is_none_or(|control| control.visible());
        let tinted = crate::backdrop::tinted().is_some();
        let ink = crate::style::Palette::current();
        let background = if tinted && overlaid {
            ink.raised_background
        } else if tinted {
            vow_design::Rgba::rgba8(0, 0, 0, 0.0)
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
        let lights_shown = freya_core::prelude::try_consume_root_context::<WindowControls>()
            .is_none_or(|controls| controls.traffic_lights_shown());
        let vertical = crate::chrome::chrome_style().layout != crate::chrome::TitlebarLayout::Top;
        let traffic = if sidebar || !lights_shown || vertical {
            None
        } else {
            Some(window_traffic_lights(
                self.on_close.clone(),
                status.activated,
                fixed,
            ))
        };

        let title_font = self.title_font.unwrap_or(TITLEBAR.font);
        let mut title_label = label()
            .text(title.clone())
            .font_family(title_font.family)
            .font_size(title_font.size as f32)
            .font_weight(FontWeight::from(title_font.weight as i32))
            .color(rgba(title_color))
            .max_lines(1)
            .text_overflow(TextOverflow::Ellipsis);
        if let Some(line_height) = self.title_line_height {
            title_label = title_label.line_height(line_height);
        }
        if let Some(id) = self.probe {
            title_label = title_label.a11y_builder(move |node| node.set_author_id(id));
        }

        if self.sidebar {
            let island = resolved.sidebar.style == SidebarStyle::Island;
            let (height, padding_top, default_padding_x) = if island {
                (
                    SETTINGS.toolbar_height as f32,
                    SETTINGS.toolbar_padding_top as f32,
                    SETTINGS.toolbar_padding_x as f32,
                )
            } else {
                (TITLEBAR_CLIENT_HEIGHT as f32, 0.0, 10.0)
            };
            let padding_x = self.padding_x.unwrap_or(default_padding_x);
            let padding_left = if traffic.is_some() {
                (if island { SIDEBAR.inset } else { 0 } + SIDEBAR.traffic_x) as f32
            } else {
                padding_x
            };

            let left_slot = rect()
                .width(Size::auto())
                .height(Size::fill())
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .main_align(Alignment::Center)
                .spacing(padding_x)
                .a11y_alt("titlebar-left-slot")
                .maybe_child(traffic)
                .maybe(self.left.is_some(), |el| {
                    el.child(self.left.clone().unwrap())
                });

            let title = rect()
                .width(Size::flex(1.0))
                .height(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::Center)
                .a11y_alt("titlebar-title")
                .on_mouse_down(drag.clone())
                .child(title_label);

            let mut root = rect()
                .width(Size::fill())
                .height(Size::px(height))
                .background(rgba(background))
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(self.spacing)
                .padding(Gaps::new(padding_top, padding_x, 0.0, padding_left))
                .child(left_slot)
                .child(title);

            if resolved.sidebar.style == SidebarStyle::Flat {
                root = root.child(
                    rect()
                        .position(Position::new_absolute().bottom(0.0).left(0.0))
                        .width(Size::fill())
                        .height(Size::px(1.0))
                        .background(rgba(resolved.sidebar.palette.line)),
                );
            }

            return place(root.into_element());
        }

        let min_slot = vow_appearance::traffic::group_rect().2 as f32;
        let padding_left = if traffic.is_some() {
            TITLEBAR.buttons.margin_start as f32
        } else {
            10.0
        };
        let left_slot = rect()
            .width(Size::auto())
            .min_width(Size::px(min_slot))
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(self.spacing)
            .a11y_alt("titlebar-left-slot")
            .maybe_child(traffic)
            .maybe_child(self.left.clone());

        let right_slot = rect()
            .width(Size::auto())
            .min_width(Size::px(min_slot))
            .height(Size::fill())
            .cross_align(Alignment::Center)
            .a11y_alt("titlebar-right-slot")
            .maybe(self.right.is_some(), |el| {
                el.child(self.right.clone().unwrap())
            })
            .on_mouse_down(drag.clone());

        let title = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .a11y_alt("titlebar-title")
            .on_mouse_down(drag)
            .child(title_label);

        let root = rect()
            .width(Size::fill())
            .height(Size::px(TITLEBAR_CLIENT_HEIGHT as f32))
            .background(rgba(background))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .padding(Gaps::new(0.0, 10.0, 0.0, padding_left))
            .child(left_slot)
            .child(title)
            .child(right_slot);

        place(root.into_element())
    }
}

pub(crate) const TITLEBAR_OVERLAY_LAYER: i16 = i16::MAX / 32;

pub fn window_traffic_lights(
    on_close: Option<Callback<(), ()>>,
    focused: bool,
    fixed: bool,
) -> TrafficLights {
    let behavior = *window_behavior();
    let green = green_action(behavior.green_button, fixed);
    TrafficLights::new()
        .round(crate::chrome::chrome_style().round_lights)
        .focused(focused)
        .zoom_supported(green_admitted(behavior.green_button, fixed))
        .fixed_width(fixed)
        .on_action(move |kind| {
            let controls = WindowControls::get();
            match kind {
                ButtonKind::Close => match &on_close {
                    Some(on_close) => on_close.call(()),
                    None => controls.close(),
                },
                ButtonKind::Minimize => controls.minimize(),
                ButtonKind::Maximize if fixed => controls.toggle_vertical_maximize(),
                ButtonKind::Maximize => run_action(&controls, green),
            }
        })
}

pub fn window_drag_handler(fixed: bool) -> impl Fn(Event<MouseEventData>) + Clone {
    let last_click = use_hook(|| Rc::new(Cell::new(None::<(Instant, (f64, f64))>)));
    let double_done = use_hook(|| Rc::new(Cell::new(false)));
    let fixed_cell = use_hook(|| Rc::new(Cell::new(fixed)));
    fixed_cell.set(fixed);
    move |event: Event<MouseEventData>| {
        if event.button != Some(MouseButton::Left) {
            return;
        }
        let fixed = fixed_cell.get();
        let controls = WindowControls::get();
        let behavior = *window_behavior();
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
            if !double_done.get() {
                double_done.set(true);
                run_action(
                    &controls,
                    double_click_action(behavior.double_click_titlebar, fixed),
                );
            }
        } else {
            last_click.set(Some((now, point)));
            double_done.set(false);
            controls.move_window();
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GreenButtonAction {
    #[default]
    Focus,
    Zoom,
    Fullscreen,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DoubleClickAction {
    Fill,
    Minimize,
    #[default]
    Zoom,
    None,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
struct WindowBehavior {
    double_click_titlebar: DoubleClickAction,
    green_button: GreenButtonAction,
}

fn window_behavior() -> &'static WindowBehavior {
    static BEHAVIOR: OnceLock<WindowBehavior> = OnceLock::new();
    BEHAVIOR.get_or_init(|| {
        let path = vow_appearance::config_dir().join("window.domain");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitlebarAction {
    Focus,
    Maximize,
    Minimize,
    Fullscreen,
    None,
}

fn green_action(action: GreenButtonAction, fixed: bool) -> TitlebarAction {
    match action {
        GreenButtonAction::Focus => TitlebarAction::Focus,
        GreenButtonAction::Zoom if fixed => TitlebarAction::None,
        GreenButtonAction::Zoom => TitlebarAction::Maximize,
        GreenButtonAction::Fullscreen => TitlebarAction::Fullscreen,
        GreenButtonAction::None => TitlebarAction::None,
    }
}

fn green_admitted(action: GreenButtonAction, fixed: bool) -> bool {
    action != GreenButtonAction::Zoom || !fixed
}

fn double_click_action(action: DoubleClickAction, fixed: bool) -> TitlebarAction {
    match action {
        DoubleClickAction::Fill | DoubleClickAction::Zoom if fixed => TitlebarAction::None,
        DoubleClickAction::Fill | DoubleClickAction::Zoom => TitlebarAction::Maximize,
        DoubleClickAction::Minimize => TitlebarAction::Minimize,
        DoubleClickAction::None => TitlebarAction::None,
    }
}

fn run_action(controls: &WindowControls, action: TitlebarAction) {
    match action {
        TitlebarAction::Maximize => controls.toggle_maximize(),
        TitlebarAction::Minimize => controls.minimize(),
        TitlebarAction::Focus | TitlebarAction::Fullscreen => controls.toggle_fullscreen(),
        TitlebarAction::None => {}
    }
}

fn rgba(color: Rgba) -> Color {
    let [r, g, b, a] = color.to_rgba8();
    Color::from_argb(a, r, g, b)
}

pub fn app_id(app_name: &str) -> String {
    format!("{NATIVE_APP_ID_PREFIX}{app_name}")
}

pub fn resize_edges() -> Element {
    let controls = WindowControls::get();
    let make = |edge: WindowEdge, position: Position, width: Size, height: Size| {
        let controls = controls.clone();
        rect()
            .position(position)
            .width(width)
            .height(height)
            .background(Color::TRANSPARENT)
            .on_mouse_down(move |event: Event<MouseEventData>| {
                if event.button == Some(MouseButton::Left) {
                    controls.resize(edge);
                }
            })
            .into_element()
    };
    let edge = Size::px(6.0);
    let corner = Size::px(12.0);
    rect()
        .position(Position::new_absolute().top(0.0).left(0.0))
        .width(Size::window_percent(100.0))
        .height(Size::window_percent(100.0))
        .background(Color::TRANSPARENT)
        .child(make(
            WindowEdge::Top,
            Position::new_absolute().top(0.0).left(0.0),
            Size::fill(),
            edge.clone(),
        ))
        .child(make(
            WindowEdge::Bottom,
            Position::new_absolute().bottom(0.0).left(0.0),
            Size::fill(),
            edge.clone(),
        ))
        .child(make(
            WindowEdge::Left,
            Position::new_absolute().top(0.0).left(0.0),
            edge.clone(),
            Size::fill(),
        ))
        .child(make(
            WindowEdge::Right,
            Position::new_absolute().top(0.0).right(0.0),
            edge.clone(),
            Size::fill(),
        ))
        .child(make(
            WindowEdge::TopLeft,
            Position::new_absolute().top(0.0).left(0.0),
            corner.clone(),
            corner.clone(),
        ))
        .child(make(
            WindowEdge::TopRight,
            Position::new_absolute().top(0.0).right(0.0),
            corner.clone(),
            corner.clone(),
        ))
        .child(make(
            WindowEdge::BottomLeft,
            Position::new_absolute().bottom(0.0).left(0.0),
            corner.clone(),
            corner.clone(),
        ))
        .child(make(
            WindowEdge::BottomRight,
            Position::new_absolute().bottom(0.0).right(0.0),
            corner.clone(),
            corner,
        ))
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_green_light_uses_the_configured_action() {
        assert_eq!(
            green_action(GreenButtonAction::Focus, false),
            TitlebarAction::Focus
        );
        assert_eq!(
            green_action(GreenButtonAction::Zoom, false),
            TitlebarAction::Maximize
        );
        assert_eq!(
            green_action(GreenButtonAction::Zoom, true),
            TitlebarAction::None
        );
        assert_eq!(
            green_action(GreenButtonAction::Fullscreen, false),
            TitlebarAction::Fullscreen
        );
        assert_eq!(
            green_action(GreenButtonAction::None, false),
            TitlebarAction::None
        );
        assert!(green_admitted(GreenButtonAction::Focus, true));
        assert!(!green_admitted(GreenButtonAction::Zoom, true));
    }

    #[test]
    fn the_double_click_uses_the_configured_action() {
        assert_eq!(
            double_click_action(DoubleClickAction::Zoom, false),
            TitlebarAction::Maximize
        );
        assert_eq!(
            double_click_action(DoubleClickAction::Fill, false),
            TitlebarAction::Maximize
        );
        assert_eq!(
            double_click_action(DoubleClickAction::Minimize, false),
            TitlebarAction::Minimize
        );
        assert_eq!(
            double_click_action(DoubleClickAction::None, false),
            TitlebarAction::None
        );
        assert_eq!(
            double_click_action(DoubleClickAction::Zoom, true),
            TitlebarAction::None
        );
    }

    #[test]
    fn window_domain_defaults_match_the_compositor() {
        let behavior: WindowBehavior = toml::from_str("").expect("empty domain");
        assert_eq!(behavior.green_button, GreenButtonAction::Focus);
        assert_eq!(behavior.double_click_titlebar, DoubleClickAction::Zoom);

        let behavior: WindowBehavior =
            toml::from_str("green_button = \"zoom\"\ndouble_click_titlebar = \"minimize\"\n")
                .expect("window.domain");
        assert_eq!(behavior.green_button, GreenButtonAction::Zoom);
        assert_eq!(behavior.double_click_titlebar, DoubleClickAction::Minimize);
    }
}
