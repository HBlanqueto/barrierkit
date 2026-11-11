use sctk::reexports::{
    calloop::{self, EventLoop},
    client::backend::ObjectId,
};
use sctk::seat::keyboard::{KeyEvent, Keysym};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

pub(super) const REVEAL_STEP: Duration = Duration::from_millis(16);
pub(super) const REVEAL_TIME: Duration = Duration::from_millis(ctdesign::DURATION_BASE + 100);

use super::{
    input::seat::{RepeatState, released_repeated_key, should_repeat_key, stop_repeat},
    surface::buffer_size,
    timer::{Timers, Wake, WakeQueue},
    window::{
        WindowControls, WindowEdge, WindowStatus, clamp_toplevel_size_dimension, resize_edge_at,
    },
};

pub(super) fn window_controls(status: WindowStatus) -> WindowControls {
    let (ping, _source) = calloop::ping::make_ping().unwrap();
    WindowControls {
        primary: super::clipboard::Clipboard::new(ping.clone()),
        clipboard: super::clipboard::Clipboard::new(ping.clone()),
        queue: Rc::new(RefCell::new(Vec::new())),
        ping,
        pointer_serial: Rc::new(std::cell::Cell::new(0)),
        status: freya_core::prelude::State::create(status),
        title: "Ventana".to_string(),
        fixed: false,
        surface_id: ObjectId::null(),
        focus_bar: freya_core::prelude::State::create(false),
        focus_bar_progress: freya_core::prelude::State::create(0.0),
        focus_overlay: false,
        focus_bar_height: freya_core::prelude::State::create(28.0),
        desktop: freya_core::prelude::State::create(None),
        fullscreen_bar: freya_core::prelude::State::create(super::FullscreenBar::Solid),
        fullscreen_push: freya_core::prelude::State::create(false),
        overlay_titlebar_height: freya_core::prelude::State::create(0.0),
        hud: false,
        panel: false,
        ime_preedit: freya_core::prelude::State::create(String::new()),
    }
}

#[test]
fn the_resize_band_picks_a_corner_before_an_edge() {
    let size = (800, 600);
    assert_eq!(resize_edge_at((1.0, 1.0), size), Some(WindowEdge::TopLeft));
    assert_eq!(resize_edge_at((12.0, 2.0), size), Some(WindowEdge::TopLeft));
    assert_eq!(resize_edge_at((400.0, 2.0), size), Some(WindowEdge::Top));
    assert_eq!(
        resize_edge_at((799.0, 599.0), size),
        Some(WindowEdge::BottomRight)
    );
    assert_eq!(
        resize_edge_at((799.0, 300.0), size),
        Some(WindowEdge::Right)
    );
    assert_eq!(resize_edge_at((400.0, 300.0), size), None);
}

#[test]
fn wakeup_runs_its_callback_on_the_loop_thread() {
    let (ping, source) = calloop::ping::make_ping().unwrap();
    let callbacks: WakeQueue = Rc::new(RefCell::new(Vec::new()));
    let mut event_loop = EventLoop::<()>::try_new().unwrap();
    let queue = callbacks.clone();
    event_loop
        .handle()
        .insert_source(source, move |_, _, ()| {
            for callback in queue.borrow().clone() {
                callback();
            }
        })
        .unwrap();
    let timers = Timers {
        queue: Rc::new(RefCell::new(Vec::new())),
        ping: ping.clone(),
        wake: Wake { ping, callbacks },
    };
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = seen.clone();
    let handle = timers.wakeup(move || {
        counter.fetch_add(1, Ordering::SeqCst);
    });
    let thread_handle = handle.clone();
    let writer = std::thread::spawn(move || thread_handle.wake());
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while seen.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        event_loop
            .dispatch(Duration::from_millis(10), &mut ())
            .unwrap();
    }
    writer.join().unwrap();
    assert_eq!(seen.load(Ordering::SeqCst), 1);
}

