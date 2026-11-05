//! Tests for the sidebar, sidebar layout and titlebar with sidebar.

use std::env;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use accesskit::Role;
use barrier_kit::ProbeExt;
use barrier_kit::controls::ToolGroup;
use barrier_kit::platform::{DecorationResponse, WindowStatus};
use barrier_kit::sidebar::{Sidebar, SidebarItem, SidebarSection};
use barrier_kit::theme;
use barrier_kit::titlebar::Titlebar;
use barrier_kit::views::split::{NarrowSidebar, SidebarToggle, SplitView};
use freya_components::button::Button;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Alignment, Direction, Size, Size2D};
use vow_appearance::{AppearanceConfig, ColorScheme, SidebarStyle, WindowColorMode, resolve};
use vow_design::{SIDEBAR, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TOOLGROUP};

#[path = "kit/image.rs"]
mod image;

use image::{decode, pixel};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn color(c: vow_design::Rgba) -> Color {
    let [r, g, b, a] = c.to_rgba8();
    Color::from_argb(a, r, g, b)
}

type Capture = Arc<Mutex<Option<(f32, f32, f32, f32)>>>;

fn with_config<F: FnOnce() -> R, R>(style: &str, f: F) -> R {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("malaysia-sidebar-test-{style}"));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("appearance.domain");
    fs::write(&path, format!("sidebar_style = \"{style}\"\n")).unwrap();
    let old = env::var_os("MALAYSIA_CONFIG_DIR");
    unsafe { env::set_var("MALAYSIA_CONFIG_DIR", &dir) };
    let result = f();
    match old {
        Some(v) => unsafe { env::set_var("MALAYSIA_CONFIG_DIR", v) },
        None => unsafe { env::remove_var("MALAYSIA_CONFIG_DIR") },
    }
    result
}

struct LayoutCase {
    capture: Capture,
}

impl App for LayoutCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let capture = self.capture.clone();
        SplitView::new(
            Sidebar::new(),
            rect().width(Size::fill()).height(Size::fill()).on_sized(
                move |event: Event<SizedEventData>| {
                    let area = event.data().area;
                    *capture.lock().unwrap() = Some((
                        area.origin.x,
                        area.origin.y,
                        area.size.width,
                        area.size.height,
                    ));
                },
            ),
        )
    }
}

fn layout_position() -> (f32, f32, f32, f32) {
    let capture: Capture = Arc::new(Mutex::new(None));
    let app = AppComponent::new(LayoutCase {
        capture: capture.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(800.0, 600.0), |_| {}, 1.0);
    test.sync_and_update();
    capture.lock().unwrap().expect("content was sized")
}

#[test]
fn island_layout_positions_content_at_sidebar_width_plus_double_inset() {
    let (x, y, width, height) = with_config("island", layout_position);
    let expected_x = SIDEBAR.width as f32 + 2.0 * SIDEBAR.inset as f32;
    assert!(
        (x - expected_x).abs() < 0.5,
        "island content x should be {expected_x}, got {x}"
    );
    assert_eq!(y, 0.0, "island content must start at the top edge");
    let expected_width = 800.0 - expected_x;
    assert!(
        (width - expected_width).abs() < 0.5,
        "island content width should be {expected_width}, got {width}"
    );
    assert_eq!(height, 600.0, "island content must fill the full height");
}

#[test]
fn flat_layout_positions_content_at_sidebar_width() {
    let (x, y, width, height) = with_config("flat", layout_position);
    let expected_x = SIDEBAR.width as f32;
    assert!(
        (x - expected_x).abs() < 0.5,
        "flat content x should be {expected_x}, got {x}"
    );
    assert_eq!(y, 0.0, "flat content must start at the top edge");
    let expected_width = 800.0 - expected_x;
    assert!(
        (width - expected_width).abs() < 0.5,
        "flat content width should be {expected_width}, got {width}"
    );
    assert_eq!(height, 600.0, "flat content must fill the full height");
}

struct TitlebarCase {
    sidebar: bool,
    status: WindowStatus,
}

impl App for TitlebarCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Titlebar::preview(self.status, "Ventana", false).with_sidebar(self.sidebar)
    }
}

