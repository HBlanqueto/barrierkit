use freya_core::{integration::UserEvent, prelude::CursorIcon};
use sctk::reexports::{
    calloop::{self, EventLoop},
    client::backend::ObjectId,
    protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape as CursorShape,
};
use sctk::seat::keyboard::{KeyEvent, Keysym};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use torin::prelude::CursorPoint;
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;

const REVEAL_STEP: Duration = Duration::from_millis(16);
const REVEAL_TIME: Duration = Duration::from_millis(vow_design::DURATION_BASE + 100);

use super::{
    input::seat::{
        RepeatState, cursor_shape_from_icon, dispatch_touch_cancels, find_index_by_id,
        released_repeated_key, should_repeat_key, stop_repeat,
    },
    runtime::{drain_user_events, make_sender},
    surface::{
        buffer_size, destroy_fractional_objects, fractional_scale_enabled, physical_point,
        scale_from_wire,
    },
    timer::{Timers, Wake, WakeQueue},
    window::{
        WindowControls, WindowEdge, WindowStatus, clamp_toplevel_size_dimension, resize_edge_at,
    },
};

fn window_controls(status: WindowStatus) -> WindowControls {
    let (ping, _source) = calloop::ping::make_ping().unwrap();
    WindowControls {
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
fn scale_from_wire_converts_fractional_scale_values() {
    assert!((scale_from_wire(120) - 1.0).abs() < f64::EPSILON);
    assert!((scale_from_wire(150) - 1.25).abs() < f64::EPSILON);
    assert!((scale_from_wire(180) - 1.5).abs() < f64::EPSILON);
    assert!((scale_from_wire(240) - 2.0).abs() < f64::EPSILON);
}

#[test]
fn buffer_size_rounds_logical_size_at_fractional_scale() {
    assert_eq!(buffer_size((801, 601), 1.25), (1001, 751));
}

#[test]
fn fractional_scale_mode_needs_both_globals() {
    assert!(fractional_scale_enabled(true, true, 1.25));
    assert!(!fractional_scale_enabled(true, false, 1.25));
    assert!(!fractional_scale_enabled(false, true, 1.25));
    assert!(!fractional_scale_enabled(false, false, 1.25));
}

#[test]
fn pointer_and_touch_coordinates_become_physical() {
    assert_eq!(
        physical_point((10.0, 20.0), 1.5),
        CursorPoint::from((15.0, 30.0))
    );
    assert_eq!(
        physical_point((3.0, 4.0), 2.0),
        CursorPoint::from((6.0, 8.0))
    );
    assert_eq!(
        physical_point((10.0, 20.0), 1.25),
        CursorPoint::from((12.5, 25.0))
    );
}

#[test]
fn fractional_scale_hit_test_respects_one_pixel_border() {
    let scale = 1.25;
    let border_physical = 1.0 * scale;
    let inside = physical_point((0.5, 0.5), scale);
    let outside = physical_point((1.5, 1.5), scale);
    assert!(inside.x < border_physical);
    assert!(inside.y < border_physical);
    assert!(outside.x >= border_physical);
    assert!(outside.y >= border_physical);
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
fn releasing_repeated_key_stops_repeat() {
    let a = KeyEvent {
        time: 0,
        raw_code: 38,
        keysym: Keysym::a,
        utf8: Some("a".into()),
    };
    let repeat = Some(RepeatState {
        event: a.clone(),
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

#[test]
fn sender_tags_user_events_with_surface_id() {
    let (ping, _) = calloop::ping::make_ping().unwrap();
    let queue: Rc<RefCell<Vec<(ObjectId, UserEvent)>>> = Rc::new(RefCell::new(Vec::new()));
    let owner = ObjectId::null();
    let sender = make_sender(queue.clone(), ping, owner.clone());
    sender(UserEvent::RequestRedraw);
    let events = drain_user_events(&queue);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, owner);
}

#[test]
fn cursor_icon_maps_to_wayland_shape() {
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::Default),
        CursorShape::Default
    );
    assert_eq!(cursor_shape_from_icon(CursorIcon::Text), CursorShape::Text);
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::Pointer),
        CursorShape::Pointer
    );
    assert_eq!(cursor_shape_from_icon(CursorIcon::Grab), CursorShape::Grab);
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::Grabbing),
        CursorShape::Grabbing
    );
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::NotAllowed),
        CursorShape::NotAllowed
    );
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::NResize),
        CursorShape::NResize
    );
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::EResize),
        CursorShape::EResize
    );
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::EwResize),
        CursorShape::EwResize
    );
    assert_eq!(
        cursor_shape_from_icon(CursorIcon::NsResize),
        CursorShape::NsResize
    );
}