#[test]
fn toplevel_size_dimension_clamps_to_unbounded() {
    assert_eq!(clamp_toplevel_size_dimension(u32::MAX), 0);
    assert_eq!(clamp_toplevel_size_dimension(i32::MAX as u32 + 1), 0);
    assert_eq!(
        clamp_toplevel_size_dimension(i32::MAX as u32),
        i32::MAX as u32
    );
    assert_eq!(clamp_toplevel_size_dimension(760), 760);
    assert_eq!(clamp_toplevel_size_dimension(0), 0);
}

#[test]
fn buffer_size_rounds_logical_size_at_fractional_scale() {
    assert_eq!(buffer_size((801, 601), 1.25), (1001, 751));
}

#[test]
fn losing_focus_cancels_repeat_and_invalidates_old_timers() {
    let event = KeyEvent {
        time: 0,
        raw_code: 0,
        keysym: Keysym::new(0),
        utf8: None,
    };
    let mut repeat = Some(RepeatState {
        event,
        delay: Duration::from_millis(200),
        rate: Duration::from_millis(40),
        first: false,
        pending: true,
        generation: 7,
    });
    let mut generation = 7;
    stop_repeat(&mut repeat, &mut generation);
    assert!(repeat.is_none());
    assert_eq!(generation, 8);
}

#[test]
fn modifier_keys_do_not_start_repeat() {
    let shift = KeyEvent {
        time: 0,
        raw_code: 50,
        keysym: Keysym::Shift_L,
        utf8: None,
    };
    assert!(!should_repeat_key(&shift));
}

#[test]
fn regular_keys_start_repeat() {
    let a = KeyEvent {
        time: 0,
        raw_code: 38,
        keysym: Keysym::a,
        utf8: Some("a".into()),
    };
    assert!(should_repeat_key(&a));
}

#[test]
fn releasing_repeated_key_stops_repeat_after_shift_is_released() {
    let a = KeyEvent {
        time: 0,
        raw_code: 38,
        keysym: Keysym::a,
        utf8: Some("a".into()),
    };
    let shifted_a = KeyEvent {
        keysym: Keysym::A,
        utf8: Some("A".into()),
        ..a.clone()
    };
    let repeat = Some(RepeatState {
        event: shifted_a,
        delay: Duration::from_millis(200),
        rate: Duration::from_millis(40),
        first: true,
        pending: false,
        generation: 1,
    });
    assert!(released_repeated_key(&repeat, &a));
}

#[test]
fn releasing_a_different_key_keeps_repeat_running() {
    let a = KeyEvent {
        time: 0,
        raw_code: 38,
        keysym: Keysym::a,
        utf8: Some("a".into()),
    };
    let b = KeyEvent {
        time: 0,
        raw_code: 56,
        keysym: Keysym::b,
        utf8: Some("b".into()),
    };
    let repeat = Some(RepeatState {
        event: b,
        delay: Duration::from_millis(200),
        rate: Duration::from_millis(40),
        first: true,
        pending: false,
        generation: 1,
    });
    assert!(!released_repeated_key(&repeat, &a));
}

#[test]
fn pressing_b_then_releasing_a_keeps_b_repeating() {
    let a = KeyEvent {
        time: 0,
        raw_code: 38,
        keysym: Keysym::a,
        utf8: Some("a".into()),
    };
    let b = KeyEvent {
        time: 0,
        raw_code: 56,
        keysym: Keysym::b,
        utf8: Some("b".into()),
    };

    let mut repeat: Option<RepeatState> = None;
    if should_repeat_key(&a) {
        repeat = Some(RepeatState {
            event: a.clone(),
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 1,
        });
    }

    if should_repeat_key(&b) {
        repeat = Some(RepeatState {
            event: b.clone(),
            delay: Duration::from_millis(200),
            rate: Duration::from_millis(40),
            first: true,
            pending: false,
            generation: 2,
        });
    }

    assert!(!released_repeated_key(&repeat, &a));
    assert!(
        repeat
            .as_ref()
            .is_some_and(|state| state.event.keysym == b.keysym)
    );
}

#[cfg(test)]
mod drag_band_tests {
    use crate::ProbeExt;
    use crate::sidebar::{Sidebar, SidebarItem};
    use crate::views::split::SplitView;
    use ctappearance::{AppearanceConfig, SidebarStyle, resolve};
    use freya_core::integration::AppComponent;
    use freya_core::prelude::{App, ContainerSizeExt, IntoElement, State, rect};
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::Size2D;