fn render_titlebar(sidebar: bool, decoration: DecorationResponse) -> Vec<u8> {
    let app = AppComponent::new(TitlebarCase {
        sidebar,
        status: WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration,
            fixed: false,
        },
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    test.render().as_ref().to_vec()
}

fn render_titlebar_pixels(
    activated: bool,
    resolved: vow_appearance::Resolved,
) -> (usize, usize, Vec<u8>) {
    let app = AppComponent::new(TitlebarCase {
        sidebar: false,
        status: WindowStatus {
            activated,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed: false,
        },
    });
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 120.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    decode(test.render().as_bytes())
}

#[test]
fn titlebar_background_follows_the_resolved_window_tint() {
    let accent = vow_design::Rgba {
        r: 0.2,
        g: 0.4,
        b: 0.9,
        a: 1.0,
    };
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: ColorScheme::Light,
            accent_color: Some([accent.r, accent.g, accent.b]),
            window_color: WindowColorMode::Accent,
            ..Default::default()
        },
        None,
    );

    let (width, _height, focused) = render_titlebar_pixels(true, resolved);
    assert_eq!(
        pixel(&focused, width, 260, 20),
        resolved.surfaces.window.to_rgba8(),
        "a focused kit titlebar must use the tinted window surface"
    );

    let (width, _height, unfocused) = render_titlebar_pixels(false, resolved);
    assert_eq!(
        pixel(&unfocused, width, 260, 20),
        resolved.surfaces.titlebar_inactive.to_rgba8(),
        "an unfocused kit titlebar must use the tinted inactive titlebar surface"
    );
}

fn titlebar_layout(sidebar: bool) -> Vec<(f32, f32, f32, f32)> {
    let app = AppComponent::new(TitlebarCase {
        sidebar,
        status: WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed: false,
        },
    });
    let (test, _) = TestingRunner::new(app, Size2D::new(300.0, 120.0), |_| {}, 1.0);
    test.find_many(|node, el| {
        let accessibility = el.accessibility();
        let label = accessibility.builder.label()?;
        let is_slot = matches!(
            label,
            "titlebar-left-slot" | "titlebar-title" | "titlebar-right-slot"
        );
        is_slot.then(|| {
            let area = node.layout().area;
            (
                area.origin.x,
                area.origin.y,
                area.size.width,
                area.size.height,
            )
        })
    })
}

#[test]
fn titlebar_without_sidebar_keeps_the_classic_three_slot_layout() {
    let bars = titlebar_layout(false);
    assert_eq!(
        bars.len(),
        3,
        "expected the three classic slots, found {bars:?}"
    );
    let widths: Vec<f32> = bars.iter().map(|(_, _, w, _)| *w).collect();
    let group_w = vow_appearance::traffic::group_rect().2 as f32;
    assert!(
        !widths.contains(&80.0),
        "left/right slots must not use a fixed 80 px width: {widths:?}"
    );
    assert!(
        widths.iter().any(|width| (*width - group_w).abs() < 0.5),
        "slots must keep at least the traffic-light width ({group_w}): {widths:?}"
    );
}

#[test]
fn titlebar_with_sidebar_uses_elastic_side_slots() {
    let bars = titlebar_layout(true);
    assert!(
        !bars.iter().any(|(_, _, w, _)| *w == 80.0),
        "with_sidebar(true) must not use fixed 80 px slots: {bars:?}"
    );
}

struct WideSlotsCase;

impl App for WideSlotsCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Titlebar::preview(
            WindowStatus {
                activated: true,
                maximized: false,
                fullscreen: false,
                decoration: DecorationResponse::Client,
                fixed: false,
            },
            "Ventana",
            false,
        )
        .left(rect().width(Size::px(120.0)).height(Size::px(10.0)))
        .right(rect().width(Size::px(100.0)).height(Size::px(10.0)))
    }
}