#[test]
fn touch_cancel_dispatches_cancelled_for_every_active_contact() {
    let mut touch_surfaces: std::collections::HashMap<i32, ObjectId> =
        std::collections::HashMap::new();
    let surface_a = ObjectId::null();
    let surface_b = ObjectId::null();
    touch_surfaces.insert(0, surface_a.clone());
    touch_surfaces.insert(1, surface_a.clone());
    touch_surfaces.insert(2, surface_b.clone());

    let mut cancelled = Vec::new();
    dispatch_touch_cancels(&mut touch_surfaces, |id, surface_id| {
        cancelled.push((id, surface_id));
    });

    assert!(
        touch_surfaces.is_empty(),
        "touch_surfaces must be empty after cancel"
    );
    assert_eq!(cancelled.len(), 3);
    assert!(cancelled.contains(&(0, surface_a.clone())));
    assert!(cancelled.contains(&(1, surface_a.clone())));
    assert!(cancelled.contains(&(2, surface_b.clone())));
}

#[test]
fn touch_routing_uses_the_stored_surface_id() {
    let surfaces = vec![('B', 1usize), ('A', 2usize)];
    let touch_id = 7;
    let stored_surface_id = 2usize;

    let mut touch_surfaces: std::collections::HashMap<i32, usize> =
        std::collections::HashMap::new();
    touch_surfaces.insert(touch_id, stored_surface_id);

    let routed_id = touch_surfaces.get(&touch_id).copied();
    let index = find_index_by_id(&surfaces, routed_id.unwrap(), |item| item.1);

    assert_eq!(routed_id, Some(stored_surface_id));
    assert_eq!(
        index,
        Some(1),
        "motion/up for the same touch id must reach surface A"
    );
}

#[test]
fn destroy_fractional_objects_is_safe_when_missing() {
    let mut fractional: Option<WpFractionalScaleV1> = None;
    let mut viewport: Option<WpViewport> = None;
    destroy_fractional_objects(&mut fractional, &mut viewport);
    assert!(fractional.is_none());
    assert!(viewport.is_none());
}

#[cfg(feature = "shell")]
#[test]
fn extension_slot_typed_access() {
    use freya_core::integration::Runner;
    use sctk::reexports::client::{
        QueueHandle,
        protocol::{wl_output::WlOutput, wl_seat::WlSeat},
    };

    use super::{
        extension::{Extension, ExtensionHooks},
        runtime::Runtime,
    };

    fn no_op_runner(_: &Runtime, _: &mut Runner) {}
    fn no_op_seat(_: &mut Runtime, _: &WlSeat) {}
    fn no_op_output(_: &mut Runtime, _: &QueueHandle<Runtime>, _: &WlOutput) {}
    fn no_op_poll(_: &mut Runtime) {}

    let mut ext = Extension {
        state: Box::new(42i32),
        hooks: ExtensionHooks {
            provide_contexts: no_op_runner,
            new_seat: no_op_seat,
            new_output: no_op_output,
            poll: no_op_poll,
        },
    };

    assert_eq!(ext.state::<i32>(), Some(&42));
    assert_eq!(ext.state::<String>(), None);
    *ext.state_mut::<i32>().expect("i32 state") = 7;
    assert_eq!(ext.state::<i32>(), Some(&7));
    assert_eq!(ext.state_mut::<String>(), None);
}