    use crate::platform::window::{WindowCommand, WindowControls, WindowStatus};

    struct DragCase;

    impl App for DragCase {
        fn render(&self) -> impl IntoElement {
            SplitView::new(
                Sidebar::new()
                    .traffic_lights(true)
                    .child(SidebarItem::new("Uno").probe("island-first-item")),
                rect().expanded(),
            )
        }
    }

    fn first_item_top(fullscreen: bool) -> f32 {
        let resolved = resolve(&AppearanceConfig::default(), None);
        let app = AppComponent::new(DragCase);
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(800.0, 400.0),
            move |runner| {
                runner.provide_root_context(move || {
                    super::window_controls(WindowStatus {
                        fullscreen,
                        ..WindowStatus::default()
                    })
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        test.find(|node, el| {
            (el.accessibility().builder.author_id() == Some("island-first-item"))
                .then(|| node.layout().area.origin.y)
        })
        .expect("first sidebar item")
    }

    #[test]
    fn without_traffic_lights_in_full_screen_the_sidebar_content_moves_up() {
        let windowed = first_item_top(false);
        let fullscreen = first_item_top(true);
        let lights_band = (ctdesign::SIDEBAR.traffic_y
            + ctdesign::TITLEBAR.buttons.diameter
            + ctdesign::SETTINGS.sidebar_lights_gap
            - ctdesign::SETTINGS.sidebar_padding_top) as f32;
        assert_eq!(windowed - fullscreen, lights_band);
    }

    #[test]
    fn a_mouse_down_on_the_island_top_band_moves_the_window() {
        let queue: Rc<RefCell<Vec<WindowCommand>>> = Rc::new(RefCell::new(Vec::new()));
        let controls_queue = queue.clone();
        let resolved = resolve(
            &AppearanceConfig {
                sidebar_style: SidebarStyle::Island,
                ..Default::default()
            },
            None,
        );
        let app = AppComponent::new(DragCase);
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(800.0, 400.0),
            move |runner| {
                runner.provide_root_context(move || WindowControls {
                    queue: controls_queue.clone(),
                    ..super::window_controls(WindowStatus::default())
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        test.press_cursor((100.0, 4.0));
        assert!(
            queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the island top band must request a window move"
        );

        queue.borrow_mut().clear();
        test.press_cursor((30.0, 30.0));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on a traffic light must not move the window"
        );

        queue.borrow_mut().clear();
        let (x, y) = test
            .find(|node, el| {
                (el.accessibility().builder.author_id() == Some("island-first-item")).then(|| {
                    let area = node.layout().area;
                    (
                        area.origin.x + area.size.width / 2.0,
                        area.origin.y + area.size.height / 2.0,
                    )
                })
            })
            .expect("the first island row exists");
        test.press_cursor((f64::from(x), f64::from(y)));
        assert!(
            !queue
                .borrow()
                .iter()
                .any(|command| matches!(command, WindowCommand::Move(_))),
            "a press on the first island row must not move the window"
        );
    }
}

#[cfg(test)]
mod overlay_titlebar_tests {
    use crate::ProbeExt;
    use crate::titlebar::Titlebar;
    use ctappearance::{AppearanceConfig, resolve};
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Direction, Size2D};

    use crate::platform::window::{WindowControls, WindowStatus};

    struct OverlayCase;

    impl App for OverlayCase {
        fn render(&self) -> impl IntoElement {
            crate::chrome::use_focus_bar_reveal();
            rect()
                .expanded()
                .direction(Direction::Vertical)
                .child(Titlebar::new().probe("overlay-title"))
                .child(
                    rect()
                        .expanded()
                        .a11y_builder(|node| node.set_author_id("overlay-content")),
                )
        }
    }

    fn top_of(test: &TestingRunner, id: &str) -> Option<f32> {
        test.find(|node, el| {
            (el.accessibility().builder.author_id() == Some(id))
                .then(|| node.layout().area.origin.y)
        })
    }

    fn runner(
        fullscreen: bool,
        overlay: bool,
        bar: Rc<RefCell<Option<State<bool>>>>,
    ) -> TestingRunner {
        let resolved = resolve(&AppearanceConfig::default(), None);
        let (test, _) = TestingRunner::new(
            AppComponent::new(OverlayCase),
            Size2D::new(400.0, 300.0),
            move |runner| {
                let bar = bar.clone();
                runner.provide_root_context(move || {
                    let controls = WindowControls {
                        focus_overlay: overlay,
                        ..super::window_controls(WindowStatus {
                            fullscreen,
                            ..WindowStatus::default()
                        })
                    };
                    *bar.borrow_mut() = Some(controls.focus_bar);
                    controls
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test
    }

    #[test]
    fn the_titlebar_goes_over_the_content_below_the_bar() {
        let bar = Rc::new(RefCell::new(None));
        let mut test = runner(true, true, bar.clone());
        test.sync_and_update();
        assert_eq!(top_of(&test, "overlay-content"), Some(0.0));
        assert_eq!(
            top_of(&test, "overlay-title"),
            None,
            "hidden without the bar"
        );

        bar.borrow_mut().as_mut().unwrap().set(true);
        test.sync_and_update();
        test.poll(super::REVEAL_STEP, super::REVEAL_TIME);
        assert_eq!(
            top_of(&test, "overlay-content"),
            Some(0.0),
            "the content does not move"
        );
        let title = top_of(&test, "overlay-title").expect("shown with the bar");
        assert!(title >= 28.0, "below the bar, got {title}");

        bar.borrow_mut().as_mut().unwrap().set(false);
        test.sync_and_update();
        assert!(
            top_of(&test, "overlay-title").is_some(),
            "slides up instead of vanishing"
        );
        test.poll(super::REVEAL_STEP, super::REVEAL_TIME);
        assert_eq!(top_of(&test, "overlay-title"), None);
    }

    #[test]
    fn without_overlay_the_titlebar_stays_in_the_layout() {
        let bar = Rc::new(RefCell::new(None));
        let mut test = runner(true, false, bar);
        test.sync_and_update();
        let content = top_of(&test, "overlay-content").expect("content");
        assert!(content > 0.0, "below the titlebar, got {content}");
    }

    #[test]
    fn outside_fullscreen_the_overlay_changes_nothing() {
        let bar = Rc::new(RefCell::new(None));
        let mut test = runner(false, true, bar);
        test.sync_and_update();
        let content = top_of(&test, "overlay-content").expect("content");
        assert!(content > 0.0, "below the titlebar, got {content}");
        assert!(top_of(&test, "overlay-title").is_some());
    }
}

#[cfg(test)]
mod traffic_light_tests {
    use crate::backdrop::{Backdrop, BackdropSource};
    use crate::sidebar::Sidebar;
    use crate::titlebar::Titlebar;
    use crate::views::split::SplitView;
    use ctappearance::{AppearanceConfig, resolve};
    use ctdesign::{Rgba, TITLEBAR};
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Size, Size2D};

    use crate::platform::window::{WindowCommand, WindowControls, WindowStatus};

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Layout {
        Titlebar,
        TintedTitlebar,
        Sidebar,
    }

    struct Case(Layout);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            let content = rect().expanded();
            match self.0 {
                Layout::Titlebar => rect()
                    .width(Size::fill())
                    .child(Titlebar::new())
                    .into_element(),
                Layout::TintedTitlebar => Backdrop::new(
                    BackdropSource::Colors([Rgba::rgb8(90, 40, 80); 3]),
                    rect().width(Size::fill()).child(Titlebar::new()),
                )
                .into_element(),
                Layout::Sidebar => {
                    SplitView::new(Sidebar::new().traffic_lights(true), content).into_element()
                }
            }
        }
    }

    fn press(layout: Layout, light: usize) -> Vec<WindowCommand> {
        let queue: Rc<RefCell<Vec<WindowCommand>>> = Rc::new(RefCell::new(Vec::new()));
        let commands = queue.clone();
        let resolved = resolve(&AppearanceConfig::default(), None);
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Case(layout)),
            Size2D::new(900.0, 300.0),
            move |runner| {
                let queue = commands.clone();
                runner.provide_root_context(move || WindowControls {
                    queue,
                    ..super::window_controls(WindowStatus::default())
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        let diameter = TITLEBAR.buttons.diameter as f32;
        let gap = TITLEBAR.buttons.spacing as f32;
        let width = 3.0 * diameter + 2.0 * gap;
        let group = test
            .find(|node, _| {
                let area = node.layout().area;
                ((area.width() - width).abs() < 0.5 && (area.height() - diameter).abs() < 0.5)
                    .then_some(area)
            })
            .expect("traffic lights");
        let left = group.min_x();
        let point = (
            f64::from(left + light as f32 * (diameter + gap) + diameter / 2.0),
            f64::from(group.center().y),
        );
        test.move_cursor(point);
        test.sync_and_update();
        test.click_cursor(point);
        test.sync_and_update();
        queue.take()
    }

    #[test]
    fn the_minimise_light_minimises_in_every_window_layout() {
        for layout in [Layout::Titlebar, Layout::TintedTitlebar, Layout::Sidebar] {
            let commands = press(layout, 1);
            assert!(
                commands
                    .iter()
                    .any(|c| matches!(c, WindowCommand::Minimize)),
                "no minimise command in {layout:?}"
            );
        }
    }

    #[test]
    fn without_an_on_close_the_close_light_closes_the_window() {
        for layout in [Layout::Titlebar, Layout::TintedTitlebar, Layout::Sidebar] {
            let commands = press(layout, 0);
            assert!(
                commands.iter().any(|c| matches!(c, WindowCommand::Close)),
                "no close command in {layout:?}"
            );
        }
    }
}

#[cfg(test)]
mod inactive_accent_tests {
    use crate::style::Palette;
    use ctappearance::{AppearanceConfig, resolve};
    use ctdesign::Rgba;
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::Cell, rc::Rc};
    use torin::prelude::Size2D;

    use crate::platform::window::WindowStatus;

    struct Probe(Rc<Cell<Option<Rgba>>>, fn() -> Rgba);

    impl App for Probe {
        fn render(&self) -> impl IntoElement {
            self.0.set(Some((self.1)()));
            rect()
        }
    }

    fn painted(activated: bool, paint: fn() -> Rgba) -> Rgba {
        let seen = Rc::new(Cell::new(None));
        let resolved = resolve(&AppearanceConfig::default(), None);
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Probe(seen.clone(), paint)),
            Size2D::new(200.0, 100.0),
            move |runner| {
                runner.provide_root_context(move || {
                    super::window_controls(WindowStatus {
                        activated,
                        ..WindowStatus::default()
                    })
                });
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.sync_and_update();
        seen.get().expect("rendered")
    }

    #[test]
    fn an_unfocused_window_paints_the_accent_grey() {
        let palette = ctappearance::palette(resolve(&AppearanceConfig::default(), None).scheme);
        let accent = || Palette::current().accent;
        assert_eq!(painted(false, accent), palette.accent_inactive);
        assert_ne!(painted(true, accent), palette.accent_inactive);
    }
}

#[cfg(test)]
mod fullscreen_chrome_tests {
    use crate::chrome::{RailStyle, TitlebarLayout, WindowChrome};
    use crate::sidebar::Sidebar;
    use crate::styles::WindowChromeStyle;
    use crate::titlebar::Titlebar;
    use crate::views::split::SplitView;
    use ctappearance::{AppearanceConfig, resolve};
    use ctdesign::TITLEBAR;
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::Size2D;

    use crate::platform::window::{WindowControls, WindowStatus};

    struct Case(TitlebarLayout, RailStyle);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            let Self(layout, rail_style) = *self;
            let detail = rect()
                .expanded()
                .child(Titlebar::new().title("Ventana").with_sidebar(true));
            crate::chrome::window_root(
                crate::styles::Theme::tengen(),
                WindowChrome::new(SplitView::new(Sidebar::new().traffic_lights(true), detail))
                    .style(move |s| WindowChromeStyle {
                        layout,
                        rail_style,
                        ..s
                    }),
            )
        }
    }

    fn light_tops(test: &TestingRunner) -> Vec<(i32, i32)> {
        let diameter = TITLEBAR.buttons.diameter as f32;
        let length = 3.0 * diameter + 2.0 * TITLEBAR.buttons.spacing as f32;
        let mut groups = test.find_many(|node, _| {
            let area = node.layout().area;
            let size = (area.width(), area.height());
            (size == (length, diameter) || size == (diameter, length))
                .then_some((area.min_x() as i32, area.min_y() as i32))
        });
        groups.sort_unstable();
        groups.dedup();
        groups
    }

    #[test]
    fn in_full_screen_every_layout_shows_its_lights_once_below_the_bar_only_while_it_is_down() {
        for (layout, rail_style) in [
            (TitlebarLayout::Top, RailStyle::Normal),
            (TitlebarLayout::Rail, RailStyle::Normal),
            (TitlebarLayout::Rail, RailStyle::Unified),
        ] {
            {
                let bar = Rc::new(RefCell::new(None));
                let shared = bar.clone();
                let resolved = resolve(&AppearanceConfig::default(), None);
                let (mut test, _) = TestingRunner::new(
                    AppComponent::new(Case(layout, rail_style)),
                    Size2D::new(900.0, 500.0),
                    move |runner| {
                        let shared = shared.clone();
                        runner.provide_root_context(move || {
                            let controls = WindowControls {
                                focus_overlay: true,
                                ..super::window_controls(WindowStatus {
                                    fullscreen: true,
                                    ..WindowStatus::default()
                                })
                            };
                            *shared.borrow_mut() = Some(controls.focus_bar);
                            controls
                        });
                        runner.provide_root_context(move || State::create(resolved));
                    },
                    1.0,
                );
                test.sync_and_update();
                assert_eq!(
                    light_tops(&test),
                    vec![],
                    "{layout:?} {rail_style:?} at rest"
                );
                bar.borrow_mut().as_mut().unwrap().set(true);
                test.sync_and_update();
                test.poll(super::REVEAL_STEP, super::REVEAL_TIME);
                let lights = light_tops(&test);
                assert_eq!(
                    lights.len(),
                    1,
                    "{layout:?} {rail_style:?} with the bar: {lights:?}"
                );
                assert!(
                    lights[0].1 >= 28,
                    "{layout:?} {rail_style:?} under the bar: {lights:?}"
                );
            }
        }
    }

    struct Reveal(crate::styles::Theme);

    impl App for Reveal {
        fn render(&self) -> impl IntoElement {
            crate::chrome::window_root(self.0.clone(), rect())
        }
    }

    #[test]
    fn the_window_theme_sets_the_animation_of_the_full_screen_bar() {
        use crate::styles::{Animation, FullscreenBarStyle, Theme};
        let instant = Theme::tengen().style(|s: FullscreenBarStyle| FullscreenBarStyle {
            animation: Animation::None,
            ..s
        });
        for (theme, at_once) in [(Theme::tengen(), false), (instant, true)] {
            let shared = Rc::new(RefCell::new(None));
            let states = shared.clone();
            let resolved = resolve(&AppearanceConfig::default(), None);
            let (mut test, _) = TestingRunner::new(
                AppComponent::new(Reveal(theme)),
                Size2D::new(100.0, 100.0),
                move |runner| {
                    let states = states.clone();
                    runner.provide_root_context(move || {
                        let controls = super::window_controls(WindowStatus {
                            fullscreen: true,
                            ..WindowStatus::default()
                        });
                        *states.borrow_mut() =
                            Some((controls.focus_bar, controls.focus_bar_progress));
                        controls
                    });
                    runner.provide_root_context(move || State::create(resolved));
                },
                1.0,
            );
            test.sync_and_update();
            let (mut bar, progress) = shared.borrow().unwrap();
            bar.set(true);
            test.sync_and_update();
            assert_eq!(*progress.peek() == 1.0, at_once);
        }
    }
}