#[test]
fn titlebar_slots_grow_with_their_content() {
    let app = AppComponent::new(WideSlotsCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(400.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    test.sync_and_update();
    let slot_width = |label: &str| {
        test.find(|node, el| {
            (el.accessibility().builder.label() == Some(label))
                .then(|| node.layout().area.size.width)
        })
        .unwrap_or_else(|| panic!("{label} exists"))
    };
    assert!(slot_width("titlebar-left-slot") >= 120.0);
    assert!(slot_width("titlebar-right-slot") >= 100.0);
    assert!(slot_width("titlebar-title") > 0.0);
}

#[test]
fn titlebar_title_uses_the_titlebar_font() {
    let app = AppComponent::new(TitlebarCase {
        sidebar: false,
        status: WindowStatus {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed: false,
        },
    });
    let (test, _) = TestingRunner::new(app, Size2D::new(300.0, 120.0), |_| {}, 1.0);
    let style = test
        .find_many(|_node, el| {
            let label = Label::try_downcast(el)?;
            (label.text == "Ventana").then(|| el.text_style().into_owned())
        })
        .into_iter()
        .next()
        .expect("the title label exists");

    assert_eq!(
        style.font_weight,
        Some(FontWeight::from(TITLEBAR.font.weight as i32)),
        "the title label must use the TITLEBAR font weight"
    );
    assert_eq!(
        style.font_size.map(f32::from),
        Some(TITLEBAR.font.size as f32),
        "the title label must use the TITLEBAR font size"
    );
    assert!(
        style
            .font_families
            .iter()
            .any(|family| family.as_ref() == TITLEBAR.font.family),
        "the title label must use the TITLEBAR font family"
    );
}

#[test]
fn titlebar_with_sidebar_hides_traffic_lights() {
    let plain = render_titlebar(false, DecorationResponse::Client);
    let with_sidebar = render_titlebar(true, DecorationResponse::Client);
    assert_ne!(
        plain, with_sidebar,
        "a sidebar titlebar must look different from one with traffic lights"
    );
}

#[test]
fn the_dragged_sidebar_edge_stays_under_the_pointer() {
    let capture: Capture = Arc::new(Mutex::new(None));
    let app = AppComponent::new(LayoutCase {
        capture: capture.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(800.0, 600.0), |_| {}, 1.0);
    test.sync_and_update();
    test.poll(Duration::from_millis(16), Duration::from_millis(400));
    let detail_x = |capture: &Capture| capture.lock().unwrap().expect("content was sized").0;
    let edge = f64::from(detail_x(&capture)) - 1.0;
    test.move_cursor((edge, 300.0));
    test.press_cursor((edge, 300.0));
    test.sync_and_update();
    let pointer = edge + 40.0;
    test.move_cursor((pointer, 300.0));
    test.sync_and_update();
    test.sync_and_update();
    let grip_centre = detail_x(&capture) - vow_design::SPLIT_VIEW.grip / 2.0;
    assert!((grip_centre - pointer as f32).abs() < 1.0);
    test.release_cursor((pointer, 300.0));
}

struct ToggleApp;

impl App for ToggleApp {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        SidebarSection::new("Favoritos").child(SidebarItem::new("Uno"))
    }
}

fn toggle_test() -> TestingRunner {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let app = AppComponent::new(ToggleApp);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 300.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    test
}

fn expanded(test: &TestingRunner) -> bool {
    section_content_height(test) > 0.0
}

#[test]
fn sidebar_section_toggles_once_per_click() {
    let mut test = toggle_test();
    test.move_cursor((5.0, 15.0));
    test.press_cursor((5.0, 15.0));
    assert!(
        expanded(&test),
        "mouse down alone must not toggle the section"
    );
    test.release_cursor((5.0, 15.0));
    assert!(!expanded(&test), "the click must collapse the section");
}

#[test]
fn sidebar_section_toggles_with_enter_and_space() {
    let mut test = toggle_test();
    assert!(expanded(&test), "section starts expanded");

    test.press_key(Key::Named(NamedKey::Tab));
    test.press_key(Key::Named(NamedKey::Enter));
    assert!(!expanded(&test), "Enter should collapse the section");

    test.press_key(Key::Character(" ".into()));
    assert!(expanded(&test), "Space should expand the section");
}

struct AnimatedSectionCase;

impl App for AnimatedSectionCase {
    fn render(&self) -> impl IntoElement {
        Sidebar::new().child(
            SidebarSection::new("Grande")
                .child(rect().width(Size::px(50.0)).height(Size::px(120.0))),
        )
    }
}

fn animated_section_test() -> (TestingRunner, State<vow_appearance::Resolved>) {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: false,
            ..Default::default()
        },
        None,
    );
    let app = AppComponent::new(AnimatedSectionCase);
    TestingRunner::new(
        app,
        Size2D::new(300.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    )
}

fn section_content_area(test: &TestingRunner) -> (f32, f32, f32, f32) {
    test.find_many(|node, el| {
        el.effect()
            .is_some_and(|effect| effect.overflow == Overflow::Clip)
            .then(|| {
                let area = node.layout().area;
                (
                    area.origin.x,
                    area.origin.y,
                    area.size.width,
                    area.size.height,
                )
            })
    })
    .pop()
    .expect("the section content container exists")
}

fn section_content_height(test: &TestingRunner) -> f32 {
    section_content_area(test).3
}

fn labeled_area(test: &TestingRunner, label: &str) -> (f32, f32, f32, f32) {
    test.find(|node, el| {
        (el.accessibility().builder.label() == Some(label)).then(|| {
            let area = node.layout().area;
            (
                area.origin.x,
                area.origin.y,
                area.size.width,
                area.size.height,
            )
        })
    })
    .unwrap_or_else(|| panic!("{label} exists"))
}

#[test]
fn expanded_section_opens_at_full_height_without_animating() {
    let (mut test, _) = animated_section_test();
    test.sync_and_update();
    let height = section_content_height(&test);
    assert!(
        (height - 120.0).abs() < 0.5,
        "an expanded section must open at its child's 120 px on the first sync, got {height}"
    );
}

#[test]
fn collapsing_section_animates_from_full_height() {
    let (mut test, _) = animated_section_test();
    test.sync_and_update();
    assert!((section_content_height(&test) - 120.0).abs() < 0.5);

    test.animation_clock().set_speed(0.05);
    test.click_cursor((12.0, 68.0));
    test.poll_n(Duration::from_millis(100), 4);

    let height = section_content_height(&test);
    assert!(
        height > 0.0 && height < 120.0,
        "the collapsing section must animate below 120 px, got {height}"
    );
}

#[test]
fn live_reduced_motion_collapses_without_animating() {
    let (mut test, mut resolved) = animated_section_test();
    test.sync_and_update();
    assert!((section_content_height(&test) - 120.0).abs() < 0.5);

    resolved.set(resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    ));
    test.sync_and_update();

    test.click_cursor((12.0, 68.0));
    let height = section_content_height(&test);
    assert!(
        height == 0.0,
        "reduced motion must collapse on the first frame, got {height}"
    );
}