#[cfg(feature = "shell")]
#[test]
fn extension_role_parent_popup_default_is_false() {
    use std::{any::Any, os::unix::net::UnixStream};

    use sctk::reexports::{
        client::{Proxy, backend::Backend},
        protocols::xdg::shell::client::xdg_popup::XdgPopup,
    };

    use super::surface::{ExtensionRole, SurfaceRole};

    #[derive(Debug)]
    struct DummyRole;

    impl ExtensionRole for DummyRole {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    let (client_sock, _server_sock) = UnixStream::pair().unwrap();
    let backend = Backend::connect(client_sock).unwrap();
    let popup = XdgPopup::inert(backend.downgrade());

    let role = SurfaceRole::Extension(Box::new(DummyRole));
    assert!(!role.parent_popup(&popup));
}

#[cfg(test)]
mod drag_band_tests {
    use crate::ProbeExt;
    use crate::sidebar::{Sidebar, SidebarItem};
    use crate::views::split::SplitView;
    use freya_core::integration::AppComponent;
    use freya_core::prelude::{App, ContainerSizeExt, IntoElement, State, rect};
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Size, Size2D};
    use vow_appearance::{AppearanceConfig, SidebarStyle, resolve};

    use crate::platform::window::{WindowCommand, WindowControls, WindowStatus};

    struct DragCase;

    impl App for DragCase {
        fn render(&self) -> impl IntoElement {
            SplitView::new(
                Sidebar::new()
                    .traffic_lights(true)
                    .child(SidebarItem::new("Uno").probe("island-first-item")),
                rect().width(Size::fill()).height(Size::fill()),
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
        let lights_band = (vow_design::SIDEBAR.traffic_y
            + vow_design::TITLEBAR.buttons.diameter
            + vow_design::SETTINGS.sidebar_lights_gap
            - vow_design::SETTINGS.sidebar_padding_top) as f32;
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
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Direction, Size, Size2D};
    use vow_appearance::{AppearanceConfig, resolve};

    use crate::platform::window::{WindowControls, WindowStatus};

    struct OverlayCase;

    impl App for OverlayCase {
        fn render(&self) -> impl IntoElement {
            crate::chrome::use_focus_bar_reveal();
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .direction(Direction::Vertical)
                .child(Titlebar::new().probe("overlay-title"))
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
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
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Size, Size2D};
    use vow_appearance::{AppearanceConfig, resolve};
    use vow_design::{Rgba, TITLEBAR};

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
            let content = rect().width(Size::fill()).height(Size::fill());
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
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::Cell, rc::Rc};
    use torin::prelude::Size2D;
    use vow_appearance::{AppearanceConfig, resolve};
    use vow_design::Rgba;

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
        let palette = vow_appearance::palette(resolve(&AppearanceConfig::default(), None).scheme);
        let accent = || Palette::current().accent;
        assert_eq!(painted(false, accent), palette.accent_inactive);
        assert_ne!(painted(true, accent), palette.accent_inactive);
    }

    #[test]
    fn an_own_accent_turns_grey_with_the_kit_accent() {
        let own = || crate::style::focus_tint(Rgba::rgb8(200, 30, 90));
        let palette = vow_appearance::palette(resolve(&AppearanceConfig::default(), None).scheme);
        assert_eq!(painted(true, own), Rgba::rgb8(200, 30, 90));
        assert_eq!(painted(false, own), palette.accent_inactive);
    }
}

#[cfg(test)]
mod fullscreen_chrome_tests {
    use crate::chrome::{ChromeStyle, Side, TitlebarLayout, WindowChrome};
    use crate::sidebar::Sidebar;
    use crate::titlebar::Titlebar;
    use crate::views::split::SplitView;
    use freya_core::integration::AppComponent;
    use freya_core::prelude::*;
    use freya_testing::TestingRunner;
    use std::{cell::RefCell, rc::Rc};
    use torin::prelude::{Size, Size2D};
    use vow_appearance::{AppearanceConfig, resolve};
    use vow_design::TITLEBAR;

    use crate::platform::window::{WindowControls, WindowStatus};

    struct Case(ChromeStyle);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            crate::chrome::use_focus_bar_reveal();
            let detail = rect()
                .width(Size::fill())
                .height(Size::fill())
                .child(Titlebar::new().title("Ventana").with_sidebar(true));
            WindowChrome::new(
                self.0,
                SplitView::new(Sidebar::new().traffic_lights(true), detail),
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
        for layout in [
            TitlebarLayout::Top,
            TitlebarLayout::Rail,
            TitlebarLayout::Unified,
            TitlebarLayout::Pill,
            TitlebarLayout::Actions,
        ] {
            for side in [Side::Left, Side::Right] {
                let style = ChromeStyle {
                    layout,
                    side,
                    ..ChromeStyle::default()
                };
                let bar = Rc::new(RefCell::new(None));
                let shared = bar.clone();
                let resolved = resolve(&AppearanceConfig::default(), None);
                let (mut test, _) = TestingRunner::new(
                    AppComponent::new(Case(style)),
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
                assert_eq!(light_tops(&test), vec![], "{style:?} at rest");
                bar.borrow_mut().as_mut().unwrap().set(true);
                test.sync_and_update();
                test.poll(super::REVEAL_STEP, super::REVEAL_TIME);
                let lights = light_tops(&test);
                assert_eq!(lights.len(), 1, "{style:?} with the bar: {lights:?}");
                assert!(lights[0].1 >= 28, "{style:?} under the bar: {lights:?}");
            }
        }
    }
}