struct DrawerCase;

impl App for DrawerCase {
    fn render(&self) -> impl IntoElement {
        Sidebar::new().child(
            SidebarSection::new("Cajón")
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(40.0))
                        .a11y_alt("drawer-first"),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(80.0))
                        .a11y_alt("drawer-last"),
                ),
        )
    }
}

#[test]
fn closing_section_slides_as_a_drawer() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: false,
            ..Default::default()
        },
        None,
    );
    let app = AppComponent::new(DrawerCase);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    test.animation_clock().set_speed(0.05);
    test.click_cursor((12.0, 68.0));

    let mut clip = section_content_area(&test);
    for _ in 0..60 {
        if clip.3 <= 72.0 {
            break;
        }
        test.poll_n(Duration::from_millis(100), 1);
        clip = section_content_area(&test);
    }
    assert!(
        clip.3 > 12.0 && clip.3 <= 72.0,
        "expected the drawer around half closed, got {}",
        clip.3
    );

    let first = labeled_area(&test, "drawer-first");
    let last = labeled_area(&test, "drawer-last");
    let clip_top = clip.1;
    let clip_bottom = clip.1 + clip.3;
    assert!(
        first.1 + first.3 <= clip_top + 0.5,
        "the first row must stay above the clip top ({clip_top}), got {first:?}"
    );
    assert!(
        last.1 < clip_bottom && last.1 + last.3 > clip_top,
        "the last row must be inside the visible area ({clip_top}..{clip_bottom}), got {last:?}"
    );
}

#[test]
fn sidebar_section_toggles_once_per_tap() {
    let mut test = toggle_test();
    test.press_touch((5.0, 15.0));
    assert!(
        expanded(&test),
        "touch start alone must not toggle the section"
    );
    test.release_touch((5.0, 15.0));
    assert!(!expanded(&test), "the tap must collapse the section");
}

struct ToolGroupCase;

impl App for ToolGroupCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        ToolGroup::new()
            .child(Button::new().child("A"))
            .child(Button::new().child("B"))
            .into_element()
    }
}

#[test]
fn toolgroup_renders_without_panic() {
    let app = AppComponent::new(ToolGroupCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 80.0), |_| {}, 1.0);
    test.sync_and_update();
    let png = test.render();
    assert!(!png.as_ref().is_empty());
}

struct TrafficLightsCase;

impl App for TrafficLightsCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Sidebar::new().traffic_lights(true).child(
            rect()
                .width(Size::fill())
                .height(Size::px(30.0))
                .a11y_alt("sidebar-first-child"),
        )
    }
}

fn traffic_layout() -> Vec<(f32, f32, f32, f32)> {
    let app = AppComponent::new(TrafficLightsCase);
    let (test, _) = TestingRunner::new(app, Size2D::new(300.0, 400.0), |_| {}, 1.0);
    test.find_many(|node, _el| {
        let area = node.layout().area;
        Some((
            area.origin.x,
            area.origin.y,
            area.size.width,
            area.size.height,
        ))
    })
}

#[test]
fn sidebar_traffic_lights_are_at_design_offset_and_first_child_sits_below() {
    with_config("island", || {
        let (_, _, group_w, group_h) = vow_appearance::traffic::group_rect();
        let layouts = traffic_layout();

        let lights = layouts
            .iter()
            .find(|(_, _, w, h)| {
                (*w - group_w as f32).abs() < 0.5 && (*h - group_h as f32).abs() < 0.5
            })
            .expect("traffic lights container must be sized");
        assert!(
            (lights.0 - SIDEBAR.traffic_x as f32).abs() < 0.5
                && (lights.1 - SIDEBAR.traffic_y as f32).abs() < 0.5,
            "traffic lights must be at ({}, {}), got ({}, {})",
            SIDEBAR.traffic_x,
            SIDEBAR.traffic_y,
            lights.0,
            lights.1
        );

        let first_child = layouts
            .iter()
            .find(|(_, _, w, h)| *w > 100.0 && (*h - 30.0).abs() < 0.5)
            .expect("first child must be laid out");
        let lights_bottom = lights.1 + lights.3;
        assert!(
            first_child.1 >= lights_bottom - 0.5,
            "first child must sit below the traffic lights (lights end at {lights_bottom}, child starts at {})",
            first_child.1
        );
    });
}

#[test]
fn sidebar_traffic_light_tokens_are_sane() {
    assert_eq!(SIDEBAR.traffic_x, 11);
    assert_eq!(SIDEBAR.traffic_y, 11);
    assert_eq!(TITLEBAR_CLIENT_HEIGHT, 44);
    assert_eq!(TITLEBAR.buttons.diameter, 14);
}

struct ItemCase;

impl App for ItemCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Sidebar::new().child(SidebarItem::new("Hola"))
    }
}

#[test]
fn sidebar_item_uses_design_padding_and_gap() {
    let app = AppComponent::new(ItemCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 300.0), |_| {}, 1.0);
    test.sync_and_update();
    let png = test.render();
    assert!(!png.as_ref().is_empty());
}

fn crop(image: &[u8], width: usize, x: usize, y: usize, size: usize) -> Vec<[u8; 4]> {
    let mut out = Vec::with_capacity(size * size);
    for row in 0..size {
        for col in 0..size {
            out.push(pixel(image, width, x + col, y + row));
        }
    }
    out
}

struct ChevronCase;

impl App for ChevronCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Sidebar::new().child(SidebarSection::new("Favoritos").child(SidebarItem::new("Uno")))
    }
}

fn chevron_area(test: &TestingRunner) -> (usize, usize) {
    test.find(|node, _el| {
        let size = node.layout().area.size;
        ((size.width - SIDEBAR.chevron as f32).abs() < 0.5
            && (size.height - SIDEBAR.chevron as f32).abs() < 0.5)
            .then(|| {
                let origin = node.layout().area.origin;
                (origin.x as usize, origin.y as usize)
            })
    })
    .expect("the chevron is visible while the section is focused")
}

#[test]
fn sidebar_section_chevron_changes_shape_when_toggled() {
    let app = AppComponent::new(ChevronCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 300.0), |_| {}, 1.0);
    let size = SIDEBAR.chevron as usize;

    test.move_cursor((20.0, 68.0));
    test.click_cursor((20.0, 68.0));
    test.sync_and_update();
    let (width, _, collapsed_image) = decode(test.render().as_bytes());
    let (x, y) = chevron_area(&test);
    let collapsed = crop(&collapsed_image, width, x, y, size);

    test.click_cursor((20.0, 68.0));
    test.sync_and_update();
    let (width, _, expanded_image) = decode(test.render().as_bytes());
    let (x, y) = chevron_area(&test);
    let expanded = crop(&expanded_image, width, x, y, size);

    assert_ne!(
        collapsed, expanded,
        "the chevron must change direction when the section collapses or expands"
    );
}

struct TallSectionCase;

impl App for TallSectionCase {
    fn render(&self) -> impl IntoElement {
        Sidebar::new().child(
            SidebarSection::new("Grande")
                .child(rect().width(Size::px(50.0)).height(Size::px(80.0))),
        )
    }
}

#[test]
fn sidebar_section_opens_to_the_real_child_height() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let app = AppComponent::new(TallSectionCase);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    for _ in 0..5 {
        test.sync_and_update();
    }
    let content = test
        .find_many(|node, el| {
            el.effect()
                .is_some_and(|effect| effect.overflow == Overflow::Clip)
                .then(|| node.layout().area)
        })
        .pop()
        .expect("the section content container exists");
    assert!(
        (content.size.height - 80.0).abs() < 0.5,
        "the section must open to the child's 80 px, got {}",
        content.size.height
    );
}

struct ExpandedSectionCase;

impl App for ExpandedSectionCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Sidebar::new().child(SidebarSection::new("Favoritos").child(SidebarItem::new("Uno")))
    }
}

#[test]
fn sidebar_section_hides_chevron_without_hover_or_focus() {
    let app = AppComponent::new(ExpandedSectionCase);
    let (test, _) = TestingRunner::new(app, Size2D::new(300.0, 300.0), |_| {}, 1.0);
    let headers = test.find_many(|node, el| {
        if el.accessibility().builder.role() == Role::Button {
            Some(node)
        } else {
            None
        }
    });
    assert_eq!(headers.len(), 1, "there must be exactly one section header");
    assert_eq!(
        headers[0].children().len(),
        1,
        "expanded section header must contain only the label when not hovered or focused"
    );
}

fn dark_resolved() -> vow_appearance::Resolved {
    resolve(
        &AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            sidebar_style: SidebarStyle::Island,
            ..Default::default()
        },
        None,
    )
}

struct IslandShadowCase;

impl App for IslandShadowCase {
    fn render(&self) -> impl IntoElement {
        let resolved = dark_resolved();
        use_init_theme(|| theme::for_scheme(true));
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(color(resolved.surfaces.window))
            .child(SplitView::new(Sidebar::new(), rect()))
    }
}

struct ToolgroupShadowCase;

impl App for ToolgroupShadowCase {
    fn render(&self) -> impl IntoElement {
        let resolved = dark_resolved();
        use_init_theme(|| theme::for_scheme(true));
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(color(resolved.surfaces.window))
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .child(
                ToolGroup::new()
                    .child(Button::new().child("A"))
                    .child(Button::new().child("B")),
            )
    }
}

#[test]
fn island_and_toolgroup_cast_no_shadow() {
    let resolved = dark_resolved();
    let expected = resolved.surfaces.window.to_rgba8();

    let app = AppComponent::new(IslandShadowCase);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 300.0),
        |runner| runner.provide_root_context(|| State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let (w, _h, image) = decode(test.render().as_bytes());
    let island_bottom = 300.0 - SIDEBAR.inset as f32 + 4.0;
    assert_eq!(
        pixel(&image, w, 110, island_bottom as usize),
        expected,
        "pixel 4 px below the island must be the window background"
    );

    let app = AppComponent::new(ToolgroupShadowCase);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(300.0, 80.0),
        |runner| runner.provide_root_context(|| State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let (w, _h, image) = decode(test.render().as_bytes());
    let capsule_bottom = 40.0 + TOOLGROUP.height as f32 / 2.0 + 3.0;
    assert_eq!(
        pixel(&image, w, 150, capsule_bottom as usize),
        expected,
        "pixel 3 px below the toolgroup capsule must be the window background"
    );
}

struct AlignedToolbarCase;

impl App for AlignedToolbarCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let content = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(
                Titlebar::preview(
                    WindowStatus {
                        activated: true,
                        maximized: false,
                        fullscreen: false,
                        decoration: DecorationResponse::Client,
                        fixed: false,
                    },
                    "Ventana",
                    false,
                )
                .with_sidebar(true)
                .left(
                    ToolGroup::new()
                        .probe("toolbar")
                        .child(Button::new().child("A")),
                ),
            )
            .child(rect().width(Size::fill()).height(Size::flex(1.0)));
        SplitView::new(Sidebar::new().traffic_lights(true), content)
    }
}

#[test]
fn island_toolbar_is_centred_with_the_traffic_lights() {
    let resolved = resolve(
        &AppearanceConfig {
            sidebar_style: SidebarStyle::Island,
            ..Default::default()
        },
        None,
    );
    let app = AppComponent::new(AlignedToolbarCase);
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(900.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    let traffic = test
        .find(|node, el| {
            (el.accessibility().builder.label() == Some("sidebar-traffic-lights"))
                .then(|| node.layout().area)
        })
        .expect("the traffic lights exist");
    let toolbar = test
        .find(|node, el| {
            (el.accessibility().builder.author_id() == Some("toolbar")).then(|| node.layout().area)
        })
        .expect("the toolbar exists");
    let traffic_center = traffic.origin.y + traffic.size.height / 2.0;
    let toolbar_center = toolbar.origin.y + toolbar.size.height / 2.0;
    assert!(
        (traffic_center - toolbar_center).abs() <= 1.0,
        "the toolbar centre ({toolbar_center}) must match the traffic lights centre ({traffic_center})"
    );
}

struct HidingCase;

impl App for HidingCase {
    fn render(&self) -> impl IntoElement {
        SplitView::new(
            Sidebar::new()
                .traffic_lights(true)
                .child(SidebarItem::new("Uno")),
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .child(Titlebar::new().with_sidebar(true).left(SidebarToggle)),
        )
    }
}

#[test]
fn the_sidebar_toggle_hides_and_shows_the_sidebar() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(HidingCase),
        Size2D::new(900.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    let labelled = |test: &TestingRunner, label: &str| {
        test.find(|node, el| {
            (el.accessibility().builder.label() == Some(label)).then(|| node.layout().area)
        })
    };
    assert!(
        labelled(&test, "Uno").is_some(),
        "the sidebar starts visible"
    );
    let toggle = |test: &mut TestingRunner| {
        let area = labelled(test, "Mostrar u ocultar la barra lateral").expect("toggle");
        test.click_cursor((f64::from(area.center().x), f64::from(area.center().y)));
        test.poll(Duration::from_millis(16), Duration::from_millis(100));
    };
    toggle(&mut test);
    assert!(labelled(&test, "Uno").is_none(), "the sidebar hides");
    toggle(&mut test);
    assert!(labelled(&test, "Uno").is_some(), "the sidebar comes back");
}

#[test]
fn the_traffic_lights_stay_in_place_when_the_sidebar_hides() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(HidingCase),
        Size2D::new(900.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.sync_and_update();
    let lights = |test: &TestingRunner| {
        let (_, _, group_width, diameter) = vow_appearance::traffic::group_rect();
        test.find_many(|node, _| {
            let area = node.layout().area;
            ((area.size.width - group_width as f32).abs() < 0.5
                && (area.size.height - diameter as f32).abs() < 0.5)
                .then_some(area)
        })
    };
    let open = lights(&test);
    assert!(!open.is_empty(), "the sidebar shows the lights");
    let toggle = test
        .find(|node, el| {
            (el.accessibility().builder.label() == Some("Mostrar u ocultar la barra lateral"))
                .then(|| node.layout().area)
        })
        .expect("toggle");
    test.click_cursor((f64::from(toggle.center().x), f64::from(toggle.center().y)));
    test.poll(Duration::from_millis(16), Duration::from_millis(100));
    let hidden = lights(&test);
    assert!(!hidden.is_empty(), "the titlebar shows the lights");
    assert!(
        hidden
            .iter()
            .all(|area| (area.origin.x - open[0].origin.x).abs() < 0.5
                && (area.origin.y - open[0].origin.y).abs() < 0.5),
        "the lights moved from {:?} to {hidden:?}",
        open[0].origin
    );
}

struct NarrowCase(NarrowSidebar);

impl App for NarrowCase {
    fn render(&self) -> impl IntoElement {
        SplitView::new(
            Sidebar::new()
                .traffic_lights(true)
                .child(SidebarItem::new("Uno").on_select(|_| {})),
            rect()
                .probe("detail")
                .width(Size::fill())
                .height(Size::fill())
                .child(Titlebar::new().with_sidebar(true).left(SidebarToggle)),
        )
        .narrow_sidebar(self.0)
    }
}

fn narrow_runner(mode: NarrowSidebar) -> TestingRunner {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(NarrowCase(mode)),
        Size2D::new(600.0, 400.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.poll(Duration::from_millis(16), Duration::from_millis(100));
    let area = labelled(&test, "Mostrar u ocultar la barra lateral").expect("toggle");
    test.click_cursor((f64::from(area.center().x), f64::from(area.center().y)));
    test.poll(Duration::from_millis(16), Duration::from_millis(100));
    test
}

fn labelled(test: &TestingRunner, label: &str) -> Option<torin::prelude::Area> {
    test.find(|node, el| {
        (el.accessibility().builder.label() == Some(label)).then(|| node.layout().area)
    })
}

fn detail_x(test: &TestingRunner) -> f32 {
    test.find(|node, el| {
        (el.accessibility().builder.author_id() == Some("detail"))
            .then(|| node.layout().area.origin.x)
    })
    .expect("the detail exists")
}

#[test]
fn an_overlaid_sidebar_covers_the_detail_and_closes_on_escape_or_on_select() {
    let mut test = narrow_runner(NarrowSidebar::Overlay);
    assert!(
        labelled(&test, "Uno").is_some(),
        "the toggle opens the sidebar"
    );
    assert_eq!(detail_x(&test), 0.0, "the detail does not move");
    test.press_key(Key::Named(NamedKey::Escape));
    test.poll(Duration::from_millis(16), Duration::from_millis(100));
    assert!(labelled(&test, "Uno").is_none(), "Escape closes it");

    let mut test = narrow_runner(NarrowSidebar::Overlay);
    let item = labelled(&test, "Uno").expect("item");
    test.click_cursor((f64::from(item.center().x), f64::from(item.center().y)));
    test.poll(Duration::from_millis(16), Duration::from_millis(100));
    assert!(
        labelled(&test, "Uno").is_none(),
        "choosing an item closes it"
    );
}

#[test]
fn a_pushed_sidebar_docks_and_narrows_the_detail() {
    let test = narrow_runner(NarrowSidebar::Push);
    assert!(
        labelled(&test, "Uno").is_some(),
        "the toggle opens the sidebar"
    );
    let docked = SIDEBAR.width as f32 + 2.0 * SIDEBAR.inset as f32;
    assert!(
        (detail_x(&test) - docked).abs() < 0.5,
        "the detail starts after the sidebar"
    );
}
